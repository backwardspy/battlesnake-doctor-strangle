pub mod strangle;

pub use strangle::StrangleStrategy;

use crate::fightsnake::{models::GameState, types::Direction};

pub trait Strategy {
    type State;

    fn get_state(&self) -> Self::State;
    fn get_movement(
        &self,
        game_state: GameState,
        state: &mut Self::State,
    ) -> Direction;
}
