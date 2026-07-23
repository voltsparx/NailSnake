# Architecture

NailSnake v1.0 keeps the root files small and moves focused behavior into
subfolders under `src/`.

## Source Layout

- `src/main.rs` - CLI parsing, startup checks, and top-level error reporting.
- `src/app.rs` - application lifecycle and render loop.
- `src/app/` - input handling, menu state, menu views, key bindings, terminal setup.
- `src/config.rs` - runtime config, stats persistence, active game persistence.
- `src/game.rs` - public game module facade.
- `src/game/` - game model, engine logic, and game unit tests.
- `src/platform.rs` - platform detection and terminal checks.
- `src/platform/` - hardware resource preflight checks.
- `src/theme.rs` - public theme module facade.
- `src/theme/` - theme data, palettes, and snake segment styling.
- `src/ui.rs` - top-level UI renderer.
- `src/ui/` - board, menu, sidebar, status, layout, overlay, animation, and skin rendering.

## Runtime Flow

1. `main.rs` parses CLI options.
2. Startup checks verify basic hardware resources, TTY availability, and terminal size.
3. `App::new` enters alternate-screen raw terminal mode and creates the initial game.
4. `App::run` renders at a capped frame rate and ticks the game at the configured speed.
5. Quit paths restore the terminal and save active games when appropriate.

## Performance Notes

- Rendering is capped at 30 FPS to reduce idle CPU use.
- Food spawning uses one-pass random selection instead of allocating a full list of empty cells.
- The game board is clamped to a practical terminal size to keep render work bounded.
