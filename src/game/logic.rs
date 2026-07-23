use std::collections::{HashSet, VecDeque};

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::config::Difficulty;

use super::{Direction, Game, GamePhase, Point};

impl Game {
    pub fn new(width: u16, height: u16, difficulty: Difficulty, wrap_walls: bool) -> Self {
        Self::with_options(
            width,
            height,
            difficulty,
            wrap_walls,
            false,
            difficulty.tick_ms(),
        )
    }

    pub fn with_options(
        width: u16,
        height: u16,
        difficulty: Difficulty,
        wrap_walls: bool,
        random_maze: bool,
        custom_speed_ms: u64,
    ) -> Self {
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
            random_maze,
            custom_speed_ms,
            walls: HashSet::new(),
            tick_count: 0,
        };
        game.generate_maze();
        game.spawn_food();
        game
    }

    pub fn reset(&mut self) {
        *self = Game::with_options(
            self.width,
            self.height,
            self.difficulty,
            self.wrap_walls,
            self.random_maze,
            self.custom_speed_ms,
        );
    }

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
        let new_walls: HashSet<Point> = self.walls.iter().map(|p| remap_point(*p)).collect();

        let mut seen = HashSet::new();
        for p in &new_snake {
            if !seen.insert(*p) {
                return false;
            }
        }
        if new_snake.contains(&new_food) || new_walls.contains(&new_food) {
            return false;
        }

        self.snake = new_snake;
        self.food = new_food;
        self.walls = new_walls;
        self.width = width;
        self.height = height;
        true
    }

    pub fn is_valid_for_resume(&self) -> bool {
        if self.width == 0 || self.height == 0 || self.snake.is_empty() {
            return false;
        }

        let mut seen = HashSet::new();
        for point in &self.snake {
            if point.x >= self.width || point.y >= self.height || !seen.insert(*point) {
                return false;
            }
        }

        self.food.x < self.width
            && self.food.y < self.height
            && !seen.contains(&self.food)
            && self
                .walls
                .iter()
                .all(|point| point.x < self.width && point.y < self.height && !seen.contains(point))
    }

    pub fn tick_interval_ms(&self) -> u64 {
        let base = self.custom_speed_ms.clamp(45, 260);
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
        } else if nx < 0 || ny < 0 || nx >= self.width as i16 || ny >= self.height as i16 {
            self.phase = GamePhase::GameOver;
            return true;
        }

        let next_point = Point {
            x: nx as u16,
            y: ny as u16,
        };

        let will_grow = next_point == self.food;
        if self.walls.contains(&next_point) {
            self.phase = GamePhase::GameOver;
            return true;
        }

        let body_collision = if will_grow {
            self.snake.iter().any(|p| *p == next_point)
        } else {
            self.snake
                .iter()
                .take(self.snake.len().saturating_sub(1))
                .any(|p| *p == next_point)
        };

        if body_collision {
            self.phase = GamePhase::GameOver;
            return true;
        }

        self.snake.push_front(next_point);

        if will_grow {
            self.score += 10 + self.food_eaten;
            self.food_eaten += 1;
            self.spawn_food();
        } else {
            self.snake.pop_back();
        }

        false
    }

    fn spawn_food(&mut self) {
        let snake_set: HashSet<Point> = self.snake.iter().copied().collect();
        let total = self.width as usize * self.height as usize;
        let blocked = self.snake.len() + self.walls.len();
        let mut empty = Vec::with_capacity(total.saturating_sub(blocked));
        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point { x, y };
                if !snake_set.contains(&p) && !self.walls.contains(&p) {
                    empty.push(p);
                }
            }
        }

        if let Some(food) = empty.choose(&mut thread_rng()) {
            self.food = *food;
        } else {
            self.phase = GamePhase::GameOver;
        }
    }

    fn generate_maze(&mut self) {
        self.walls.clear();
        if !self.random_maze || self.width < 14 || self.height < 10 {
            return;
        }

        let start = self.snake.front().copied().unwrap_or(Point {
            x: self.width / 2,
            y: self.height / 2,
        });

        for y in 2..self.height.saturating_sub(2) {
            for x in 2..self.width.saturating_sub(2) {
                let p = Point { x, y };
                let near_start = x.abs_diff(start.x) <= 4 && y.abs_diff(start.y) <= 3;
                let corridor = x == start.x || y == start.y;
                let sparse_wall = x % 4 == 0 && y % 3 == 0;
                if sparse_wall && !near_start && !corridor {
                    self.walls.insert(p);
                }
            }
        }
    }
}
