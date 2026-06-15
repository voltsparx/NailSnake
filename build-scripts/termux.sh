#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
#
# NailSnake: Cross-platform terminal Snake for Windows, Linux, and macOS
# Copyright (c) 2026 voltsparx
#
# Repository: https://github.com/voltsparx/NailSnake
# Contact: voltsparx@gmail.com
# License: See LICENSE file in the project root
#
# Termux (Android) specific build/install script

set -u
set -o pipefail

APP_NAME="nailsnake"
THEME_TAG="NailSnake"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INSTALL_ROOT="$HOME/.local/share/$APP_NAME"
USER_BIN_DIR="$HOME/.local/bin"
SYSTEM_LAUNCHER="$USER_BIN_DIR/$APP_NAME"
TEST_BASE="$SCRIPT_DIR/test-root"
TEST_ROOT="$TEST_BASE/$APP_NAME"
TEST_BIN_DIR="$TEST_BASE/bin"
TEST_LAUNCHER="$TEST_BIN_DIR/$APP_NAME"
MANIFEST_NAME=".nailsnake-install.json"
MODE="${1:-install}"
TERMUX_BIN_DIR="$PREFIX/bin"

log() { printf '[%s][%s] %s\n' "$THEME_TAG" "$1" "$2"; }
info() { log "INFO" "$1"; }
ok() { log " OK " "$1"; }
warn() { log "WARN" "$1"; }
fail() { log "FAIL" "$1"; exit 1; }

run_cmd() {
  "$@"
  local code=$?
  if [ "$code" -ne 0 ]; then
    fail "Command failed ($code): $*"
  fi
}

confirm_action() {
  local prompt="$1"
  local default="${2:-yes}"
  if [ ! -t 0 ]; then
    return 0
  fi

  local suffix="[Y/n]"
  [ "$default" = "no" ] && suffix="[y/N]"
  printf '%s %s ' "$prompt" "$suffix"
  read -r answer
  if [ -z "$answer" ]; then
    [ "$default" = "yes" ]
    return
  fi

  case "$(printf '%s' "$answer" | tr '[:upper:]' '[:lower:]')" in
    y|yes) return 0 ;;
    *) return 1 ;;
  esac
}

ensure_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    fail "$1 is required but was not found in PATH."
  fi
}

ensure_termux_packages() {
  local missing=()

  if ! command -v cargo >/dev/null 2>&1; then
    missing+=("rust")
  fi

  if ! command -v clang >/dev/null 2>&1; then
    missing+=("clang")
  fi

  if ! command -v pkg-config >/dev/null 2>&1; then
    missing+=("pkg-config")
  fi

  if [ ${#missing[@]} -gt 0 ]; then
    info "Missing Termux packages: ${missing[*]}"
    confirm_action "Install missing Termux packages?" yes || fail "Required packages not installed."

    run_cmd pkg update -y
    run_cmd pkg install -y "${missing[@]}"
  fi
}

manifest_path() { printf '%s/%s\n' "$1" "$MANIFEST_NAME"; }
is_managed_install() { [ -f "$(manifest_path "$1")" ]; }

ensure_managed_or_missing() {
  if [ -e "$1" ] && ! is_managed_install "$1"; then
    fail "Refusing to replace '$1' because it is not marked as an NailSnake managed install."
  fi
}

build_release() {
  info "Building release binary (Termux)."
  run_cmd cargo build --release
}

build_debug() {
  info "Building debug binary."
  run_cmd cargo build
}

run_tests() {
  info "Running tests."
  run_cmd cargo test
}

copy_binary() {
  local dest_dir="$1"
  run_cmd mkdir -p "$dest_dir"
  run_cmd cp "$REPO_ROOT/target/release/$APP_NAME" "$dest_dir/$APP_NAME"
  run_cmd chmod +x "$dest_dir/$APP_NAME"
}

write_manifest() {
  local app_root="$1"
  local install_mode="$2"
  local cargo_version
  cargo_version="$(cargo --version 2>/dev/null || printf 'unknown')"

  cat >"$(manifest_path "$app_root")" <<EOF
{
  "app_name": "$APP_NAME",
  "installed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "install_mode": "$install_mode",
  "source_repo": "$REPO_ROOT",
  "cargo_version": "$cargo_version",
  "platform": "termux"
}
EOF
}

write_launcher() {
  local app_root="$1"
  local launcher_path="$2"

  run_cmd mkdir -p "$(dirname "$launcher_path")"
  cat >"$launcher_path" <<EOF
#!/usr/bin/env sh
# NailSnake - terminal Snake game (Termux)
APP_ROOT="$app_root"
BINARY="\$APP_ROOT/bin/$APP_NAME"
if [ ! -f "\$BINARY" ]; then
  echo "[FAIL] NailSnake is not installed correctly at \$APP_ROOT." >&2
  exit 1
fi
exec "\$BINARY" "\$@"
EOF
  run_cmd chmod +x "$launcher_path"
}

termux_install_to_prefix() {
  info "Symlinking to Termux prefix: $TERMUX_BIN_DIR/$APP_NAME"
  run_cmd mkdir -p "$TERMUX_BIN_DIR"
  ln -sf "$INSTALL_ROOT/bin/$APP_NAME" "$TERMUX_BIN_DIR/$APP_NAME" 2>/dev/null || {
    run_cmd cp "$INSTALL_ROOT/bin/$APP_NAME" "$TERMUX_BIN_DIR/$APP_NAME"
    run_cmd chmod +x "$TERMUX_BIN_DIR/$APP_NAME"
  }
  ok "Binary linked to Termux prefix PATH."
}

termux_remove_from_prefix() {
  if [ -f "$TERMUX_BIN_DIR/$APP_NAME" ]; then
    rm -f "$TERMUX_BIN_DIR/$APP_NAME"
    ok "Removed binary from Termux prefix PATH."
  fi
}

smoke_test() {
  local launcher_path="$1"
  info "Running smoke test."
  "$launcher_path" --version >/dev/null 2>&1 || fail "Smoke test failed for $launcher_path."
  ok "Smoke test passed."
}

show_install_summary() {
  local install_mode="$1"
  local app_root="$2"
  local launcher_paths="$3"
  ok "NailSnake $install_mode completed successfully (Termux)."
  info "Installed application: $app_root"
  info "System command: $APP_NAME"
  info "Launcher paths: $launcher_paths"
  info "The binary is also available at $TERMUX_BIN_DIR/$APP_NAME"
}

deploy_install() {
  local target_root="$1"
  local launcher_path="$2"
  local install_mode="$3"
  local add_to_prefix="$4"
  local parent_dir
  local stage_root
  local stage_bin
  local backup_root

  ensure_managed_or_missing "$target_root"
  ensure_termux_packages

  parent_dir="$(dirname "$target_root")"
  stage_root="$parent_dir/.${APP_NAME}-staging-$$"
  stage_bin="$stage_root/bin"
  backup_root="$parent_dir/.${APP_NAME}-backup-$$"

  rm -rf "$stage_root" "$backup_root"
  run_cmd mkdir -p "$stage_bin"

  info "Preparing staged files for $install_mode."
  build_release
  copy_binary "$stage_bin"
  write_manifest "$stage_root" "$install_mode"

  [ -d "$target_root" ] && run_cmd mv "$target_root" "$backup_root"
  if ! mv "$stage_root" "$target_root"; then
    [ -d "$backup_root" ] && mv "$backup_root" "$target_root"
    fail "Unable to move the staged install into place."
  fi

  if ! write_launcher "$target_root" "$launcher_path"; then
    rm -rf "$target_root"
    [ -d "$backup_root" ] && mv "$backup_root" "$target_root"
    fail "Unable to create launcher in $(dirname "$launcher_path")."
  fi

  if [ "$add_to_prefix" = "yes" ]; then
    termux_install_to_prefix
  fi

  smoke_test "$launcher_path"
  rm -rf "$stage_root" "$backup_root"
  show_install_summary "$install_mode" "$target_root" "$launcher_path"
}

uninstall_system() {
  if [ -d "$INSTALL_ROOT" ]; then
    is_managed_install "$INSTALL_ROOT" || fail "Refusing to remove $INSTALL_ROOT because it is not marked as managed by this installer."
    run_cmd rm -rf "$INSTALL_ROOT"
    ok "Removed $INSTALL_ROOT"
  else
    warn "No managed installation was found at $INSTALL_ROOT."
  fi

  [ -f "$SYSTEM_LAUNCHER" ] && run_cmd rm -f "$SYSTEM_LAUNCHER" && ok "Removed launcher $SYSTEM_LAUNCHER"
  termux_remove_from_prefix
  ok "Uninstall complete."
}

case "$MODE" in
  install)
    deploy_install "$INSTALL_ROOT" "$SYSTEM_LAUNCHER" "install" "yes"
    ;;
  test)
    ensure_termux_packages
    build_debug
    run_tests
    deploy_install "$TEST_ROOT" "$TEST_LAUNCHER" "test" "no"
    info "Repo-local test binary: $TEST_LAUNCHER"
    ;;
  update)
    ensure_termux_packages
    is_managed_install "$INSTALL_ROOT" || fail "No managed installation found to update. Run install first."
    deploy_install "$INSTALL_ROOT" "$SYSTEM_LAUNCHER" "update" "yes"
    ;;
  uninstall)
    uninstall_system
    ;;
  build)
    ensure_termux_packages
    build_release
    ok "Release binary built at target/release/$APP_NAME"
    ;;
  check)
    ensure_termux_packages
    run_cmd cargo check
    ok "Code check passed."
    ;;
  run)
    ensure_termux_packages
    run_cmd cargo run -- "${@:2}"
    ;;
  *)
    fail "Unsupported mode '$MODE'. Use: install, test, update, uninstall, build, check, run"
    ;;
esac
