use std::{
    collections::{
        hash_map::{DefaultHasher, Entry},
        HashMap,
    },
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

use color_eyre::{eyre::eyre, Result};
#[cfg(debug_assertions)]
use itertools::Itertools;

#[cfg(debug_assertions)]
use super::utils::Indent;
use super::{game::Game, score_factors::ScoreFactors, SnakeID, ME};
use crate::{
    fightsnake::types::Direction,
    strategies::strangle::score_factors::DeathKind,
};

type BigbrainScores = HashMap<SnakeID, ScoreFactors>;

pub struct BigbrainResult {
    pub scores:    BigbrainScores,
    pub direction: Option<Direction>,
    pub depth:     u64,
}

impl BigbrainResult {
    const fn inner(scores: BigbrainScores, depth: u64) -> Self {
        Self {
            scores,
            direction: None,
            depth,
        }
    }

    const fn outer(
        scores: BigbrainScores,
        direction: Direction,
        depth: u64,
    ) -> Self {
        Self {
            scores,
            direction: Some(direction),
            depth,
        }
    }
}

#[cfg(debug_assertions)]
macro_rules! trace {
    ($($tts:tt)*) => {
        println!($($tts)*);
    }
}

#[cfg(not(debug_assertions))]
macro_rules! trace {
    ($($tts:tt)*) => {};
}

fn calculate_hash(game: &Game) -> u64 {
    let mut hasher = DefaultHasher::new();
    game.hash(&mut hasher);
    hasher.finish()
}

pub struct BigbrainOptions {
    pub max_depth:  u64,
    pub time_limit: Duration,
}

fn should_exit(game: &Game, depth: u64, max_depth: u64) -> bool {
    !game.snakes.iter().any(|s| s.id == ME)
        || game.multisnake && game.snakes.len() <= 1
        || depth == max_depth
}

#[allow(clippy::too_many_lines)]
/// # Errors
///
/// Can fail if something is wrong with the input data, for example if a snake
/// has no body.
pub fn bigbrain(
    game: &Game,
    snake_index: usize,
    depth: u64,
    moves: &HashMap<SnakeID, Direction>,
    known_scores: &mut HashMap<u64, HashMap<SnakeID, ScoreFactors>>,
    start: Instant,
    options: &BigbrainOptions,
) -> Result<Option<BigbrainResult>> {
    if start.elapsed() >= options.time_limit {
        return Ok(None);
    }

    #[cfg(debug_assertions)]
    let align = Indent(depth, snake_index as u64);

    let snake = &game.snakes[snake_index];

    trace!(
        "{align}bigbrain running for snake #{} on depth {}/{} (snakes: {:?}, \
         pending moves: {:?})",
        snake.id,
        depth,
        options.max_depth,
        game.snakes.iter().map(|snake| snake.id).join(", "),
        moves
    );

    let mut game = game.clone();
    let mut moves = moves.clone();

    if snake.id == ME && depth > 0 {
        trace!("{align}we've hit a new depth");

        // remove moves for dead snakes
        moves.retain(|snake_id, _| {
            game.snakes.iter().any(|snake| snake.id == *snake_id)
        });

        let (new_game, death_kind_map) = game.step(&moves)?;

        game = new_game;
        moves.clear();

        trace!("{align}game stepped and moves cleared.");

        if should_exit(&game, depth, options.max_depth) {
            let hash = calculate_hash(&game);
            let scores = known_scores.entry(hash).or_insert({
                // score snakes still in the game
                let mut scores: HashMap<_, _> = game
                    .snakes
                    .iter()
                    .map(|snake| {
                        (snake.id, game.score(snake, DeathKind::Normal))
                    })
                    .collect();

                // add bad scores for anyone who died
                for snake in &game.prev_snakes {
                    if let Entry::Vacant(e) = scores.entry(snake.id) {
                        e.insert(ScoreFactors::dead(
                            snake.id,
                            *death_kind_map.get(&snake.id).ok_or(eyre!(
                                "snake died without a death_kind_map entry"
                            ))?,
                            game.multisnake,
                        ));
                    }
                }

                scores
            });

            trace!("{align}propagating up!");
            return Ok(Some(BigbrainResult::inner(scores.clone(), depth)));
        }
    }

    let directions = snake.possible_directions(&game.board);
    let mut best_direction = Direction::Up;

    let mut has_best_result = false;
    let mut best_result = BigbrainResult::inner(
        game.snakes
            .iter()
            .map(|snake| {
                (
                    snake.id,
                    ScoreFactors::dead(
                        snake.id,
                        DeathKind::Normal,
                        game.multisnake,
                    ),
                )
            })
            .collect(),
        depth,
    );

    let next_snake_index = (snake_index + 1) % game.snakes.len();
    let next_depth = if next_snake_index == ME {
        depth + 1
    } else {
        depth
    };

    for direction in directions {
        trace!("{align}snake {} trying {direction}", snake.id);

        moves.insert(snake.id, direction);
        let result = bigbrain(
            &game,
            next_snake_index,
            next_depth,
            &moves,
            known_scores,
            start,
            options,
        )?;

        let mut result = if let Some(result) = result {
            result
        } else {
            trace!("{align}ran out of time, aborting!");
            return Ok(None);
        };

        // ensure we always have our own score in here
        result.scores.entry(snake.id).or_insert_with(|| {
            ScoreFactors::dead(snake.id, DeathKind::Normal, game.multisnake)
        });

        trace!(
            "{align}moves {:?} on depth {depth} gets the following scores:\n{}",
            moves,
            result
                .scores
                .iter()
                .map(|(snake_id, score)| format!(
                    "{snake_id}: {}\n{score}",
                    score.calculate(result.depth)
                ))
                .join("\n"),
        );

        if has_best_result {
            let score = result
                .scores
                .get(&snake.id)
                .unwrap_or(&ScoreFactors::dead(
                    snake.id,
                    DeathKind::Normal,
                    game.multisnake,
                ))
                .calculate(result.depth);

            trace!("{align}comparing {score} against previous best...");
            if score
                > best_result.scores[&snake.id].calculate(best_result.depth)
            {
                trace!(
                    "{align}{direction} is better! setting that as best score."
                );
                best_result = result;
                best_direction = direction;
            } else {
                trace!("{align}worse...");
            }
        } else {
            trace!(
                "{align}got our first scores for this depth: {:?}",
                result
                    .scores
                    .iter()
                    .map(|(snake_id, score)| format!(
                        "snake {snake_id}: {}",
                        score.calculate(result.depth)
                    ))
                    .join(", ")
            );
            best_result = result;
            best_direction = direction;
            has_best_result = true;
        }
    }

    trace!(
        "{align}snake {}'s best move at this depth is {best_direction} with a \
         score of {}",
        snake.id,
        best_result
            .scores
            .get(&snake.id)
            .unwrap_or(&ScoreFactors::dead(
                snake.id,
                DeathKind::Normal,
                game.multisnake
            ))
            .calculate(best_result.depth)
    );

    Ok(Some(BigbrainResult::outer(
        best_result.scores,
        best_direction,
        best_result.depth,
    )))
}
