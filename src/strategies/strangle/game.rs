use std::{collections::HashMap, fmt};

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
    pub prev_food:  Vec<Coord>,
    pub board:      Board,
    pub multisnake: bool,
}

impl Game {
    pub fn new(snakes: Vec<Snake>, food: Vec<Coord>, board: Board) -> Self {
        let multisnake = snakes.len() > 1;
        let prev_food = food.clone();
        Game {
            snakes,
            food,
            prev_food,
            board,
            multisnake,
        }
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

        let freespace = step.calculate_free_space();

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

            if let Some(index) = self.freespace_index(snake.body[0]) {
                if !freespace[index] {
                    if trace_sim {
                        println!(
                            "snake {} dying because it's not in freespace",
                            snake.id,
                        );
                    }
                    return false;
                }
            } else {
                if trace_sim {
                    println!(
                        "snake {} dying because it's not in freespace",
                        snake.id,
                    );
                }
                return false;
            }

            true
        });

        // step 2a resolve head-to-head collisions
        let mut keep = vec![true; step.snakes.len()];
        for (i, a) in step.snakes.iter().enumerate() {
            for b in &step.snakes[i + 1..] {
                if a.body[0] == b.body[0] {
                    if b.body.len() >= a.body.len() {
                        if trace_sim {
                            println!(
                                "snake {} dying due to head-on collision with \
                                 snake {}",
                                a.id, b.id
                            );
                        }
                        keep[i] = false;
                    }
                    if a.body.len() >= b.body.len() {
                        if trace_sim {
                            println!(
                                "snake {} dying due to head-on collision with \
                                 snake {}",
                                b.id, a.id
                            );
                        }
                        keep[i + 1] = false;
                    }
                }
            }
        }
        let mut kill_iter = keep.iter();
        step.snakes.retain(|_| *kill_iter.next().unwrap());

        if trace_sim {
            println!(
                "STEP 2.1: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        if trace_sim {
            println!(
                "STEP 2.2: {} snakes, {} food",
                step.snakes.len(),
                step.food.len()
            );
        }

        // step 3 - eat food
        step.prev_food.clear();
        step.prev_food.extend_from_slice(&step.food);
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

        // measure against prev_food, otherwise eating food removes it and thus
        // puts us far away from the nearest food.
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
                other.id != snake.id && other.body.len() >= snake.body.len()
            })
            .map(|other| manhattan_distance(head, other.body[0]))
            .min()
            .unwrap_or(0);

        ScoreFactors::alive(
            snake.id,
            snake.health,
            closest_food,
            closest_larger_snake,
            self.snakes.len() as i64 - 1,
            depth,
        )
    }

    fn freespace_index(&self, coord: Coord) -> Option<usize> {
        if !self.board.contains(coord) {
            None
        } else {
            Some((coord.y * self.board.width + coord.x) as usize)
        }
    }

    fn calculate_free_space(&self) -> Vec<bool> {
        let mut freespace =
            vec![true; (self.board.width * self.board.height) as usize];

        for snake in &self.snakes {
            for part in snake.body.iter().skip(1) {
                if let Some(index) = self.freespace_index(*part) {
                    freespace[index] = false;
                }
            }
        }

        freespace
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

        Game::new(
            snakes
                .into_iter()
                .enumerate()
                .map(|(id, snake)| Snake {
                    id,
                    body: snake.body,
                    health: snake.health,
                })
                .collect(),
            state.board.food,
            Board {
                width:  state.board.width,
                height: state.board.height,
            },
        )
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (0..self.board.height).rev() {
            for x in 0..self.board.width {
                if let Some(snake) = self.snakes.iter().find(|snake| {
                    snake.body.iter().any(|c| c.x == x && c.y == y)
                }) {
                    write!(f, "{}", snake.id)?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
