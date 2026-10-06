# NailSnake

[![Version](https://img.shields.io/badge/version-v1.0-dea584)](https://github.com/voltsparx/NailSnake)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![CI](https://github.com/voltsparx/NailSnake/actions/workflows/rust.yml/badge.svg)](https://github.com/voltsparx/NailSnake/actions/workflows/rust.yml)
[![Tests](https://img.shields.io/badge/tests-15%2F15-passing-brightgreen)](https://github.com/voltsparx/NailSnake/actions)
[![Platform: Windows | Linux | macOS](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS-blue)]()

A polished, full-screen terminal snake game written in Rust.  NailSnake runs on
**Windows**, **Linux**, and **macOS** - dropping you into the alternate screen
buffer with raw keyboard input, just like vim or neovim.  It *feels* like a
lightweight GUI without ever leaving your terminal.

> Inspired by [nsnake](https://github.com/alexdantas/nSnake), rebuilt with
> Rust's safety guarantees, richer colour palettes, persistent high-scores,
> and a proper `man` page.

---

## Features

| Capability | Detail |
|------------|--------|
| **Cross-platform** | Windows Terminal, PowerShell, cmd, Linux VTs, macOS Terminal, iTerm2 |
| **Vim-like TUI** | Alternate screen, hidden cursor, status bar, sidebar info panel |
| **Rich colours** | Truecolor, 256-colour, and basic ANSI - auto-detected or forced with `--color` |
| **Safe terminal handling** | Restores your shell on quit, panic, or Ctrl+C |
| **Continue game** | Saves an active session on quit/force-quit and offers Continue on next launch |
| **Safe resize** | Keeps the logical game intact; small windows show a resize warning |
| **Difficulty presets** | Chill, Normal, Hard, Insane - each with progressive speed-up |
| **Arcade menu** | nSnake-inspired main menu with settings, help, and an animated Rust-themed backdrop |
| **Wrap mode** | Optional wall-wrapping instead of instant death |
| **Persistent settings** | Settings, key bindings, high score, and games played survive restart |
| **Manual page** | `man nailsnake` after installing the man page |

---

## Quick start

```bash
# Run in one shot - no install needed
cargo run --release

# Or install globally
cargo install --path .
```

## Packaging And Installers

NailSnake includes packaging helpers for release builds:

```bash
# Arch Linux package with makepkg
makepkg -f

# Linux interactive packager/installer
./installer/install-linux.sh

# Termux on Android
./installer/install-termux.sh

# macOS .pkg builder/installer
./installer/install-macos.sh
```

```powershell
# Windows PowerShell installer
.\installer\install-windows.ps1

# cmd.exe wrapper
.\installer\install-windows.cmd
```

The Linux installer detects the active package manager (`apt`, `pacman`, `dnf`,
`zypper`, `xbps`, `yum`, or `apk`) and can create `.deb`, `.pkg.tar.zst`,
`.rpm`, `.xbps`, or generic `.tar.gz` packages. It checks prerequisites first,
asks before installing missing tools, then asks again before installing the
created package.

The Termux installer builds a Termux-native binary, creates a `.tar.gz` archive,
and can install it into `$PREFIX/bin`.

The macOS installer creates a CLI-only `.pkg` that installs to `/usr/local/bin`
and does not create an app launcher icon.

The Windows installer builds `nailsnake.exe`, installs it under either the
current user's local Programs directory or `Program Files`, and updates PATH so
both cmd.exe and PowerShell can run `nailsnake` from a new terminal window.

### Linux / macOS - binary + man page

```bash
make install                    # /usr/local/bin + man page
# user-local (no root):
PREFIX=$HOME/.local make install
export MANPATH="$HOME/.local/share/man:${MANPATH:-}"
```

Or the man page alone:

```bash
./scripts/install-man.sh
# or
make install-man
```

Then:

```bash
man nailsnake
```

### Windows

Build and run in **Windows Terminal**, PowerShell, or cmd:

```powershell
cargo install --path .
nailsnake
```

Windows has no built-in `man`.  Install the manual for **Git Bash / MSYS2**:

```powershell
.\scripts\install-man.ps1 -UserLocal
```

Or read the man source at `man/nailsnake.1`, or use `nailsnake --help`.

---

## Controls

| Key | Action |
|-----|--------|
| `Enter` | Start from title screen |
| Configured movement keys | Move (arrow keys by default) |
| Configured pause key / `Esc` | Pause / resume |
| `R` | Restart |
| `Q` | Open the pause menu while playing; quit from the main menu |
| `Esc` | Pause, go back, or quit from the main menu depending on context |
| `Ctrl+C` | Force quit (terminal restored) |

---

## CLI options

```
nailsnake [OPTIONS]

Options:
  -d, --difficulty <DIFFICULTY>  chill | normal | hard | insane
  -w, --wrap                     wrap around walls for this launch
  -c, --color <COLOR>            auto | truecolor | ansi256 | basic
  -g, --grid                     show grid dots for this launch
      --about                    print project information
  -h, --help                     print help (see also: man nailsnake)
  -V, --version                  print version
```

---

## Examples

```bash
# Casual
nailsnake

# Bring the heat
nailsnake -d hard --wrap --grid

# Force a specific colour mode
nailsnake --color truecolor

# Print project metadata
nailsnake --about

# Read the full manual
man nailsnake
```

---

## Settings and save files

| OS | Path |
|----|------|
| Linux / BSD | `~/.config/NailSnake/settings.json`, `stats.json` |
| macOS | `~/Library/Application Support/NailSnake/settings.json`, `stats.json` |
| Windows | `%APPDATA%\NailSnake\settings.json`, `stats.json` |

Active game resume data is stored separately as a versioned `active-game.json`
in the platform data directory. Saves are written through a synced temporary
file before replacement; invalid or incompatible snapshots are discarded.
`NO_COLOR` forces the basic palette when `--color` is not supplied. See
[`docs/persistence.md`](docs/persistence.md).

---

## Requirements

- **Rust 1.70+** (edition 2021)
- **Terminal**: full UI at **90x28**; compact/minimal layouts down to **50x18**
- **Interactive TTY** (not a piped or scripted session)

## Documentation

Detailed documentation lives in [`docs/`](docs/).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for development setup, pull request
guidelines, and code style conventions.

## Security

See [`SECURITY.md`](SECURITY.md) for supported versions and how to report
vulnerabilities.

## Attribution

**Author:** Voltsparx - **Contact:** [voltsparx@gmail.com](mailto:voltsparx@gmail.com)

## License

MIT - Copyright (c) 2026 Voltsparx
