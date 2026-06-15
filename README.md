# NailSnake

A polished, full-screen terminal snake game written in Rust. NailSnake runs on **Windows**, **Linux**, and **macOS** using the alternate screen buffer and raw mode input — the same approach as vim/neovim — so it feels like a lightweight GUI while staying pure CLI.

Inspired by [nsnake](https://github.com/alexdantas/nSnake), rebuilt with Rust safety, richer colors, persistent high scores, and a `man` manual page.

## Features

- **Cross-platform** — Windows Terminal, PowerShell, cmd, Linux VTs, macOS Terminal/iTerm
- **Vim-like TUI** — alternate screen, hidden cursor, status bar, sidebar info panel
- **Rich colors** — truecolor, 256-color, and basic ANSI (`--color auto`)
- **Safe terminal handling** — restores your shell on quit, panic, or Ctrl+C
- **Live resize** — adapts when the terminal window is resized
- **Difficulty presets** — Chill, Normal, Hard, Insane with progressive speed-up
- **Wrap mode** — optional wall wrapping instead of instant death
- **Persistent stats** — high score saved per OS config directory
- **Manual page** — `man nailsnake` after installing the man page (man-db on Linux)

## Install

### From source (all platforms)

```bash
cargo install --path .
```

Or run without installing:

```bash
cargo run --release
```

### Linux / macOS — binary + man page

```bash
make install          # /usr/local/bin + man page + mandb
# user-local (no root):
PREFIX=$HOME/.local make install
export MANPATH="$HOME/.local/share/man:${MANPATH:-}"
```

Or man page only:

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

Windows has no built-in `man`. Install the manual for **Git Bash / MSYS2**:

```powershell
.\scripts\install-man.ps1 -UserLocal
```

Or read the man source: `man/nailsnake.1`, or use `nailsnake --help`.

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

## Stats file locations

| OS | Path |
|----|------|
| Linux | `~/.config/NailSnake/stats.json` |
| macOS | `~/Library/Application Support/NailSnake/stats.json` |
| Windows | `%APPDATA%\NailSnake\stats.json` |

## Examples

```bash
nailsnake
nailsnake -d hard --wrap --grid
nailsnake --color truecolor
man nailsnake
```

## Requirements

- Rust 1.70+ (edition 2021)
- Terminal at least **60×22** characters
- Interactive TTY (not piped output)

### Windows build note

If `link.exe` is missing (no Visual Studio C++ Build Tools), use the GNU toolchain with Scoop MinGW:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
$env:Path = "$env:USERPROFILE\scoop\apps\mingw\current\bin;" + $env:Path
cargo +stable-x86_64-pc-windows-gnu build --release
.\target\release\nailsnake.exe
```

The project `.cargo/config.toml` already points the GNU linker at MinGW.

## Atribution

**Author:** Voltsparx · **Contact:** [voltsparx@gmail.com](mailto:voltsparx@gmail.com)

## License

MIT — Copyright (c) 2026 Voltsparx
