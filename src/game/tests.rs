use std::collections::VecDeque;

use crate::config::Difficulty;

use super::*;

#[test]
fn snake_grows_when_eating_food() {
    let mut game = Game::new(10, 10, Difficulty::Normal, false);
    game.phase = GamePhase::Running;
    let initial_len = game.snake.len();
    if let Some(h) = game.snake.front() {
        game.food = Point { x: h.x + 1, y: h.y };
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

#[test]
fn moving_into_tail_cell_is_allowed_when_not_eating() {
    let mut game = Game::new(8, 8, Difficulty::Normal, false);
    game.phase = GamePhase::Running;
    game.snake = VecDeque::from([
        Point { x: 3, y: 3 },
        Point { x: 3, y: 4 },
        Point { x: 2, y: 4 },
        Point { x: 2, y: 3 },
    ]);
    game.direction = Direction::Left;
    game.pending_direction = Direction::Left;
    game.food = Point { x: 7, y: 7 };

    game.tick();

    assert_eq!(game.phase, GamePhase::Running);
    assert_eq!(game.snake.front(), Some(&Point { x: 2, y: 3 }));
}

#[test]
fn random_maze_leaves_start_area_open() {
    let game = Game::with_options(30, 18, Difficulty::Normal, false, true, 130);
    let head = game.snake.front().copied().unwrap();
    assert!(!game.walls.contains(&head));
    assert!(!game.walls.contains(&game.food));
    assert!(!game.walls.is_empty());
}

#[test]
fn game_state_round_trips_for_resume() {
    let mut game = Game::with_options(30, 18, Difficulty::Hard, true, true, 90);
    game.phase = GamePhase::Paused;
    game.tick_count = 42;

    let json = serde_json::to_string(&game).unwrap();
    let restored: Game = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.phase, GamePhase::Paused);
    assert_eq!(restored.tick_count, 42);
    assert!(restored.is_valid_for_resume());
}
