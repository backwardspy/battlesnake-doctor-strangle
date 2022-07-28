use std::{
    collections::{
        hash_map::{DefaultHasher, Entry},
        HashMap,
    },
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

#[cfg(debug_assertions)]
use itertools::Itertools;

#[cfg(debug_assertions)]
use super::utils::Indent;
use super::{game::Game, score_factors::ScoreFactors, SnakeID, ME};
use crate::fightsnake::types::Direction;

type BigbrainScores = HashMap<SnakeID, ScoreFactors>;

pub struct BigbrainResult {
    pub scores:    BigbrainScores,
    pub direction: Option<Direction>,
    pub depth:     u64,
}

impl BigbrainResult {
    fn inner(scores: BigbrainScores, depth: u64) -> Self {
        Self {
            scores,
            direction: None,
            depth,
        }
    }

    fn outer(scores: BigbrainScores, direction: Direction, depth: u64) -> Self {
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
    pub trace_sim:  bool,
}

pub fn bigbrain(
    game: &Game,
    snake_index: usize,
    depth: u64,
    moves: &HashMap<SnakeID, Direction>,
    known_scores: &mut HashMap<u64, HashMap<SnakeID, ScoreFactors>>,
    start: Instant,
    options: &BigbrainOptions,
) -> Option<BigbrainResult> {
    if start.elapsed() >= options.time_limit {
        return None;
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

    let snakes_before = game.snakes.clone();

    if snake.id == ME && depth > 0 {
        trace!("{align}we've hit a new depth");

        // remove moves for dead snakes
        moves.retain(|snake_id, _| {
            game.snakes.iter().any(|snake| snake.id == *snake_id)
        });
        assert!(
            moves.len() == game.snakes.len(),
            "wrong number of moves to simulate game"
        );

        let new_game = game.step(&moves, options.trace_sim);

        game = new_game;
        moves.clear();

        trace!("{align}game stepped and moves cleared.");

        let hash = calculate_hash(&game);

        let scores = known_scores.entry(hash).or_insert({
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

            scores
        });

        let mut exit = false;

        if !game.snakes.iter().any(|s| s.id == ME) {
            trace!("{align}this has killed our snake.");
            exit = true;
        }

        if game.multisnake && game.snakes.len() <= 1 {
            trace!("{align}not enough snakes to continue multisnake game.");
            exit = true;
        }

        if depth == options.max_depth {
            trace!("{align}search depth {} reached.", options.max_depth);
            exit = true;
        }

        if exit {
            trace!("{align}propagating up!");
            return Some(BigbrainResult::inner(scores.clone(), depth));
        }
    }

    let directions = snake.possible_directions(&game.board);
    let mut best_direction = Direction::Up;

    let mut has_best_result = false;
    let mut best_result = BigbrainResult::inner(
        game.snakes
            .iter()
            .map(|snake| (snake.id, ScoreFactors::dead(snake.id, depth)))
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
        );

        if result.is_none() {
            trace!("{align}ran out of time, aborting!");
            return None;
        }

        let mut result = result.unwrap();

        // ensure we always have our own score in here
        result
            .scores
            .entry(snake.id)
            .or_insert_with(|| ScoreFactors::dead(snake.id, depth));

        trace!(
            "{align}moves {:?} on depth {depth} gets the following scores:",
            moves
        );

        #[cfg(debug_assertions)]
        for score in result.scores.values() {
            trace!("{align}  * {score}");
        }

        if !has_best_result {
            trace!(
                "{align}got our first scores for this depth: {:?}",
                result
                    .scores
                    .iter()
                    .map(|(snake_id, score)| format!(
                        "snake {snake_id}: {}",
                        score.calculate()
                    ))
                    .join(", ")
            );
            best_result = result;
            best_direction = direction;
            has_best_result = true;
        } else {
            let score = result
                .scores
                .get(&snake.id)
                .unwrap_or(&ScoreFactors::dead(snake.id, depth))
                .calculate();

            trace!("{align}comparing {score} against previous best...");
            if score > best_result.scores[&snake.id].calculate() {
                trace!(
                    "{align}{direction} is better! setting that as best score."
                );
                best_result = result;
                best_direction = direction;
            } else {
                trace!("{align}worse...");
            }
        }
    }

    trace!(
        "{align}snake {}'s best move at this depth is {best_direction} with a \
         score of {}",
        snake.id,
        best_result
            .scores
            .get(&snake.id)
            .unwrap_or(&ScoreFactors::dead(snake.id, depth))
    );

    Some(BigbrainResult::outer(
        best_result.scores,
        best_direction,
        best_result.depth,
    ))
}
