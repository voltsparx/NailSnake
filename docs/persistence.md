# Persistence

NailSnake keeps long-term stats and resumable active games in separate files.
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

NailSnake validates saved games before showing Continue. Invalid, completed, or
corrupt snapshots are ignored and removed.

The game saves active sessions when quitting during Running or Paused states,
including Ctrl+C force quit. Starting a new Arcade Mode clears the saved session.
