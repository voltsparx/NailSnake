#!/usr/bin/env sh
set -eu

APP_NAME="nailsnake"
DISPLAY_NAME="NailSnake"
IDENTIFIER="io.github.voltsparx.nailsnake"
ROOT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
VERSION=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1)
DIST_DIR="$ROOT_DIR/dist"
STAGE_DIR="$DIST_DIR/macos-pkg-root"
PKG_PATH="$DIST_DIR/$APP_NAME-$VERSION-macos.pkg"

say() {
    printf '%s\n' "$*"
}

ask_yes_no() {
    prompt=$1
    default=${2:-n}
    if [ "$default" = "y" ]; then suffix="[Y/n]"; else suffix="[y/N]"; fi
    printf '%s %s ' "$prompt" "$suffix"
    read -r answer
    answer=${answer:-$default}
    case "$answer" in y|Y|yes|YES) return 0 ;; *) return 1 ;; esac
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1
}

if [ "$(uname -s)" != "Darwin" ]; then
    say "This installer must be run on macOS."
    exit 1
fi

if ! need_cmd cargo; then
    say "Rust/Cargo is required."
    if need_cmd brew && ask_yes_no "Install Rust with Homebrew now?" "y"; then
        brew install rust
    else
        say "Install Rust from https://rustup.rs/ or with Homebrew, then re-run this script."
        exit 1
    fi
fi

if ! need_cmd pkgbuild; then
    say "pkgbuild is required and is provided by Xcode Command Line Tools."
    if ask_yes_no "Open the Xcode Command Line Tools installer now?" "y"; then
        xcode-select --install || true
    fi
    say "After the tools finish installing, re-run this script."
    exit 1
fi

mkdir -p "$DIST_DIR"
say "Building $DISPLAY_NAME release binary..."
cd "$ROOT_DIR"
cargo build --release --locked

rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/usr/local/bin" "$STAGE_DIR/usr/local/share/man/man1" \
    "$STAGE_DIR/usr/local/share/doc/$APP_NAME" "$STAGE_DIR/usr/local/share/licenses/$APP_NAME"
cp "$ROOT_DIR/target/release/$APP_NAME" "$STAGE_DIR/usr/local/bin/$APP_NAME"
cp "$ROOT_DIR/man/$APP_NAME.1" "$STAGE_DIR/usr/local/share/man/man1/$APP_NAME.1"
cp "$ROOT_DIR/README.md" "$STAGE_DIR/usr/local/share/doc/$APP_NAME/README.md"
cp "$ROOT_DIR/LICENSE" "$STAGE_DIR/usr/local/share/licenses/$APP_NAME/LICENSE"

pkgbuild \
    --root "$STAGE_DIR" \
    --identifier "$IDENTIFIER" \
    --version "$VERSION" \
    --install-location / \
    "$PKG_PATH"

say "Created package: $PKG_PATH"
say "This installs only a CLI utility under /usr/local/bin; no app icon is created."

if ask_yes_no "Install it system-wide now? This may ask for administrator privileges." "n"; then
    sudo installer -pkg "$PKG_PATH" -target /
    say "$DISPLAY_NAME installed. Run: $APP_NAME"
else
    say "Package left at: $PKG_PATH"
fi
