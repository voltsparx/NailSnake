#!/usr/bin/env bash
# Install nailsnake.1 for man(1) / man-db (Linux, macOS, BSD, Git Bash).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAN_SRC="${ROOT}/man/nailsnake.1"
PREFIX="${PREFIX:-/usr/local}"
MANDIR="${MANDIR:-${PREFIX}/share/man/man1}"

if [[ ! -f "${MAN_SRC}" ]]; then
  echo "error: missing ${MAN_SRC}" >&2
  exit 1
fi

# User-local install when not root and PREFIX is default system path
if [[ "${EUID:-$(id -u)}" -ne 0 && "${PREFIX}" == "/usr/local" && -z "${FORCE_SYSTEM:-}" ]]; then
  PREFIX="${HOME}/.local"
  MANDIR="${PREFIX}/share/man/man1"
fi

mkdir -p "${MANDIR}"
install -m 644 "${MAN_SRC}" "${MANDIR}/nailsnake.1"

if command -v mandb >/dev/null 2>&1; then
  mandb -q "${PREFIX}/share/man" 2>/dev/null || mandb -q || true
  echo "man-db updated."
elif command -v makewhatis >/dev/null 2>&1; then
  makewhatis "${MANDIR}" 2>/dev/null || true
  echo "makewhatis updated."
fi

echo "Installed: ${MANDIR}/nailsnake.1"
echo "Run: man nailsnake"
if [[ "${PREFIX}" == "${HOME}/.local" ]]; then
  echo "Tip: export MANPATH=\"\${HOME}/.local/share/man:\${MANPATH:-}\""
fi
