#!/usr/bin/env sh
# Build the release binary and place a directly runnable copy in dist/.
set -eu

ROOT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
DIST_DIR="$ROOT_DIR/dist"
SOURCE_BINARY="$ROOT_DIR/target/release/nailsnake"
OUTPUT_BINARY="$DIST_DIR/nailsnake"

printf '%s\n' "Building NailSnake release binary..."
mkdir -p "$DIST_DIR"
cd "$ROOT_DIR"
CARGO_TERM_VERBOSE=true CARGO_TERM_PROGRESS_WHEN=always CARGO_TERM_PROGRESS_WIDTH=80 \
    cargo build --release --locked --verbose

cp "$SOURCE_BINARY" "$OUTPUT_BINARY"
chmod +x "$OUTPUT_BINARY"
printf '%s\n' "Ready: $OUTPUT_BINARY"
