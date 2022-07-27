use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use itertools::Itertools;

use super::{
    board::Board,
    score_factors::ScoreFactors,
    snake::Snake,
    SnakeID,
    ME,
};
use crate::fightsnake::{
    constants::MAX_HEALTH,
    models::GameState,
    types::{Coord, Direction},
    utils::manhattan_distance,
};

pub enum GameType {
    Solo,
    Duel,
    Triple,
    Quadruple,
    TooMany,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub snakes:     Vec<Snake>,
    pub food:       Vec<Coord>,
    pub board:      Board,
    pub multisnake: bool,
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

    pub fn game_type(&self) -> GameType {
        assert!(!self.snakes.is_empty(), "no game can have zero snakes");
        match self.snakes.len() {
            1 => GameType::Solo,
            2 => GameType::Duel,
            3 => GameType::Triple,
            4 => GameType::Quadruple,
            _ => GameType::TooMany,
        }
    }

    pub fn step(
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

            if !step.board.contains(snake.body[0]) {
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

    pub fn score(&self, snake: &Snake, depth: u64) -> ScoreFactors {
        if !self.snakes.contains(snake) {
            // we really don't want to die
            return ScoreFactors::dead(snake.id, depth);
        }

        let head = snake.body[0];

        let closest_food = self
            .food
            .iter()
            .map(|food| manhattan_distance(*food, head))
            .min()
            .unwrap_or(0);

        let closest_larger_snake = self
            .snakes
            .iter()
            .filter(|other| {
                other.id != ME && other.body.len() >= snake.body.len()
            })
            .map(|other| manhattan_distance(head, other.body[0]))
            .min()
            .unwrap_or(0);

        ScoreFactors::alive(
            snake.id,
            closest_food,
            closest_larger_snake,
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
