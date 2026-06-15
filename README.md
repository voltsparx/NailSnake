# NailSnake

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![CI](https://github.com/voltsparx/NailSnake/actions/workflows/rust.yml/badge.svg)](https://github.com/voltsparx/NailSnake/actions/workflows/rust.yml)
[![Tests](https://img.shields.io/badge/tests-12%2F12-passing-brightgreen)](https://github.com/voltsparx/NailSnake/actions)
[![Platform: Windows | Linux | macOS](https://img.shields.io/badge/Platform-Windows%20|%20Linux%20|%20macOS-blue)]()

A polished, full-screen terminal snake game written in Rust.  NailSnake runs on
**Windows**, **Linux**, and **macOS** — dropping you into the alternate screen
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
| **Rich colours** | Truecolor, 256-colour, and basic ANSI — auto-detected or forced with `--color` |
| **Safe terminal handling** | Restores your shell on quit, panic, or Ctrl+C |
| **Live resize** | Adapts seamlessly when the terminal window is resized |
| **Difficulty presets** | Chill, Normal, Hard, Insane — each with progressive speed-up |
| **Wrap mode** | Optional wall-wrapping instead of instant death |
| **Persistent stats** | High score saved per OS config directory (see below) |
| **Manual page** | `man nailsnake` after installing the man page |

---

## Quick start

```bash
# Run in one shot — no install needed
cargo run --release

# Or install globally
cargo install --path .
```

### Linux / macOS — binary + man page

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
| `↑` `↓` `←` `→` | Move |
| `W` `A` `S` `D` | Move (alternate) |
| `Space` | Pause / resume |
| `R` | Restart |
| `Q` / `Esc` | Quit |
| `Ctrl+C` | Force quit (terminal restored) |

---

## CLI options

```
nailsnake [OPTIONS]

Options:
  -d, --difficulty <DIFFICULTY>  chill | normal | hard | insane [default: normal]
  -w, --wrap                     wrap around walls
  -c, --color <COLOR>            auto | truecolor | 256 | basic [default: auto]
  -g, --grid                     show grid dots
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

# Read the full manual
man nailsnake
```

---

## Stats file locations

| OS | Path |
|----|------|
| Linux / BSD | `~/.config/NailSnake/stats.json` |
| macOS | `~/Library/Application Support/NailSnake/stats.json` |
| Windows | `%APPDATA%\NailSnake\stats.json` |

---

## Requirements

- **Rust 1.70+** (edition 2021)
- **Terminal** at least **60×22** characters
- **Interactive TTY** (not a piped or scripted session)



## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for development setup, pull request
guidelines, and code style conventions.

## Security

See [`SECURITY.md`](SECURITY.md) for supported versions and how to report
vulnerabilities.

## Attribution

**Author:** Voltsparx · **Contact:** [voltsparx@gmail.com](mailto:voltsparx@gmail.com)

## License

MIT — Copyright (c) 2026 Voltsparx
