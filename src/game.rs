mod logic;
mod model;

#[cfg(test)]
mod tests;

pub use model::{
    Direction, Game, GamePhase, Point, MIN_BOARD_HEIGHT, MIN_BOARD_WIDTH, SAVE_VERSION,
};
