use std::collections::VecDeque;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug)]
pub struct Game {
    pub width: u16,
    pub height: u16,
    pub snake: VecDeque<Point>,
    pub direction: Direction,
    pub pending_direction: Direction,
    pub food: Point,
    pub score: u32,
    pub food_eaten: u32,
    pub phase: GamePhase,
    pub difficulty: Difficulty,
    pub wrap_walls: bool,
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

    pub fn reset(&mut self) {
        *self = Game::new(self.width, self.height, self.difficulty, self.wrap_walls);
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        if width == self.width && height == self.height {
            return;
        }
        let difficulty = self.difficulty;
        let wrap = self.wrap_walls;
        *self = Game::new(width, height, difficulty, wrap);
    }

    pub fn tick_interval_ms(&self) -> u64 {
        let base = self.difficulty.tick_ms();
        let speedup = (self.food_eaten / 5).min(8) as u64 * 8;
        base.saturating_sub(speedup).max(40)
    }

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

    fn spawn_food(&mut self) {
        let mut empty: Vec<Point> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point { x, y };
                if !self.snake.contains(&p) {
                    empty.push(p);
                }
            }
        }

        if empty.is_empty() {
            self.phase = GamePhase::GameOver;
            return;
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
