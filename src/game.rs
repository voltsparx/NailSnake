mod logic;
mod model;

#[cfg(test)]
mod tests;

pub use model::{
    Direction, Game, GamePhase, Point, MAX_BOARD_HEIGHT, MAX_BOARD_WIDTH, MIN_BOARD_HEIGHT,
    MIN_BOARD_WIDTH, SAVE_VERSION,
};
