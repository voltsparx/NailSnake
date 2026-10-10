use std::collections::VecDeque;

use crate::config::Difficulty;

use super::*;

#[test]
fn rejects_invalid_dimensions() {
    assert!(Game::try_with_options(3, 4, Difficulty::Normal, false, false, 130).is_err());
    assert!(Game::try_with_options(4, 3, Difficulty::Normal, false, false, 130).is_err());
}

#[test]
fn initial_state_is_valid() {
    assert!(Game::new(10, 10, Difficulty::Normal, false).is_valid_for_resume());
}

#[test]
fn queued_turns_preserve_order_and_reject_reversal() {
    let mut game = Game::new(10, 10, Difficulty::Normal, false);
    game.set_direction(Direction::Up);
    game.set_direction(Direction::Left);
    game.set_direction(Direction::Right);
    assert_eq!(
        game.direction_queue,
        VecDeque::from([Direction::Up, Direction::Left])
    );
    game.tick();
    assert_eq!(game.direction, Direction::Up);
    game.tick();
    assert_eq!(game.direction, Direction::Left);
}

#[test]
fn wall_and_wrap_edges_are_handled() {
    let mut solid = Game::new(5, 5, Difficulty::Normal, false);
    solid.snake = VecDeque::from([Point { x: 0, y: 2 }]);
    solid.direction = Direction::Left;
    assert!(solid.tick());
    assert_eq!(solid.phase, GamePhase::GameOver);

    let mut wrap = Game::new(5, 5, Difficulty::Normal, true);
    wrap.snake = VecDeque::from([Point { x: 0, y: 2 }]);
    wrap.direction = Direction::Left;
    wrap.tick();
    assert_eq!(wrap.snake.front(), Some(&Point { x: 4, y: 2 }));
}

#[test]
fn moving_into_tail_is_allowed_unless_growing() {
    let mut game = Game::new(8, 8, Difficulty::Normal, false);
    game.snake = VecDeque::from([
        Point { x: 3, y: 3 },
        Point { x: 3, y: 4 },
        Point { x: 2, y: 4 },
        Point { x: 2, y: 3 },
    ]);
    game.direction = Direction::Left;
    game.food = Point { x: 7, y: 7 };
    game.tick();
    assert_eq!(game.phase, GamePhase::Running);

    let mut growing = Game::new(8, 8, Difficulty::Normal, false);
    growing.snake = VecDeque::from([
        Point { x: 3, y: 3 },
        Point { x: 3, y: 4 },
        Point { x: 2, y: 4 },
        Point { x: 2, y: 3 },
    ]);
    growing.direction = Direction::Left;
    growing.food = Point { x: 2, y: 3 };
    growing.tick();
    assert_eq!(growing.phase, GamePhase::GameOver);
}

#[test]
fn eating_grows_and_saturates_score() {
    let mut game = Game::new(10, 10, Difficulty::Normal, false);
    let head = game.snake.front().copied().unwrap();
    game.food = Point {
        x: head.x + 1,
        y: head.y,
    };
    game.score = u32::MAX - 1;
    game.food_eaten = u32::MAX;
    game.tick();
    assert_eq!(game.score, u32::MAX);
    assert_eq!(game.food_eaten, u32::MAX);
}

#[test]
fn speed_is_clamped_and_has_floor() {
    let mut game = Game::with_options(10, 10, Difficulty::Normal, false, false, 1);
    assert_eq!(game.tick_interval_ms(), 45);
    game.custom_speed_ms = 9_999;
    assert_eq!(game.tick_interval_ms(), 260);
    game.custom_speed_ms = 45;
    game.food_eaten = 100;
    assert_eq!(game.tick_interval_ms(), 40);
}

#[test]
fn maze_food_is_reachable_and_valid() {
    for _ in 0..10 {
        let game = Game::with_options(30, 18, Difficulty::Normal, false, true, 130);
        assert!(game.is_valid_for_resume());
        assert!(!game.walls.contains(&game.food));
    }
}

#[test]
fn resize_is_a_non_destructive_viewport_operation() {
    let mut game = Game::new(12, 10, Difficulty::Normal, false);
    game.score = 42;
    let snake = game.snake.clone();
    assert!(!game.resize(20, 20));
    assert_eq!(game.score, 42);
    assert_eq!(game.snake, snake);
}

#[test]
fn resume_rejects_wrong_version_and_round_trips_current_version() {
    let game = Game::with_options(30, 18, Difficulty::Hard, true, true, 90);
    let json = serde_json::to_string(&game).unwrap();
    let restored: Game = serde_json::from_str(&json).unwrap();
    assert!(restored.is_valid_for_resume());
    let mut invalid = restored;
    invalid.save_version += 1;
    assert!(!invalid.is_valid_for_resume());
}

#[test]
fn filling_the_board_reports_a_win() {
    let mut game = Game::new(4, 4, Difficulty::Normal, false);
    game.snake = VecDeque::from([
        Point { x: 2, y: 3 },
        Point { x: 1, y: 3 },
        Point { x: 0, y: 3 },
        Point { x: 0, y: 2 },
        Point { x: 1, y: 2 },
        Point { x: 2, y: 2 },
        Point { x: 3, y: 2 },
        Point { x: 3, y: 1 },
        Point { x: 2, y: 1 },
        Point { x: 1, y: 1 },
        Point { x: 0, y: 1 },
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: 2, y: 0 },
        Point { x: 3, y: 0 },
    ]);
    game.direction = Direction::Right;
    game.food = Point { x: 3, y: 3 };

    assert!(game.tick());
    assert_eq!(game.phase, GamePhase::Won);
    assert_eq!(game.snake.len(), 16);
}

#[test]
fn resume_rejects_oversized_board_and_terminal_phases() {
    let mut game = Game::new(10, 10, Difficulty::Normal, false);
    game.width = super::MAX_BOARD_WIDTH + 1;
    assert!(!game.is_valid_for_resume());

    game = Game::new(10, 10, Difficulty::Normal, false);
    game.phase = GamePhase::Won;
    assert!(!game.is_valid_for_resume());
}
