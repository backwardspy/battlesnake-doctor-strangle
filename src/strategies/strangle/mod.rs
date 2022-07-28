pub mod bench;
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

use self::game::Game;
use super::Strategy;
use crate::{
    fightsnake::{models::GameState, types::Direction},
    strategies::strangle::brain::{bigbrain, BigbrainOptions, BigbrainResult},
};

pub const TRACE_SIM: bool = false;

pub struct StrangleStrategy;

type SnakeID = usize;
const ME: SnakeID = 0;

impl Strategy for StrangleStrategy {
    fn get_movement(&self, game_state: GameState) -> Direction {
        let start = Instant::now();

        const TIME_LIMIT: Duration = Duration::from_millis(400);

        let game = Game::from(game_state);

        let mut depth = 1;

        let mut result = BigbrainResult {
            scores:    HashMap::new(),
            direction: None,
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
                    trace_sim:  TRACE_SIM,
                },
            ) {
                Some(new_result) => result = new_result,
                None => break,
            }

            depth += 1;
            println!(
                "{} ms elapsed of limit {}, going to depth {depth}...",
                start.elapsed().as_millis(),
                TIME_LIMIT.as_millis(),
            );
        }

        println!("got a result from depth {depth}");

        result
            .direction
            .expect("bigbrain must return a direction from the root invocation")
    }
}
