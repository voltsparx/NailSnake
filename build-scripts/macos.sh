#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
#
# NailSnake: Cross-platform terminal Snake for Windows, Linux, and macOS
# Copyright (c) 2026 voltsparx
#
# Repository: https://github.com/voltsparx/NailSnake
# Contact: voltsparx@gmail.com
# License: See LICENSE file in the project root

set -u
set -o pipefail

APP_NAME="nailsnake"
THEME_TAG="NailSnake"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INSTALL_ROOT="$HOME/Library/Application Support/$APP_NAME"
USER_BIN_DIR="$HOME/.local/bin"
SYSTEM_LAUNCHER="$USER_BIN_DIR/$APP_NAME"
MAN_PAGE_DEST="$INSTALL_ROOT/man/man1/$APP_NAME.1"
MAN_SRC="$REPO_ROOT/man/$APP_NAME.1"
TEST_BASE="$SCRIPT_DIR/test-root"
TEST_ROOT="$TEST_BASE/$APP_NAME"
TEST_BIN_DIR="$TEST_BASE/bin"
TEST_LAUNCHER="$TEST_BIN_DIR/$APP_NAME"
MANIFEST_NAME=".nailsnake-install.json"
MODE="${1:-install}"
PROFILE_FILES=("$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.bashrc" "$HOME/.profile")

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

ensure_rust() {
  ensure_command cargo
}

manifest_path() { printf '%s/%s\n' "$1" "$MANIFEST_NAME"; }
is_managed_install() { [ -f "$(manifest_path "$1")" ]; }

ensure_managed_or_missing() {
  if [ -e "$1" ] && ! is_managed_install "$1"; then
    fail "Refusing to replace '$1' because it is not marked as an NailSnake managed install."
  fi
}

build_release() {
  info "Building release binary."
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

install_man_page() {
  if [ -f "$MAN_SRC" ]; then
    run_cmd mkdir -p "$(dirname "$MAN_PAGE_DEST")"
    run_cmd cp "$MAN_SRC" "$MAN_PAGE_DEST"
    run_cmd chmod 644 "$MAN_PAGE_DEST"
    if command -v makewhatis >/dev/null 2>&1; then
      makewhatis "$(dirname "$MAN_PAGE_DEST")" 2>/dev/null || true
    fi
    ok "Man page installed."
  else
    warn "Man page source not found at $MAN_SRC, skipping."
  fi
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
  "cargo_version": "$cargo_version"
}
EOF
}

write_launcher() {
  local app_root="$1"
  local launcher_path="$2"

  run_cmd mkdir -p "$(dirname "$launcher_path")"
  cat >"$launcher_path" <<EOF
#!/usr/bin/env sh
# NailSnake - terminal Snake game
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

ensure_path_block() {
  run_cmd mkdir -p "$USER_BIN_DIR"
  local marker_start="# >>> nailsnake >>>"
  local marker_end="# <<< nailsnake <<<"
  local profile
  local written="no"

  for profile in "${PROFILE_FILES[@]}"; do
    [ -f "$profile" ] || : >"$profile" || continue
    if grep -Fq "$marker_start" "$profile" 2>/dev/null; then
      continue
    fi

    {
      printf '\n%s\n' "$marker_start"
      printf '%s\n' 'export PATH="$HOME/.local/bin:$PATH"'
      printf '%s\n' "export MANPATH=\"$INSTALL_ROOT/man:\${MANPATH:-}\""
      printf '%s\n' "$marker_end"
    } >>"$profile" || continue
    written="yes"
  done

  if [ "$written" = "yes" ]; then
    ok "Added PATH and MANPATH updates to shell profiles."
  else
    info "Shell profile updates already present."
  fi
}

remove_profile_block() {
  local marker_start="# >>> nailsnake >>>"
  local marker_end="# <<< nailsnake <<<"
  local profile

  for profile in "${PROFILE_FILES[@]}"; do
    [ -f "$profile" ] || continue
    awk -v start="$marker_start" -v end="$marker_end" '
      $0 == start { skip = 1; next }
      $0 == end { skip = 0; next }
      !skip { print }
    ' "$profile" >"$profile.tmp" && mv "$profile.tmp" "$profile"
  done
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
  ok "NailSnake $install_mode completed successfully."
  info "Installed application: $app_root"
  info "System command: $APP_NAME"
  info "Launcher paths: $launcher_paths"
  info "Man page: $app_root/man/man1/$APP_NAME.1"
  info "Reload your shell or run: export PATH=\"$USER_BIN_DIR:\$PATH\" && export MANPATH=\"$app_root/man:\${MANPATH:-}\""
}

deploy_install() {
  local target_root="$1"
  local launcher_path="$2"
  local install_mode="$3"
  local add_to_profile="$4"
  local parent_dir
  local stage_root
  local stage_bin
  local backup_root

  ensure_managed_or_missing "$target_root"
  ensure_rust

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

  install_man_page

  if [ "$add_to_profile" = "yes" ]; then
    ensure_path_block
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
  remove_profile_block
  ok "Shell profile updates removed."
}

case "$MODE" in
  install)
    deploy_install "$INSTALL_ROOT" "$SYSTEM_LAUNCHER" "install" "yes"
    ;;
  test)
    ensure_rust
    build_debug
    run_tests
    deploy_install "$TEST_ROOT" "$TEST_LAUNCHER" "test" "no"
    info "Repo-local test binary: $TEST_LAUNCHER"
    ;;
  update)
    ensure_rust
    is_managed_install "$INSTALL_ROOT" || fail "No managed installation found to update. Run install first."
    deploy_install "$INSTALL_ROOT" "$SYSTEM_LAUNCHER" "update" "yes"
    ;;
  uninstall)
    uninstall_system
    ;;
  build)
    ensure_rust
    build_release
    ok "Release binary built at target/release/$APP_NAME"
    ;;
  check)
    ensure_rust
    run_cmd cargo check
    ok "Code check passed."
    ;;
  run)
    ensure_rust
    run_cmd cargo run -- "${@:2}"
    ;;
  *)
    fail "Unsupported mode '$MODE'. Use: install, test, update, uninstall, build, check, run"
    ;;
esac
