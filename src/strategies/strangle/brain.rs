use std::collections::{hash_map::Entry, HashMap};

use itertools::Itertools;
use rand::prelude::SliceRandom;

use super::{
    game::Game,
    score_factors::ScoreFactors,
    utils::Indent,
    SnakeID,
    ME,
};
use crate::fightsnake::types::Direction;

type BigbrainScores = HashMap<SnakeID, ScoreFactors>;

pub struct BigbrainResult {
    scores:        BigbrainScores,
    pub direction: Option<Direction>,
}

impl BigbrainResult {
    fn inner(scores: BigbrainScores) -> Self {
        Self {
            scores,
            direction: None,
        }
    }

    fn outer(scores: BigbrainScores, direction: Direction) -> Self {
        Self {
            scores,
            direction: Some(direction),
        }
    }
}

pub fn bigbrain(
    game: &Game,
    snake_index: usize,
    depth: u64,
    max_depth: u64,
    moves: &HashMap<SnakeID, Direction>,
    trace: bool,
    trace_sim: bool,
) -> BigbrainResult {
    let align = Indent(depth, snake_index as u64);
    let snake = &game.snakes[snake_index];

    if trace {
        println!(
            "{align}bigbrain running for snake #{} on depth {}/{} (snakes: \
             {:?}, pending moves: {:?})",
            snake.id,
            depth,
            max_depth,
            game.snakes.iter().map(|snake| snake.id).join(", "),
            moves
        );
    }

    let mut game = game.clone();
    let mut moves = moves.clone();

    let snakes_before = game.snakes.clone();

    if snake.id == ME && depth > 0 {
        if trace {
            println!("{align}we've hit a new depth");
        }

        // remove moves for dead snakes
        moves.retain(|snake_id, _| {
            game.snakes.iter().any(|snake| snake.id == *snake_id)
        });
        assert!(
            moves.len() == game.snakes.len(),
            "wrong number of moves to simulate game"
        );

        let new_game = game.step(&moves, trace_sim);
        game = new_game;
        moves.clear();

        if trace {
            println!("{align}game stepped and moves cleared.");
        }

        // score snakes still in the game
        let mut scores: HashMap<_, _> = game
            .snakes
            .iter()
            .map(|snake| (snake.id, game.score(snake, depth)))
            .collect();

        // add bad scores for anyone who died
        for snake in snakes_before {
            if let Entry::Vacant(e) = scores.entry(snake.id) {
                e.insert(ScoreFactors::dead(snake.id, depth));
            }
        }

        let mut exit = false;

        if !game.snakes.iter().any(|s| s.id == snake.id) {
            if trace {
                println!("{align}this has killed our snake.");
            }
            exit = true;
        }

        if game.multisnake && game.snakes.len() <= 1 {
            if trace {
                println!(
                    "{align}not enough snakes to continue multisnake game."
                );
            }
            exit = true;
        }

        if depth == max_depth {
            if trace {
                println!("{align}search depth {max_depth} reached.");
            }
            exit = true;
        }

        if exit {
            if trace {
                println!("{align}propagating up!");
            }
            return BigbrainResult::inner(scores);
        }
    }

    let mut has_best_score = false;
    let mut best_scores: HashMap<_, _> = game
        .snakes
        .iter()
        .map(|snake| (snake.id, ScoreFactors::dead(snake.id, depth)))
        .collect();

    let directions = snake.possible_directions(&game.board);
    let mut best_direction = *directions
        .choose(&mut rand::thread_rng())
        .expect("no directions");

    let next_snake_index = (snake_index + 1) % game.snakes.len();
    let next_depth = if next_snake_index == ME {
        depth + 1
    } else {
        depth
    };

    for direction in directions {
        if trace {
            println!("{align}snake {} trying {direction}", snake.id);
        }

        moves.insert(snake.id, direction);
        let result = bigbrain(
            &game,
            next_snake_index,
            next_depth,
            max_depth,
            &moves,
            trace,
            trace_sim,
        );

        if trace {
            println!(
                "{align}moves {:?} on depth {depth} gets the following scores:",
                moves
            );

            for score in result.scores.values() {
                println!("{align}  * {score}");
            }
        }

        if result.scores.contains_key(&snake.id) {
            // the highest scoring direction for the current snake is propagated
            if result.scores[&snake.id].calculate()
                > best_scores[&snake.id].calculate()
                || !has_best_score
            {
                if trace {
                    println!(
                        "{align}snake {} seems to do better going {direction} \
                         than the previous best of {best_direction}",
                        snake.id
                    );
                }

                best_scores = result.scores;
                best_direction = direction;
                has_best_score = true;
            }
        } else if trace {
            println!("{align}this kills snake {}. score ignored!", snake.id)
        }
    }

    if trace {
        println!(
            "{align}snake {}'s best move at this depth is {best_direction} \
             with a score of {}",
            snake.id, best_scores[&snake.id],
        );
    }

    BigbrainResult::outer(best_scores, best_direction)
}
