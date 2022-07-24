use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
};

use crate::fightsnake::{
    models::GameState,
    types::{Coord, Direction},
    utils::manhattan_distance,
};

use itertools::Itertools;

use super::Strategy;

const MAX_HEALTH: i64 = 100;

enum GameType {
    Solo,
    Duel,
    Triple,
    Quadruple,
    TooMany,
}

fn get_search_depth(game_type: GameType) -> u64 {
    #[cfg(debug_assertions)]
    match game_type {
        _ => 1,
    }

    #[cfg(not(debug_assertions))]
    match game_type {
        GameType::Solo => 14,
        GameType::Duel => 6,
        GameType::Triple => 3,
        GameType::Quadruple => 1,
        GameType::TooMany => 1,
    }
}

#[cfg(debug_assertions)]
pub const TRACE_SIM: bool = false;
#[cfg(debug_assertions)]
pub const TRACE_BIGBRAIN: bool = true;

#[cfg(not(debug_assertions))]
pub const TRACE_SIM: bool = false;
#[cfg(not(debug_assertions))]
pub const TRACE_BIGBRAIN: bool = false;

pub struct StrangleStrategy;

type SnakeID = usize;

const ME: SnakeID = 0;

#[derive(Debug)]
struct ScoreFactors {
    snake_id: SnakeID,
    dead: bool,
    death_kind: DeathKind,
    health: i64,
    closest_food_distance: i64,
    remaining_opponents: i64,
}

impl ScoreFactors {
    const HEALTH_WEIGHT: i64 = 100;
    const CLOSEST_FOOD_DISTANCE_WEIGHT: i64 = -10;
    const REMAINING_OPPONENTS_WEIGHT: i64 = -1000;

    fn alive(
        snake_id: SnakeID,
        health: i64,
        closest_food_distance: i64,
        remaining_opponents: i64,
    ) -> ScoreFactors {
        ScoreFactors {
            snake_id,
            dead: false,
            death_kind: DeathKind::Normal,
            health,
            closest_food_distance,
            remaining_opponents,
        }
    }

    fn dead(snake_id: SnakeID, death_kind: DeathKind) -> Self {
        Self {
            snake_id,
            dead: true,
            death_kind,
            health: 0,
            closest_food_distance: 0,
            remaining_opponents: 0,
        }
    }

    fn calculate(&self) -> i64 {
        if self.dead {
            match self.death_kind {
                DeathKind::Normal => -1000000,
                DeathKind::HeadToHead => -100000,
            }
        } else {
            self.health * Self::HEALTH_WEIGHT
                + self.closest_food_distance * Self::CLOSEST_FOOD_DISTANCE_WEIGHT
                + self.remaining_opponents * Self::REMAINING_OPPONENTS_WEIGHT
        }
    }
}

impl fmt::Display for ScoreFactors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.dead {
            write!(
                f,
                "{} (snake {} is freakin dead dude)",
                self.calculate(),
                self.snake_id
            )
        } else {
            write!(
                f,
                "{} (snake {} @ {} health, {} turns to nearest food, {} remaining opponents)",
                self.calculate(),
                self.snake_id,
                self.health,
                self.closest_food_distance,
                self.remaining_opponents
            )
        }
    }
}

#[derive(Clone, Debug)]
struct Snake {
    id: SnakeID,
    body: VecDeque<Coord>,
    health: i64,
}

#[derive(Clone, Debug)]
struct Board {
    width: i64,
    height: i64,
}

#[derive(Clone, Debug)]
struct Game {
    snakes: Vec<Snake>,
    food: Vec<Coord>,
    board: Board,
    multisnake: bool,
}

#[derive(Debug, Clone, Copy)]
enum DeathKind {
    Normal,
    HeadToHead,
}

impl Snake {
    pub fn facing(&self) -> Option<Direction> {
        Direction::between(&self.body[1], &self.body[0])
    }
}

impl PartialEq for Snake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Board {
    fn contains(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.width && coord.y < self.height
    }
}

impl Game {
    fn game_type(&self) -> GameType {
        assert!(self.snakes.len() > 0, "no game can have zero snakes");
        match self.snakes.len() {
            1 => GameType::Solo,
            2 => GameType::Duel,
            3 => GameType::Triple,
            4 => GameType::Quadruple,
            _ => GameType::TooMany,
        }
    }

    fn step(&self, moves: &HashMap<SnakeID, Direction>) -> (Game, HashMap<SnakeID, DeathKind>) {
        assert!(moves.len() == self.snakes.len(), "wrong number of moves");

        if TRACE_SIM {
            println!("\nseeing what happens with the following moves:");
            for (snake_id, direction) in moves.iter() {
                println!("   snake #{} moves {}", snake_id, direction);
            }
        }

        let mut step = self.clone();

        if TRACE_SIM {
            println!(
                "STEP 0: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        // step 1 - move snakes
        for snake in &mut step.snakes {
            let direction = *moves
                .get(&snake.id)
                .expect(&format!("snake #{} didn't provide a move", snake.id));

            snake
                .body
                .push_front(snake.body.front().unwrap().neighbour(direction));
            snake.body.pop_back();
            snake.health -= 1;
            if TRACE_SIM {
                println!(
                    "snake {} moving {}, down to {} hp",
                    snake.id, direction, snake.health
                );
            }
        }

        if TRACE_SIM {
            println!(
                "STEP 1: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        // step 2 - remove eliminated battlesnakes
        let mut death_kind_map = HashMap::new();

        step.snakes.retain(|snake| {
            if snake.health <= 0 {
                if TRACE_SIM {
                    println!("snake {} dying from {} hp", snake.id, snake.health);
                }
                death_kind_map.insert(snake.id, DeathKind::Normal);
                return false;
            }

            if !step.board.contains(&snake.body[0]) {
                if TRACE_SIM {
                    println!(
                        "snake {} dying because it's gone out of bounds at {}",
                        snake.id, snake.body[0]
                    );
                }
                death_kind_map.insert(snake.id, DeathKind::Normal);
                return false;
            }

            if snake.body.iter().skip(1).any(|c| *c == snake.body[0]) {
                if TRACE_SIM {
                    println!(
                        "snake {} dying because it hit its own body at {}",
                        snake.id, snake.body[0]
                    );
                }
                death_kind_map.insert(snake.id, DeathKind::Normal);
                return false;
            }

            true
        });

        if TRACE_SIM {
            println!(
                "STEP 2.1: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        fn get_crashed_snakes(snakes: Vec<Snake>) -> (HashSet<usize>, HashMap<SnakeID, DeathKind>) {
            let mut death_kind_map = HashMap::new();
            let mut kill = HashSet::new();
            for ((ai, a), (bi, b)) in snakes
                .iter()
                .enumerate()
                .combinations(2)
                .map(|v| (v[0], v[1]))
            {
                // check head-to-body collisions
                if b.body.iter().skip(1).any(|c| *c == a.body[0]) {
                    kill.insert(ai);
                    if TRACE_SIM {
                        println!("{} is dying because it hit {}'s body", a.id, b.id);
                    }
                    death_kind_map.insert(a.id, DeathKind::Normal);
                }
                if a.body.iter().skip(1).any(|c| *c == b.body[0]) {
                    kill.insert(bi);
                    if TRACE_SIM {
                        println!("{} is dying because it hit {}'s body'", b.id, a.id);
                    }
                    death_kind_map.insert(b.id, DeathKind::Normal);
                }

                // check head-to-head collisions
                if a.body[0] == b.body[0] {
                    match (a.body.len() as i64 - b.body.len() as i64).signum() {
                        1 => {
                            kill.insert(bi);
                            death_kind_map.insert(b.id, DeathKind::HeadToHead);
                            if TRACE_SIM {
                                println!(
                                    "{} is dying because it hit the longer {} head-on",
                                    b.id, a.id
                                );
                            }
                        }
                        -1 => {
                            kill.insert(ai);
                            death_kind_map.insert(a.id, DeathKind::HeadToHead);
                            if TRACE_SIM {
                                println!(
                                    "{} is dying because it hit the longer {} head-on",
                                    a.id, b.id
                                );
                            }
                        }
                        _ => {
                            kill.insert(ai);
                            kill.insert(bi);
                            death_kind_map.insert(a.id, DeathKind::HeadToHead);
                            death_kind_map.insert(b.id, DeathKind::HeadToHead);
                            if TRACE_SIM {
                                println!(
                                    "{} and {} are dying because of a same-size head-on collision",
                                    a.id, b.id
                                );
                            }
                        }
                    }
                }
            }
            (kill, death_kind_map)
        }
        let (crashed, crash_deaths) = get_crashed_snakes(step.snakes.clone());
        death_kind_map.extend(crash_deaths.into_iter());

        let mut keep = (0..step.snakes.len()).map(|i| !crashed.contains(&i));
        step.snakes.retain(|_| keep.next().unwrap());

        if TRACE_SIM {
            println!(
                "STEP 2.2: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        // step 3 - eat food
        step.food.retain(|food| {
            for snake in &mut step.snakes {
                if snake.body[0] == *food {
                    if TRACE_SIM {
                        println!("snake {} eating food at {}", snake.id, food);
                    }
                    snake.health = MAX_HEALTH;
                    snake.body.push_back(*snake.body.back().unwrap());
                    return false;
                }
            }
            true
        });

        if TRACE_SIM {
            println!(
                "STEP 3: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        // step 4 - spawn new food
        // we can't predict this. we assume none will spawn, and if it does then we'll adapt to it
        // on the next real turn.

        if TRACE_SIM {
            println!("[ end of sim ]\n");
        }

        (step, death_kind_map)
    }

    fn score(&self, snake: &Snake, death_kind_map: &HashMap<SnakeID, DeathKind>) -> ScoreFactors {
        if !self.snakes.contains(snake) {
            // we really don't want to die
            return ScoreFactors::dead(snake.id, death_kind_map[&snake.id]);
        }

        let closest_food_distance = self
            .food
            .iter()
            .map(|food| manhattan_distance(food, &snake.body[0]))
            .min()
            .unwrap_or(0);

        ScoreFactors::alive(
            snake.id,
            snake.health,
            closest_food_distance,
            self.snakes.len() as i64 - 1,
        )
    }
}

impl From<GameState> for Game {
    fn from(state: GameState) -> Self {
        // sorting the snakes to put us first makes minmaxing easier.
        let you_idx = state
            .board
            .snakes
            .iter()
            .position(|snake| snake.id == state.you.id)
            .expect("you don't seem to be in the game state");

        let mut snakes = state.board.snakes;
        snakes.swap(ME, you_idx);

        let multisnake = snakes.len() > 1;

        Game {
            snakes: snakes
                .into_iter()
                .enumerate()
                .map(|(id, snake)| Snake {
                    id,
                    body: snake.body,
                    health: snake.health,
                })
                .collect(),
            food: state.board.food,
            board: Board {
                width: state.board.width,
                height: state.board.height,
            },
            multisnake,
        }
    }
}

fn possible_directions(facing: Option<Direction>) -> Vec<Direction> {
    match facing {
        Some(next_facing) => Direction::iter()
            .copied()
            .filter(|d| *d != next_facing.opposite())
            .collect(),
        None => Direction::iter().copied().collect(),
    }
}

type BigbrainScores = HashMap<SnakeID, ScoreFactors>;

struct BigbrainResult {
    scores: BigbrainScores,
    direction: Option<Direction>,
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

struct Indent(u64, u64);
impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.0 * 4 + self.1 {
            write!(f, "█")?;
        }
        write!(f, "▶ ")
    }
}

fn bigbrain(
    game: &Game,
    snake_index: usize,
    depth: u64,
    max_depth: u64,
    moves: &HashMap<SnakeID, Direction>,
) -> BigbrainResult {
    let align = Indent(depth, snake_index as u64);
    let snake = &game.snakes[snake_index];

    if TRACE_BIGBRAIN {
        println!(
            "{align}bigbrain running for snake #{} on depth {}/{} (snakes: {:?}, pending moves: {:?})",
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
        if TRACE_BIGBRAIN {
            println!("{align}we've hit a new depth");
        }

        // remove moves for dead snakes
        moves.retain(|snake_id, _| game.snakes.iter().any(|snake| snake.id == *snake_id));
        assert!(
            moves.len() == game.snakes.len(),
            "wrong number of moves to simulate game"
        );

        let (new_game, death_kind_map) = game.step(&moves);
        game = new_game;
        moves.clear();

        if TRACE_BIGBRAIN {
            println!("{align}game stepped and moves cleared.");
        }

        // score snakes still in the game
        let mut scores: HashMap<_, _> = game
            .snakes
            .iter()
            .map(|snake| (snake.id, game.score(snake, &death_kind_map)))
            .collect();

        // add bad scores for anyone who died
        for snake in snakes_before {
            if !scores.contains_key(&snake.id) {
                scores.insert(
                    snake.id,
                    ScoreFactors::dead(snake.id, death_kind_map[&snake.id]),
                );
            }
        }

        let mut exit = false;

        if !game.snakes.iter().any(|s| s.id == snake.id) {
            if TRACE_BIGBRAIN {
                println!("{align}this has killed our snake.");
            }
            exit = true;
        }

        if game.multisnake && game.snakes.len() <= 1 {
            if TRACE_BIGBRAIN {
                println!("{align}not enough snakes to continue multisnake game.");
            }
            exit = true;
        }

        if depth == max_depth {
            if TRACE_BIGBRAIN {
                println!("{align}search depth {max_depth} reached.");
            }
            exit = true;
        }

        if exit {
            if TRACE_BIGBRAIN {
                println!("{align}propagating up!");
            }
            return BigbrainResult::inner(scores);
        }
    }

    let mut has_best_score = false;
    let mut best_scores: HashMap<_, _> = game
        .snakes
        .iter()
        .map(|snake| (snake.id, ScoreFactors::dead(snake.id, DeathKind::Normal)))
        .collect();
    let mut best_direction = Direction::Up;

    let next_snake_index = (snake_index + 1) % game.snakes.len();
    let next_depth = if next_snake_index == ME {
        depth + 1
    } else {
        depth
    };
    for direction in possible_directions(snake.facing()) {
        if TRACE_BIGBRAIN {
            println!("{align}snake {} trying {direction}", snake.id);
        }

        moves.insert(snake.id, direction);
        let result = bigbrain(&game, next_snake_index, next_depth, max_depth, &moves);

        if TRACE_BIGBRAIN {
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
            if result.scores[&snake.id].calculate() > best_scores[&snake.id].calculate()
                || !has_best_score
            {
                if TRACE_BIGBRAIN {
                    println!("{align}snake {} seems to do better going {direction} than the previous best of {best_direction}", snake.id);
                }

                best_scores = result.scores;
                best_direction = direction;
                has_best_score = true;
            }
        } else if TRACE_BIGBRAIN {
            println!("{align}this kills snake {}. score ignored!", snake.id)
        }
    }

    if TRACE_BIGBRAIN {
        println!(
            "{align}snake {}'s best move at this depth is {best_direction} with a score of {}",
            snake.id, best_scores[&snake.id],
        );
    }

    BigbrainResult::outer(best_scores, best_direction)
}

impl Strategy for StrangleStrategy {
    fn get_movement(&self, state: GameState) -> Direction {
        let game = Game::from(state);
        let max_depth = get_search_depth(game.game_type());
        println!(
            "searching {max_depth} moves ahead for {} snakes",
            game.snakes.len()
        );

        let result = bigbrain(&game, 0, 0, max_depth, &HashMap::new());

        result
            .direction
            .expect("bigbrain must return a direction from the root invocation")
    }
}
