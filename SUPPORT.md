# Support

## Documentation

- **Man page:** `man nailsnake` (after installing)
- **Built-in help:** `nailsnake --help`
- **README:** See [README.md](README.md)

## Community

- **Issues:** https://github.com/voltsparx/NailSnake/issues
- **Discussions:** https://github.com/voltsparx/NailSnake/discussions

## Troubleshooting

### Terminal too small

NailSnake requires at least 60 columns by 22 rows. Resize your terminal
window and try again.

### "Not a TTY" error

Run NailSnake from an interactive terminal, not from a pipe, script, or CI.

### No color / wrong colors

Use the `--color` flag to force a color mode:

```bash
nailsnake --color truecolor
nailsnake --color 256
nailsnake --color basic
```

## Filing a Bug

Open an issue at https://github.com/voltsparx/NailSnake/issues with:

- Your operating system and terminal emulator
- The exact command you ran
- Full error output
- Steps to reproduce
