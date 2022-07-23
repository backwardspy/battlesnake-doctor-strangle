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

// turns on verbose logging of the game simulation step.
const TRACE_SIM: bool = false;

// turns on verbose logging of the bigbrain algorithm.
const TRACE_BIGBRAIN: bool = false;

// number of turns to search
// actual recursion depth will be this * the number of snakes each turn.
const SEARCH_DEPTH: u64 = 6;

pub struct StrangleStrategy;

type SnakeID = usize;

const ME: SnakeID = 0;

#[derive(Debug)]
struct ScoreFactors {
    snake_id: SnakeID,
    dead: bool,
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
            health,
            closest_food_distance,
            remaining_opponents,
        }
    }

    fn dead(snake_id: SnakeID) -> Self {
        Self {
            snake_id,
            dead: true,
            health: 0,
            closest_food_distance: 0,
            remaining_opponents: 0,
        }
    }

    fn calculate(&self) -> i64 {
        if self.dead {
            i64::MIN
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
    fn step(&self, moves: &HashMap<SnakeID, Direction>) -> Game {
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
        step.snakes.retain(|snake| {
            if snake.health <= 0 {
                if TRACE_SIM {
                    println!("snake {} dying from {} hp", snake.id, snake.health);
                }
                return false;
            }

            if !step.board.contains(&snake.body[0]) {
                if TRACE_SIM {
                    println!(
                        "snake {} dying because it's gone out of bounds at {}",
                        snake.id, snake.body[0]
                    );
                }
                return false;
            }

            if snake.body.iter().skip(1).any(|c| *c == snake.body[0]) {
                if TRACE_SIM {
                    println!(
                        "snake {} dying because it hit its own body at {}",
                        snake.id, snake.body[0]
                    );
                }
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

        fn get_crashed_snakes(snakes: Vec<Snake>) -> HashSet<usize> {
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
                }
                if a.body.iter().skip(1).any(|c| *c == b.body[0]) {
                    kill.insert(bi);
                    if TRACE_SIM {
                        println!("{} is dying because it hit {}'s body'", b.id, a.id);
                    }
                }

                // check head-to-head collisions
                if a.body[0] == b.body[0] {
                    match (a.body.len() as i64 - b.body.len() as i64).signum() {
                        1 => {
                            kill.insert(bi);
                            if TRACE_SIM {
                                println!(
                                    "{} is dying because it hit the longer {} head-on",
                                    b.id, a.id
                                );
                            }
                        }
                        -1 => {
                            kill.insert(ai);
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
            kill
        }
        let crashed = get_crashed_snakes(step.snakes.clone());
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

        step
    }

    fn score(&self, snake: &Snake) -> ScoreFactors {
        if !self.snakes.contains(snake) {
            // we really don't want to die
            return ScoreFactors::dead(snake.id);
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
    snake_id: SnakeID,
    depth: u64,
    moves: &HashMap<SnakeID, Direction>,
) -> BigbrainResult {
    let align = Indent(depth, snake_id as u64);

    if TRACE_BIGBRAIN {
        println!(
            "{align}bigbrain running for snake #{} on depth {}/{} (snakes: {:?}, pending moves: {:?})",
            snake_id,
            depth,
            SEARCH_DEPTH,
            game.snakes.iter().map(|snake| snake.id).join(", "),
            moves
        );
    }

    let snake = &game.snakes[snake_id];

    let mut game = game.clone();
    let mut moves = moves.clone();

    let snakes_before = game.snakes.clone();

    if snake_id == ME && depth > 0 {
        if TRACE_BIGBRAIN {
            println!("{align}we've hit a new depth");
        }

        // remove moves for dead snakes
        moves.retain(|snake_id, _| game.snakes.iter().any(|snake| snake.id == *snake_id));
        assert!(
            moves.len() == game.snakes.len(),
            "wrong number of moves to simulate game"
        );

        game = game.step(&moves);
        moves.clear();

        if TRACE_BIGBRAIN {
            println!("{align}game stepped and moves cleared.");
        }

        // score snakes still in the game
        let mut scores: HashMap<_, _> = game
            .snakes
            .iter()
            .map(|snake| (snake.id, game.score(snake)))
            .collect();

        // add bad scores for anyone who died
        for snake in snakes_before {
            if !scores.contains_key(&snake.id) {
                scores.insert(snake.id, ScoreFactors::dead(snake.id));
            }
        }

        let mut exit = false;

        if !game.snakes.iter().any(|snake| snake.id == snake_id) {
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

        if depth == SEARCH_DEPTH {
            if TRACE_BIGBRAIN {
                println!("{align}search depth {SEARCH_DEPTH} reached.");
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

    let mut best_scores: HashMap<_, _> = game
        .snakes
        .iter()
        .map(|snake| (snake.id, ScoreFactors::dead(snake.id)))
        .collect();
    let mut best_direction = Direction::Up;

    let next_snake_id = (snake_id + 1) % game.snakes.len();
    let next_depth = if next_snake_id == ME {
        depth + 1
    } else {
        depth
    };
    for direction in possible_directions(snake.facing()) {
        if TRACE_BIGBRAIN {
            println!("{align}snake {snake_id} trying {direction}");
        }

        moves.insert(snake_id, direction);
        let result = bigbrain(&game, next_snake_id, next_depth, &moves);

        if TRACE_BIGBRAIN {
            println!(
                "{align}moves {:?} on depth {depth} gets the following scores:",
                moves
            );

            for score in result.scores.values() {
                println!("{align}  * {score}");
            }
        }

        if result.scores.contains_key(&snake_id) {
            // the highest scoring direction for the current snake is propagated
            if result.scores[&snake_id].calculate() > best_scores[&snake_id].calculate() {
                if TRACE_BIGBRAIN {
                    println!("{align}snake {snake_id} seems to do better going {direction} than the previous best of {best_direction}");
                }

                best_scores = result.scores;
                best_direction = direction;
            }
        } else if TRACE_BIGBRAIN {
            println!("{align}this kills snake {snake_id}. score ignored!")
        }
    }

    if TRACE_BIGBRAIN {
        println!(
            "{align}snake {snake_id}'s best move at this depth is {best_direction} with a score of {}",
            best_scores[&snake_id],
        );
    }

    BigbrainResult::outer(best_scores, best_direction)
}

impl Strategy for StrangleStrategy {
    fn get_movement(&self, state: GameState) -> Direction {
        assert!(SEARCH_DEPTH > 0, "we gotta simulate at least one turn...");
        let game = Game::from(state);
        let result = bigbrain(&game, ME, 0, &HashMap::new());

        result
            .direction
            .expect("bigbrain must return a direction from the root invocation")
    }
}
