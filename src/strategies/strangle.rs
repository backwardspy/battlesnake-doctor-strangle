#[cfg(not(debug_assertions))]
use std::time::Instant;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    fmt,
};

use itertools::Itertools;
#[cfg(not(debug_assertions))]
use rand::Rng;

use super::Strategy;
use crate::fightsnake::{
    models::GameState,
    types::{Coord, Direction},
    utils::manhattan_distance,
};

const MAX_HEALTH: i64 = 100;

enum GameType {
    Solo,
    Duel,
    Triple,
    Quadruple,
    TooMany,
}

#[cfg(debug_assertions)]
pub const TRACE_SIM: bool = false;
#[cfg(debug_assertions)]
pub const TRACE_BIGBRAIN: bool = true;

#[cfg(not(debug_assertions))]
pub const TRACE_SIM: bool = false;
#[cfg(not(debug_assertions))]
pub const TRACE_BIGBRAIN: bool = false;

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

#[derive(Debug)]
struct ScoreFactors {
    snake_id:              SnakeID,
    dead:                  bool,
    health:                i64,
    closest_food_distance: i64,
    remaining_opponents:   i64,
    depth:                 u64,
}

impl ScoreFactors {
    const CLOSEST_FOOD_DISTANCE_WEIGHT: i64 = -10;
    const DEPTH_WEIGHT: i64 = 1000;
    const HEALTH_WEIGHT: i64 = 100;
    const REMAINING_OPPONENTS_WEIGHT: i64 = -10000;

    fn alive(
        snake_id: SnakeID,
        health: i64,
        closest_food_distance: i64,
        remaining_opponents: i64,
        depth: u64,
    ) -> ScoreFactors {
        ScoreFactors {
            snake_id,
            dead: false,
            health,
            closest_food_distance,
            remaining_opponents,
            depth,
        }
    }

    fn dead(snake_id: SnakeID, depth: u64) -> Self {
        Self {
            snake_id,
            dead: true,
            health: 0,
            closest_food_distance: 0,
            remaining_opponents: 0,
            depth,
        }
    }

    fn calculate(&self) -> i64 {
        if self.dead {
            -1000000 + self.depth as i64 * Self::DEPTH_WEIGHT
        } else {
            self.health * Self::HEALTH_WEIGHT
                + self.closest_food_distance
                    * Self::CLOSEST_FOOD_DISTANCE_WEIGHT
                + self.remaining_opponents * Self::REMAINING_OPPONENTS_WEIGHT
                + self.depth as i64 * Self::DEPTH_WEIGHT
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
                "{} (snake {} @ {} health, {} turns to nearest food, {} \
                 remaining opponents)",
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
    id:     SnakeID,
    body:   VecDeque<Coord>,
    health: i64,
}

#[derive(Clone, Debug)]
struct Board {
    width:  i64,
    height: i64,
}

#[derive(Clone, Debug)]
struct Game {
    snakes:     Vec<Snake>,
    food:       Vec<Coord>,
    board:      Board,
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
        coord.x >= 0
            && coord.y >= 0
            && coord.x < self.width
            && coord.y < self.height
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (0..self.board.height).rev() {
            for x in 0..self.board.width {
                if self.snakes.iter().any(|snake| {
                    snake.body.iter().any(|c| c.x == x && c.y == y)
                }) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Game {
    fn get_crashed_snakes(
        snakes: Vec<Snake>,
        trace_sim: bool,
    ) -> HashSet<usize> {
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
                if trace_sim {
                    println!(
                        "{} is dying because it hit {}'s body",
                        a.id, b.id
                    );
                }
            }
            if a.body.iter().skip(1).any(|c| *c == b.body[0]) {
                kill.insert(bi);
                if trace_sim {
                    println!(
                        "{} is dying because it hit {}'s body'",
                        b.id, a.id
                    );
                }
            }

            // check head-to-head collisions
            if a.body[0] == b.body[0] {
                match (a.body.len() as i64 - b.body.len() as i64).signum() {
                    1 => {
                        kill.insert(bi);
                        if trace_sim {
                            println!(
                                "{} is dying because it hit the longer {} \
                                 head-on",
                                b.id, a.id
                            );
                        }
                    },
                    -1 => {
                        kill.insert(ai);
                        if trace_sim {
                            println!(
                                "{} is dying because it hit the longer {} \
                                 head-on",
                                a.id, b.id
                            );
                        }
                    },
                    _ => {
                        kill.insert(ai);
                        kill.insert(bi);
                        if trace_sim {
                            println!(
                                "{} and {} are dying because of a same-size \
                                 head-on collision",
                                a.id, b.id
                            );
                        }
                    },
                }
            }
        }
        kill
    }

    fn game_type(&self) -> GameType {
        assert!(!self.snakes.is_empty(), "no game can have zero snakes");
        match self.snakes.len() {
            1 => GameType::Solo,
            2 => GameType::Duel,
            3 => GameType::Triple,
            4 => GameType::Quadruple,
            _ => GameType::TooMany,
        }
    }

    fn step(
        &self,
        moves: &HashMap<SnakeID, Direction>,
        trace_sim: bool,
    ) -> Game {
        assert!(moves.len() == self.snakes.len(), "wrong number of moves");

        if trace_sim {
            println!("\nseeing what happens with the following moves:");
            for (snake_id, direction) in moves.iter() {
                println!("   snake #{} moves {}", snake_id, direction);
            }
        }

        let mut step = self.clone();

        if trace_sim {
            println!(
                "STEP 0: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        // step 1 - move snakes
        for snake in &mut step.snakes {
            let direction = *moves.get(&snake.id).unwrap_or_else(|| {
                panic!("snake #{} didn't provide a move", snake.id)
            });

            snake
                .body
                .push_front(snake.body.front().unwrap().neighbour(direction));
            snake.body.pop_back();
            snake.health -= 1;
            if trace_sim {
                println!(
                    "snake {} moving {}, down to {} hp",
                    snake.id, direction, snake.health
                );
            }
        }

        if trace_sim {
            println!(
                "STEP 1: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        // step 2 - remove eliminated battlesnakes
        step.snakes.retain(|snake| {
            if snake.health <= 0 {
                if trace_sim {
                    println!(
                        "snake {} dying from {} hp",
                        snake.id, snake.health
                    );
                }
                return false;
            }

            if !step.board.contains(&snake.body[0]) {
                if trace_sim {
                    println!(
                        "snake {} dying because it's gone out of bounds at {}",
                        snake.id, snake.body[0]
                    );
                }
                return false;
            }

            if snake.body.iter().skip(1).any(|c| *c == snake.body[0]) {
                if trace_sim {
                    println!(
                        "snake {} dying because it hit its own body at {}",
                        snake.id, snake.body[0]
                    );
                }
                return false;
            }

            true
        });

        if trace_sim {
            println!(
                "STEP 2.1: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        let crashed = Self::get_crashed_snakes(step.snakes.clone(), trace_sim);

        let mut keep = (0..step.snakes.len()).map(|i| !crashed.contains(&i));
        step.snakes.retain(|_| keep.next().unwrap());

        if trace_sim {
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
                    if trace_sim {
                        println!("snake {} eating food at {}", snake.id, food);
                    }
                    snake.health = MAX_HEALTH;
                    snake.body.push_back(*snake.body.back().unwrap());
                    return false;
                }
            }
            true
        });

        if trace_sim {
            println!(
                "STEP 3: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        // step 4 - spawn new food
        // we can't predict this. we assume none will spawn, and if it does then
        // we'll adapt to it on the next real turn.

        if trace_sim {
            println!("[ end of sim ]\n");
        }

        step
    }

    fn score(&self, snake: &Snake, depth: u64) -> ScoreFactors {
        if !self.snakes.contains(snake) {
            // we really don't want to die
            return ScoreFactors::dead(snake.id, depth);
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
            depth,
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
                width:  state.board.width,
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
    scores:    BigbrainScores,
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
    let mut best_direction = Direction::Up;

    let next_snake_index = (snake_index + 1) % game.snakes.len();
    let next_depth = if next_snake_index == ME {
        depth + 1
    } else {
        depth
    };
    for direction in possible_directions(snake.facing()) {
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

#[cfg(not(debug_assertions))]
fn make_snake(
    id: SnakeID,
    board_width: i64,
    board_height: i64,
    num_players: u64,
) -> Snake {
    let spacing = board_width / num_players as i64;
    let offset = spacing / 2;

    let xpos = offset + spacing * id as i64;

    let body: VecDeque<_> = (2..board_height - 2)
        .map(|y| Coord { x: xpos, y })
        .collect();

    Snake {
        id,
        body,
        health: 100,
    }
}

#[cfg(not(debug_assertions))]
fn benchmark_game(
    num_players: u64,
    board_width: i64,
    board_height: i64,
) -> u64 {
    const LIMIT: f64 = 250.0; // millis
    const RUNS: u64 = 15;

    let mut rng = rand::thread_rng();

    let game = Game {
        snakes:     (0..num_players)
            .map(|id| {
                make_snake(
                    id as SnakeID,
                    board_width,
                    board_height,
                    num_players,
                )
            })
            .collect(),
        food:       (5..rng.gen_range(0..10))
            .map(|_| Coord {
                x: rng.gen_range(0..board_width),
                y: rng.gen_range(0..board_height),
            })
            .collect(),
        board:      Board {
            width:  board_width,
            height: board_height,
        },
        multisnake: num_players > 1,
    };

    println!(
        "measuring performance for a {num_players} player game with {RUNS} \
         runs per depth..."
    );

    for depth in 1..=20 {
        let millis = (0..RUNS)
            .map(|_| {
                let now = Instant::now();
                bigbrain(&game, 0, 0, depth, &HashMap::new(), false, false);
                let elapsed = now.elapsed();
                elapsed.as_millis() as f64
            })
            .sum::<f64>()
            / RUNS as f64;

        if millis >= LIMIT {
            let chosen_depth = (depth - 1).max(1);
            println!(
                "reached the limit of {LIMIT} ms at depth {depth} (took \
                 {millis} ms). going with a max depth of {chosen_depth}"
            );
            return chosen_depth;
        }
    }

    println!(
        "we somehow managed all tests without timing out, so going with a max \
         depth of 20."
    );
    println!("consider testing even further..?");
    20
}

impl Strategy for StrangleStrategy {
    type State = StrangleState;

    #[cfg(debug_assertions)]
    fn get_state(&self) -> Self::State {
        Self::State {
            solo_depth:      3,
            duel_depth:      2,
            triple_depth:    1,
            quadruple_depth: 1,
            too_many_depth:  1,
        }
    }

    #[cfg(not(debug_assertions))]
    fn get_state(&self) -> Self::State {
        const BOARD_WIDTH: i64 = 11;
        const BOARD_HEIGHT: i64 = 11;
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
            TRACE_BIGBRAIN,
            TRACE_SIM,
        );

        result
            .direction
            .expect("bigbrain must return a direction from the root invocation")
    }
}
