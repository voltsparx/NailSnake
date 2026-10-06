use std::collections::VecDeque;

use nailsnake::config::Difficulty;
use nailsnake::{Direction, Game, GamePhase, Point};

#[test]
fn scripted_gameplay_never_breaks_state_invariants() {
    let mut game = Game::new(16, 12, Difficulty::Normal, true);
    for direction in [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
    ]
    .into_iter()
    .cycle()
    .take(80)
    {
        if game.phase != GamePhase::Running {
            break;
        }
        game.set_direction(direction);
        game.tick();
        assert!(game.is_valid_for_resume() || game.phase == GamePhase::GameOver);
    }
}

#[test]
fn wrap_mode_allows_edge_transition() {
    let mut game = Game::new(10, 10, Difficulty::Normal, true);
    game.snake = VecDeque::from([
        Point { x: 0, y: 5 },
        Point { x: 1, y: 5 },
        Point { x: 2, y: 5 },
    ]);
    game.direction = Direction::Left;
    game.tick();
    assert_eq!(game.phase, GamePhase::Running);
    assert_eq!(game.snake.front(), Some(&Point { x: 9, y: 5 }));
}

#[test]
fn saved_game_state_serializes_with_queue() {
    let mut game = Game::new(12, 12, Difficulty::Hard, false);
    game.set_direction(Direction::Up);
    game.set_direction(Direction::Left);
    game.phase = GamePhase::Paused;
    let restored: Game = serde_json::from_str(&serde_json::to_string(&game).unwrap()).unwrap();
    assert!(restored.is_valid_for_resume());
    assert_eq!(restored.direction_queue.len(), 2);
}
