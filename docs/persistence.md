# Persistence

NailSnake keeps settings, long-term stats, and resumable active games in separate files.
This avoids mixing high-score data with emergency resume data.

## Stats

Stats are saved as JSON in the platform config directory:

| OS | Path |
| --- | --- |
| Linux / BSD | `~/.config/NailSnake/stats.json` |
| macOS | `~/Library/Application Support/NailSnake/stats.json` |
| Windows | `%APPDATA%\NailSnake\stats.json` |

## Active Game

Active game snapshots are saved as `active-game.json` in the platform local data
directory. The snapshot is written through a temporary file and then renamed, so
partial writes are less likely to leave corrupt data.

Snapshots include a save format version. NailSnake validates board dimensions,
snake uniqueness/continuity, food, walls, direction queue, and version before
showing Continue. Invalid, completed, or corrupt snapshots are ignored and removed.

The game saves active sessions when quitting during Running or Paused states,
including Ctrl+C force quit. Starting a new Arcade Mode clears the saved session.

## Settings

`settings.json` lives beside `stats.json` in the configuration directory. It
stores difficulty, wrapping, color preference, grid, maze mode, skin, custom
speed, and the five configurable bindings. Settings use defaults when missing;
malformed settings/stat files are retained with a `.corrupt-<timestamp>.json`
suffix before defaults are used.
