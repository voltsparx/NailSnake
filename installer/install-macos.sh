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
MIN_RUST_MINOR=88

say() {
    printf '%s\n' "$*"
}

ask_yes_no() {
    prompt=$1
    default=${2:-n}
    if [ "$default" = "y" ]; then suffix="[Y/n]"; else suffix="[y/N]"; fi
    printf '%s %s ' "$prompt" "$suffix"
    read -r answer || answer=""
    answer=${answer:-$default}
    case "$answer" in y|Y|yes|YES) return 0 ;; *) return 1 ;; esac
}

need_cmd() {
    command -v "$1" >/dev/null 2>&1
}

rust_toolchain_ok() {
    version=$(rustc --version | awk '{print $2}')
    major=$(printf '%s' "$version" | awk -F. '{print $1}')
    minor=$(printf '%s' "$version" | awk -F. '{print $2}')
    [ "${major:-0}" -gt 1 ] || { [ "${major:-0}" -eq 1 ] && [ "${minor:-0}" -ge "$MIN_RUST_MINOR" ]; }
}

ensure_rust_toolchain() {
    if rust_toolchain_ok; then
        say "Rust toolchain: $(rustc --version)"
        return 0
    fi
    if need_cmd rustup; then
        say "NailSnake requires Rust 1.$MIN_RUST_MINOR+; installing rustup stable..."
        rustup toolchain install stable
        export RUSTUP_TOOLCHAIN=stable
        rust_toolchain_ok
        say "Rust toolchain: $(rustc --version)"
    else
        say "Rust 1.$MIN_RUST_MINOR+ is required. Install or update it with rustup, then re-run this installer."
        return 1
    fi
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

ensure_rust_toolchain

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
say "This may take a while on first run. (profile: release-installer)"
cd "$ROOT_DIR"
CARGO_TERM_VERBOSE=true CARGO_TERM_PROGRESS_WHEN=always CARGO_TERM_PROGRESS_WIDTH=80 \
    cargo build --profile release-installer --locked --verbose

say "Build complete."
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/usr/local/bin" "$STAGE_DIR/usr/local/share/man/man1" \
    "$STAGE_DIR/usr/local/share/doc/$APP_NAME" "$STAGE_DIR/usr/local/share/licenses/$APP_NAME"
cp "$ROOT_DIR/target/release-installer/$APP_NAME" "$STAGE_DIR/usr/local/bin/$APP_NAME"
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
