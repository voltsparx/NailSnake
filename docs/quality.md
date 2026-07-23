# Quality Checks

Use these commands before publishing NailSnake v1.0:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

Installer syntax checks:

```bash
bash -n installer/install-linux.sh
bash -n installer/install-macos.sh
bash -n installer/install-termux.sh
```

PowerShell parser check:

```powershell
$null = [System.Management.Automation.Language.Parser]::ParseFile(
  "installer/install-windows.ps1",
  [ref]$null,
  [ref]$null
)
```

Architectural review focus:

- Keep engine rules in `src/game/`.
- Keep terminal lifecycle and input handling in `src/app/`.
- Keep rendering-only behavior in `src/ui/`.
- Keep persistence in `src/config.rs`.
- Avoid introducing rendering allocations inside the game tick path.
