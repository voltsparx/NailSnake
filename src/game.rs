use std::collections::{HashSet, VecDeque};

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::config::Difficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn delta(self) -> (i16, i16) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Running,
    Paused,
    GameOver,
    Menu,
}

/// Core game state: snake body, food, score, and physics parameters.
///
/// The snake is stored as a `VecDeque` so pushing a new head and popping the
/// tail (when not eating) is O(1). The direction queue is minimal — we only
/// buffer one pending direction to prevent the player from reversing into
/// themselves within a single tick.
#[derive(Debug)]
pub struct Game {
    pub width: u16,
    pub height: u16,
    pub snake: VecDeque<Point>,
    pub direction: Direction,
    /// Buffered direction: applied at the next tick boundary so the player
    /// can press a key between ticks without it being swallowed.
    pub pending_direction: Direction,
    pub food: Point,
    pub score: u32,
    pub food_eaten: u32,
    pub phase: GamePhase,
    pub difficulty: Difficulty,
    pub wrap_walls: bool,
    /// Monotonically increasing tick counter, used for speed-up calculations
    /// and could drive animation or replay features later.
    pub tick_count: u64,
}

impl Game {
    pub fn new(width: u16, height: u16, difficulty: Difficulty, wrap_walls: bool) -> Self {
        let mid_x = width / 2;
        let mid_y = height / 2;
        let mut snake = VecDeque::new();
        snake.push_back(Point { x: mid_x, y: mid_y });
        snake.push_back(Point {
            x: mid_x.saturating_sub(1),
            y: mid_y,
        });
        snake.push_back(Point {
            x: mid_x.saturating_sub(2),
            y: mid_y,
        });

        let mut game = Self {
            width,
            height,
            snake,
            direction: Direction::Right,
            pending_direction: Direction::Right,
            food: Point { x: 0, y: 0 },
            score: 0,
            food_eaten: 0,
            phase: GamePhase::Running,
            difficulty,
            wrap_walls,
            tick_count: 0,
        };
        game.spawn_food();
        game
    }

    /// Full reset — equivalent to constructing a new `Game` with the same
    /// dimensions and settings, but updates in place.
    pub fn reset(&mut self) {
        *self = Game::new(self.width, self.height, self.difficulty, self.wrap_walls);
    }

    /// Proportionally remap all snake segments and food to a new board size.
    ///
    /// Returns `false` (without mutating) if the remap would cause a collision,
    /// so the caller can decide to start a fresh game instead.
    pub fn resize(&mut self, width: u16, height: u16) -> bool {
        if width == self.width && height == self.height {
            return true;
        }

        let scale_x = width as f32 / self.width as f32;
        let scale_y = height as f32 / self.height as f32;

        let remap_point = |p: Point| -> Point {
            let nx = ((p.x as f32 * scale_x).round() as u16).min(width.saturating_sub(1));
            let ny = ((p.y as f32 * scale_y).round() as u16).min(height.saturating_sub(1));
            Point { x: nx, y: ny }
        };

        let new_snake: VecDeque<Point> = self.snake.iter().map(|p| remap_point(*p)).collect();
        let new_food = remap_point(self.food);

        let mut seen = std::collections::HashSet::new();
        for p in &new_snake {
            if !seen.insert(*p) {
                return false;
            }
        }
        if new_snake.contains(&new_food) {
            return false;
        }

        self.snake = new_snake;
        self.food = new_food;
        self.width = width;
        self.height = height;
        true
    }

    /// Milliseconds between game ticks.
    ///
    /// Starts at the difficulty's base interval and gets faster by 8 ms every
    /// 5 food eaten, to a floor of 40 ms (~25 tps).
    pub fn tick_interval_ms(&self) -> u64 {
        let base = self.difficulty.tick_ms();
        let speedup = (self.food_eaten / 5).min(8) as u64 * 8;
        base.saturating_sub(speedup).max(40)
    }

    /// Set the pending direction, ignoring requests that would reverse the
    /// snake (which would cause instant self-collision).
    pub fn set_direction(&mut self, dir: Direction) {
        if self.phase != GamePhase::Running {
            return;
        }
        if dir.opposite(self.direction) {
            return;
        }
        self.pending_direction = dir;
    }

    pub fn toggle_pause(&mut self) {
        self.phase = match self.phase {
            GamePhase::Running => GamePhase::Paused,
            GamePhase::Paused => GamePhase::Running,
            other => other,
        };
    }

    /// Advance the game by one tick.
    ///
    /// Commits the buffered direction, computes the new head position, checks
    /// collisions (walls, self, or food), and returns `true` if the game ended
    /// (so the caller can persist stats).
    pub fn tick(&mut self) -> bool {
        if self.phase != GamePhase::Running {
            return false;
        }

        self.direction = self.pending_direction;
        self.tick_count += 1;

        let head = self.snake.front().copied().unwrap();
        let (dx, dy) = self.direction.delta();
        let mut nx = head.x as i16 + dx;
        let mut ny = head.y as i16 + dy;

        if self.wrap_walls {
            if nx < 0 {
                nx = self.width as i16 - 1;
            } else if nx >= self.width as i16 {
                nx = 0;
            }
            if ny < 0 {
                ny = self.height as i16 - 1;
            } else if ny >= self.height as i16 {
                ny = 0;
            }
        } else if nx < 0
            || ny < 0
            || nx >= self.width as i16
            || ny >= self.height as i16
        {
            self.phase = GamePhase::GameOver;
            return true;
        }

        let next_point = Point {
            x: nx as u16,
            y: ny as u16,
        };

        if self.snake.iter().any(|p| *p == next_point) {
            self.phase = GamePhase::GameOver;
            return true;
        }

        self.snake.push_front(next_point);

        if next_point == self.food {
            self.score += 10 + self.food_eaten;
            self.food_eaten += 1;
            self.spawn_food();
        } else {
            self.snake.pop_back();
        }

        false
    }

    /// Place food on a random empty cell.
    ///
    /// If every cell is occupied by the snake, the game is won (→ GameOver).
    /// Uses `HashSet` for the snake-body lookup to avoid O(n²) scans on large
    /// boards.
    fn spawn_food(&mut self) {
        let snake_set: HashSet<Point> = self.snake.iter().copied().collect();
        let total = self.width as usize * self.height as usize;
        let snake_len = self.snake.len();

        if snake_len >= total {
            self.phase = GamePhase::GameOver;
            return;
        }

        let mut empty = Vec::with_capacity(total - snake_len);
        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point { x, y };
                if !snake_set.contains(&p) {
                    empty.push(p);
                }
            }
        }

        self.food = *empty.choose(&mut thread_rng()).expect("non-empty vec");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_grows_when_eating_food() {
        let mut game = Game::new(10, 10, Difficulty::Normal, false);
        game.phase = GamePhase::Running;
        let initial_len = game.snake.len();
        if let Some(h) = game.snake.front() {
            game.food = Point {
                x: h.x + 1,
                y: h.y,
            };
        }
        game.pending_direction = Direction::Right;
        game.tick();
        assert_eq!(game.snake.len(), initial_len + 1);
        assert!(game.score > 0);
    }

    #[test]
    fn reverse_direction_is_ignored() {
        let mut game = Game::new(10, 10, Difficulty::Normal, false);
        game.phase = GamePhase::Running;
        game.set_direction(Direction::Left);
        assert_eq!(game.pending_direction, Direction::Right);
    }

    #[test]
    fn wall_collision_ends_game() {
        let mut game = Game::new(5, 5, Difficulty::Normal, false);
        game.phase = GamePhase::Running;
        game.snake.clear();
        game.snake.push_back(Point { x: 0, y: 0 });
        game.direction = Direction::Left;
        game.pending_direction = Direction::Left;
        game.tick();
        assert_eq!(game.phase, GamePhase::GameOver);
    }

    #[test]
    fn wrap_mode_wraps_coordinates() {
        let mut game = Game::new(5, 5, Difficulty::Normal, true);
        game.phase = GamePhase::Running;
        game.snake.clear();
        game.snake.push_back(Point { x: 0, y: 2 });
        game.direction = Direction::Left;
        game.pending_direction = Direction::Left;
        game.tick();
        assert_eq!(game.phase, GamePhase::Running);
        assert_eq!(game.snake.front(), Some(&Point { x: 4, y: 2 }));
    }

    #[test]
    fn pause_toggles_running_state() {
        let mut game = Game::new(8, 8, Difficulty::Normal, false);
        game.phase = GamePhase::Running;
        game.toggle_pause();
        assert_eq!(game.phase, GamePhase::Paused);
        game.toggle_pause();
        assert_eq!(game.phase, GamePhase::Running);
    }
}
