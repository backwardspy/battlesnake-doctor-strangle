mod bench;
mod board;
mod brain;
mod game;
mod score_factors;
mod snake;
mod utils;

use std::collections::HashMap;

use self::game::Game;
use super::Strategy;
#[cfg(not(debug_assertions))]
use crate::strategies::strangle::bench::benchmark_game;
use crate::{
    fightsnake::{models::GameState, types::Direction},
    strategies::strangle::{brain::bigbrain, game::GameType},
};

pub const TRACE_SIM: bool = cfg!(debug_assertions);

pub struct StrangleState {
    solo_depth:      u64,
    duel_depth:      u64,
    triple_depth:    u64,
    quadruple_depth: u64,
    too_many_depth:  u64,
}

pub struct StrangleStrategy;

type SnakeID = usize;
const ME: SnakeID = 0;

impl Strategy for StrangleStrategy {
    type State = StrangleState;

    #[cfg(debug_assertions)]
    fn get_state(&self) -> Self::State {
        Self::State {
            solo_depth:      3,
            duel_depth:      2,
            triple_depth:    2,
            quadruple_depth: 1,
            too_many_depth:  1,
        }
    }

    #[cfg(not(debug_assertions))]
    fn get_state(&self) -> Self::State {
        const BOARD_WIDTH: i64 = 19;
        const BOARD_HEIGHT: i64 = 19;
        Self::State {
            solo_depth:      benchmark_game(1, BOARD_WIDTH, BOARD_HEIGHT)
                .min(15),
            duel_depth:      benchmark_game(2, BOARD_WIDTH, BOARD_HEIGHT)
                .min(6),
            triple_depth:    benchmark_game(3, BOARD_WIDTH, BOARD_HEIGHT)
                .min(3),
            quadruple_depth: benchmark_game(4, BOARD_WIDTH, BOARD_HEIGHT)
                .min(2),
            too_many_depth:  1,
        }
    }

    fn get_movement(
        &self,
        game_state: GameState,
        state: &mut Self::State,
    ) -> Direction {
        let game = Game::from(game_state);
        let max_depth = match game.game_type() {
            GameType::Solo => state.solo_depth,
            GameType::Duel => state.duel_depth,
            GameType::Triple => state.triple_depth,
            GameType::Quadruple => state.quadruple_depth,
            GameType::TooMany => state.too_many_depth,
        };

        println!(
            "searching {max_depth} moves ahead for {} snakes",
            game.snakes.len()
        );

        let result = bigbrain(
            &game,
            0,
            0,
            max_depth,
            &HashMap::new(),
            TRACE_SIM,
        );

        result
            .direction
            .expect("bigbrain must return a direction from the root invocation")
    }
}
