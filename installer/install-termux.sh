#!/usr/bin/env sh
set -eu

APP_NAME="nailsnake"
DISPLAY_NAME="NailSnake"
ROOT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
VERSION=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1)
DIST_DIR="$ROOT_DIR/dist"
STAGE_DIR="$DIST_DIR/termux-stage"
PACKAGE_PATH="$DIST_DIR/${APP_NAME}-${VERSION}-termux-$(uname -m).tar.gz"

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

if [ -z "${PREFIX:-}" ] || [ ! -d "$PREFIX" ] || ! printf '%s' "$PREFIX" | grep -qi termux; then
    say "This installer is intended for Termux on Android."
    say "For regular Linux, use: ./installer/install-linux.sh"
    exit 1
fi

if ! need_cmd cargo || ! need_cmd clang || ! need_cmd pkg-config; then
    say "Missing Termux build packages."
    if ask_yes_no "Install rust, clang, and pkg-config with pkg?" "y"; then
        pkg update -y
        pkg install -y rust clang pkg-config
    else
        say "Cannot continue without Rust and build tools."
        exit 1
    fi
fi

mkdir -p "$DIST_DIR"
say "Building $DISPLAY_NAME release binary for Termux..."
cd "$ROOT_DIR"
cargo build --release --locked --verbose

rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR/bin" "$STAGE_DIR/share/man/man1" "$STAGE_DIR/share/doc/$APP_NAME"
cp "$ROOT_DIR/target/release/$APP_NAME" "$STAGE_DIR/bin/$APP_NAME"
cp "$ROOT_DIR/man/$APP_NAME.1" "$STAGE_DIR/share/man/man1/$APP_NAME.1"
cp "$ROOT_DIR/README.md" "$STAGE_DIR/share/doc/$APP_NAME/README.md"
cp "$ROOT_DIR/LICENSE" "$STAGE_DIR/share/doc/$APP_NAME/LICENSE"

(cd "$STAGE_DIR" && tar -czf "$PACKAGE_PATH" .)
say "Created Termux package archive: $PACKAGE_PATH"

if ask_yes_no "Install to Termux prefix now?" "n"; then
    mkdir -p "$PREFIX/bin" "$PREFIX/share/man/man1" "$PREFIX/share/doc/$APP_NAME"
    cp "$STAGE_DIR/bin/$APP_NAME" "$PREFIX/bin/$APP_NAME"
    cp "$STAGE_DIR/share/man/man1/$APP_NAME.1" "$PREFIX/share/man/man1/$APP_NAME.1"
    cp "$STAGE_DIR/share/doc/$APP_NAME/README.md" "$PREFIX/share/doc/$APP_NAME/README.md"
    cp "$STAGE_DIR/share/doc/$APP_NAME/LICENSE" "$PREFIX/share/doc/$APP_NAME/LICENSE"
    chmod +x "$PREFIX/bin/$APP_NAME"
    say "$DISPLAY_NAME installed. Run: $APP_NAME"
else
    say "Archive left at: $PACKAGE_PATH"
fi
