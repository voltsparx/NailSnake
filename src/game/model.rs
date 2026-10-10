use std::collections::{HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::config::Difficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self, other: Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }

    pub(super) fn delta(self) -> (i16, i16) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Running,
    Paused,
    GameOver,
    Won,
    Menu,
}

/// Format version for resumable game snapshots.  Keeping this with the pure
/// game state makes incompatible saves fail closed instead of being guessed at.
pub const SAVE_VERSION: u32 = 1;
pub const MIN_BOARD_WIDTH: u16 = 4;
pub const MIN_BOARD_HEIGHT: u16 = 4;
/// Resume files are user-editable.  Keep their dimensions bounded so a
/// malformed file cannot turn rendering into an enormous allocation/workload.
pub const MAX_BOARD_WIDTH: u16 = 120;
pub const MAX_BOARD_HEIGHT: u16 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    #[serde(default)]
    pub save_version: u32,
    pub width: u16,
    pub height: u16,
    pub snake: VecDeque<Point>,
    pub direction: Direction,
    /// At most two turns may be buffered.  This is enough for a quick corner
    /// without letting key repeat turn the game into a command backlog.
    #[serde(default)]
    pub direction_queue: VecDeque<Direction>,
    pub food: Point,
    pub score: u32,
    pub food_eaten: u32,
    pub phase: GamePhase,
    pub difficulty: Difficulty,
    pub wrap_walls: bool,
    pub random_maze: bool,
    pub custom_speed_ms: u64,
    pub walls: HashSet<Point>,
    pub tick_count: u64,
}
