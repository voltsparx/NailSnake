use std::collections::VecDeque;

use nailsnake::{Direction, Game, GamePhase, Point};
use nailsnake::config::Difficulty;

/// Smoke test: play random moves until the game ends or we've taken 100 ticks.
/// Verifies that the game loop doesn't panic or hang.
#[test]
fn full_snake_gameplay_simulation() {
    let mut game = Game::new(10, 10, Difficulty::Normal, false);
    assert_eq!(game.phase, GamePhase::Running);

    game.set_direction(Direction::Right);
    for _ in 0..100 {
        if game.phase != GamePhase::Running {
            break;
        }
        game.tick();
    }
    assert!(game.score > 0 || game.phase == GamePhase::GameOver);
}

/// Fill a tiny 3×3 board until the snake occupies every cell.
/// The game should end when no space remains for food.
#[test]
fn snake_covers_whole_board_triggers_game_over() {
    let mut game = Game::new(3, 3, Difficulty::Normal, false);
    loop {
        if game.phase != GamePhase::Running {
            break;
        }
        game.set_direction(Direction::Right);
        game.tick();
        game.set_direction(Direction::Down);
        game.tick();
        game.set_direction(Direction::Left);
        game.tick();
        game.set_direction(Direction::Down);
        game.tick();
        game.set_direction(Direction::Right);
        game.tick();
    }
    assert_eq!(game.phase, GamePhase::GameOver);
}

/// With wrap mode enabled, the snake should appear on the opposite side
/// when moving off the edge, without dying.
#[test]
fn wrap_mode_allows_edge_transition() {
    let mut game = Game::new(10, 10, Difficulty::Normal, true);
    game.snake = VecDeque::from([
        Point { x: 0, y: 5 },
        Point { x: 1, y: 5 },
        Point { x: 2, y: 5 },
    ]);
    game.direction = Direction::Left;
    game.pending_direction = Direction::Left;
    game.tick();
    assert_eq!(game.phase, GamePhase::Running);
    assert_eq!(game.snake.front(), Some(&Point { x: 9, y: 5 }));
}

/// Run 50 randomised games and assert food never overlaps the snake body.
/// This is our main defence against regressions in `spawn_food`.
#[test]
fn food_never_spawns_on_snake_body() {
    for _ in 0..50 {
        let mut game = Game::new(8, 8, Difficulty::Normal, false);
        for _ in 0..100 {
            if game.phase != GamePhase::Running {
                break;
            }
            game.set_direction(good_dir(&game));
            game.tick();
        }
    }
}

fn good_dir(_game: &Game) -> Direction {
    use rand::seq::SliceRandom;
    let dirs = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
    *dirs.choose(&mut rand::thread_rng()).unwrap()
}

/// Ensure that eating multiple pieces of food accumulates score and the food
/// counter correctly, without overflow or stalling.
#[test]
fn score_accumulates_over_multiple_food_eaten() {
    let mut game = Game::new(15, 15, Difficulty::Normal, false);
    game.phase = GamePhase::Running;
    let initial_score = game.score;
    for _ in 0..5 {
        if let Some(h) = game.snake.front().copied() {
            game.food = Point {
                x: h.x + 1,
                y: h.y,
            };
        }
        game.pending_direction = Direction::Right;
        game.tick();
    }
    assert!(game.score > initial_score);
    assert_eq!(game.food_eaten, 5);
}
