# Contributing to NailSnake

Thank you for your interest in contributing! NailSnake is a cross-platform
terminal Snake game built with Rust and ratatui.

## Getting Started

1. Fork the repository.
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/NailSnake.git
   cd NailSnake
   ```
3. Set up your Rust environment:
   ```bash
   rustup update stable
   cargo check
   ```

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

All tests must pass before submitting a pull request.

### Linting

```bash
cargo clippy
```

The project enforces zero clippy warnings.

### Manual

If you modify CLI flags or behaviour, update the man page at `man/nailsnake.1`.

## Pull Request Guidelines

- Keep changes focused. One feature/fix per PR.
- Write a clear, descriptive PR title and summary.
- Include steps to reproduce the bug or verify the feature.
- Add or update tests for any functional changes.
- Run `cargo test` and `cargo clippy` before submitting.
- Update the man page if the CLI interface changes.

## Code Style

- Follow standard Rust formatting (`cargo fmt`).
- No trailing whitespace.
- Prefer explicit error handling with `anyhow::Result` over panics.
- Document public items with doc comments (`///`).

## Reporting Issues

Report bugs and suggest features by opening a GitHub issue at
https://github.com/voltsparx/NailSnake/issues.

## Security

Found a security issue? Please read SECURITY.md before opening a public issue.
