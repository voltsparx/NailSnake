#!/usr/bin/env sh
set -eu

APP_NAME="nailsnake"
DISPLAY_NAME="NailSnake"
VERSION="1.0.0"
IDENTIFIER="io.github.voltsparx.nailsnake"
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
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
    read answer
    answer=${answer:-$default}
    case "$answer" in y|Y|yes|YES) return 0 ;; *) return 1 ;; esac
}

if [ "$(uname -s)" != "Darwin" ]; then
    say "This installer must be run on macOS."
    exit 1
fi

command -v cargo >/dev/null 2>&1 || { say "cargo is required."; exit 1; }
command -v pkgbuild >/dev/null 2>&1 || { say "pkgbuild is required. Install Xcode Command Line Tools."; exit 1; }

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
