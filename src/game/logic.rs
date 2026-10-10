use std::collections::{HashSet, VecDeque};

use anyhow::{bail, Result};
use rand::{thread_rng, Rng};

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
        Self::try_with_options(
            width,
            height,
            difficulty,
            wrap_walls,
            random_maze,
            custom_speed_ms,
        )
        .expect("game board dimensions must be at least 4x4")
    }

    pub fn try_with_options(
        width: u16,
        height: u16,
        difficulty: Difficulty,
        wrap_walls: bool,
        random_maze: bool,
        custom_speed_ms: u64,
    ) -> Result<Self> {
        if width < super::MIN_BOARD_WIDTH || height < super::MIN_BOARD_HEIGHT {
            bail!(
                "game board must be at least {}x{} (got {width}x{height})",
                super::MIN_BOARD_WIDTH,
                super::MIN_BOARD_HEIGHT
            );
        }
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
            save_version: super::SAVE_VERSION,
            width,
            height,
            snake,
            direction: Direction::Right,
            direction_queue: VecDeque::new(),
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
        Ok(game)
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

    /// Logical board dimensions deliberately do not follow terminal resizes.
    /// Rendering may clip or show a too-small warning, but a resize must never
    /// rewrite a live snake, score, maze, or food location.
    pub fn resize(&mut self, width: u16, height: u16) -> bool {
        width == self.width && height == self.height
    }

    pub fn is_valid_for_resume(&self) -> bool {
        if self.save_version != super::SAVE_VERSION
            || self.width < super::MIN_BOARD_WIDTH
            || self.height < super::MIN_BOARD_HEIGHT
            || self.width > super::MAX_BOARD_WIDTH
            || self.height > super::MAX_BOARD_HEIGHT
            || self.snake.is_empty()
            || self.direction_queue.len() > 2
            || !matches!(self.phase, GamePhase::Running | GamePhase::Paused)
        {
            return false;
        }

        let cells = self.width as usize * self.height as usize;
        if self.snake.len() > cells || self.walls.len() > cells.saturating_sub(self.snake.len()) {
            return false;
        }

        let mut seen = HashSet::new();
        for point in &self.snake {
            if point.x >= self.width || point.y >= self.height || !seen.insert(*point) {
                return false;
            }
        }

        // A VecDeque may wrap internally; collect only for validation, which
        // happens on load rather than every frame.
        let segments: Vec<Point> = self.snake.iter().copied().collect();
        if !segments
            .windows(2)
            .all(|pair| self.points_are_adjacent(pair[0], pair[1]))
        {
            return false;
        }

        let mut effective = self.direction;
        for queued in &self.direction_queue {
            if *queued == effective || queued.opposite(effective) {
                return false;
            }
            effective = *queued;
        }

        self.food.x < self.width
            && self.food.y < self.height
            && !seen.contains(&self.food)
            && self.walls.iter().all(|point| {
                point.x < self.width
                    && point.y < self.height
                    && !seen.contains(point)
                    && *point != self.food
            })
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
        let effective = self
            .direction_queue
            .back()
            .copied()
            .unwrap_or(self.direction);
        if dir == effective || dir.opposite(effective) || self.direction_queue.len() >= 2 {
            return;
        }
        self.direction_queue.push_back(dir);
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

        if let Some(direction) = self.direction_queue.pop_front() {
            self.direction = direction;
        }
        self.tick_count = self.tick_count.saturating_add(1);

        let Some(head) = self.snake.front().copied() else {
            self.phase = GamePhase::GameOver;
            return true;
        };
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
            self.score = self
                .score
                .saturating_add(10u32.saturating_add(self.food_eaten));
            self.food_eaten = self.food_eaten.saturating_add(1);
            if !self.spawn_food() {
                return true;
            }
        } else {
            self.snake.pop_back();
        }

        false
    }

    /// Returns whether an empty reachable cell was available for the next
    /// food.  No such cell means the player filled every reachable cell.
    fn spawn_food(&mut self) -> bool {
        let snake_set: HashSet<Point> = self.snake.iter().copied().collect();
        let mut rng = thread_rng();
        let mut selected = None;
        let mut empty_count = 0usize;

        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point { x, y };
                if !snake_set.contains(&p) && !self.walls.contains(&p) && self.is_reachable(p) {
                    empty_count += 1;
                    if rng.gen_range(0..empty_count) == 0 {
                        selected = Some(p);
                    }
                }
            }
        }

        if let Some(food) = selected {
            self.food = food;
            true
        } else {
            self.phase = GamePhase::Won;
            false
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
                let sparse_wall = thread_rng().gen_ratio(1, 11);
                if sparse_wall && !near_start && !corridor {
                    self.walls.insert(p);
                }
            }
        }
    }

    fn is_reachable(&self, target: Point) -> bool {
        let Some(start) = self.snake.front().copied() else {
            return false;
        };
        let mut visited = HashSet::from([start]);
        let mut queue = VecDeque::from([start]);
        while let Some(point) = queue.pop_front() {
            if point == target {
                return true;
            }
            for direction in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                let (dx, dy) = direction.delta();
                let x = point.x as i16 + dx;
                let y = point.y as i16 + dy;
                if x < 0 || y < 0 || x >= self.width as i16 || y >= self.height as i16 {
                    continue;
                }
                let next = Point {
                    x: x as u16,
                    y: y as u16,
                };
                if !self.walls.contains(&next) && visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        false
    }

    fn points_are_adjacent(&self, first: Point, second: Point) -> bool {
        let direct = first.x.abs_diff(second.x) + first.y.abs_diff(second.y) == 1;
        direct
            || (self.wrap_walls
                && ((first.y == second.y
                    && ((first.x == 0 && second.x + 1 == self.width)
                        || (second.x == 0 && first.x + 1 == self.width)))
                    || (first.x == second.x
                        && ((first.y == 0 && second.y + 1 == self.height)
                            || (second.y == 0 && first.y + 1 == self.height)))))
    }
}
