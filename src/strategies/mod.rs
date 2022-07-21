mod strangle;

pub use strangle::StrangleStrategy;

use crate::fightsnake::{models::GameState, types::Direction};

pub trait Strategy {
    fn get_movement(&self, state: &GameState) -> Direction;
}
