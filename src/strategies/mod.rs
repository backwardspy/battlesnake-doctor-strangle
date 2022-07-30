pub mod strangle;

use color_eyre::Result;
pub use strangle::Strangle;

use crate::fightsnake::{models::GameState, types::Direction};

pub trait Strategy {
    /// # Errors
    ///
    /// Can fail for a wide range of reasons usually due to invalid game states.
    fn get_movement(&self, game_state: GameState) -> Result<Direction>;
}
