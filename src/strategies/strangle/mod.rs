mod board;
pub mod brain;
mod game;
mod score_factors;
mod snake;
mod utils;

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use color_eyre::{eyre::eyre, Result};

use self::game::Game;
use super::Strategy;
use crate::{
    fightsnake::{models::GameState, types::Direction},
    strategies::strangle::brain::{bigbrain, BigbrainOptions, BigbrainResult},
};

pub const TRACE_SIM: bool = false;

pub struct Strangle;

type SnakeID = usize;
const ME: SnakeID = 0;

impl Strategy for Strangle {
    fn get_movement(&self, game_state: GameState) -> Result<Direction> {
        const TIME_LIMIT: Duration = Duration::from_millis(400);

        let start = Instant::now();

        let game = Game::try_from(game_state)?;

        let mut depth = 1;

        let mut result = BigbrainResult {
            scores:    HashMap::new(),
            direction: None,
            depth:     0,
        };

        let mut known_scores = HashMap::new();

        while start.elapsed() < TIME_LIMIT {
            match bigbrain(
                &game,
                0,
                0,
                &HashMap::new(),
                &mut known_scores,
                start,
                &BigbrainOptions {
                    max_depth:  depth,
                    time_limit: TIME_LIMIT,
                },
            )? {
                Some(new_result) => {
                    result = new_result;
                    if result.depth < depth {
                        println!(
                            "bigbrain only got to depth {}/{}, exiting early.",
                            result.depth, depth
                        );
                        break;
                    }
                },
                None => break,
            }

            depth += 1;
        }

        println!("got a result from depth {depth}");

        result.direction.ok_or(eyre!(
            "bigbrain must return a direction from the root invocation"
        ))
    }
}
