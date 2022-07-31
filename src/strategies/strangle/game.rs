use std::{collections::HashMap, fmt};

use color_eyre::{eyre::eyre, Report, Result};

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

pub enum Type {
    Solo,
    Duel,
    Triple,
    Quadruple,
    TooMany,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Game {
    pub snakes:      Vec<Snake>,
    pub prev_snakes: Vec<Snake>,
    pub food:        Vec<Coord>,
    pub prev_food:   Vec<Coord>,
    pub hazards:     Vec<Coord>,
    pub board:       Board,
    pub multisnake:  bool,
}

impl Game {
    pub fn new(
        snakes: Vec<Snake>,
        food: Vec<Coord>,
        hazards: Vec<Coord>,
        board: Board,
    ) -> Self {
        let multisnake = snakes.len() > 1;
        let prev_snakes = snakes.clone();
        let prev_food = food.clone();
        Self {
            snakes,
            prev_snakes,
            food,
            prev_food,
            hazards,
            board,
            multisnake,
        }
    }

    pub fn game_type(&self) -> Type {
        assert!(!self.snakes.is_empty(), "no game can have zero snakes");
        match self.snakes.len() {
            1 => Type::Solo,
            2 => Type::Duel,
            3 => Type::Triple,
            4 => Type::Quadruple,
            _ => Type::TooMany,
        }
    }

    pub fn step(&self, moves: &HashMap<SnakeID, Direction>) -> Result<Self> {
        assert!(moves.len() == self.snakes.len(), "wrong number of moves");

        let mut step = self.clone();

        // step 1 - move snakes
        for snake in &mut step.snakes {
            let direction = *moves.get(&snake.id).unwrap_or_else(|| {
                panic!("snake #{} didn't provide a move", snake.id)
            });

            snake.body.pop_back();
            snake.body.push_front(
                snake
                    .body
                    .front()
                    .ok_or(eyre!("snake without a body"))?
                    .neighbour(direction),
            );
            snake.health -= 1;
        }

        let freespace = step.calculate_free_space()?;

        // step 2 - remove eliminated battlesnakes
        step.prev_snakes.clear();
        step.prev_snakes.extend_from_slice(&step.snakes);

        step.snakes.retain(|snake| {
            if snake.health <= 0 {
                return false;
            }

            #[allow(clippy::expect_used)] // inside retain again...
            if let Some(index) = self
                .freespace_index(snake.body[0])
                .expect("invalid freespace index")
            {
                if !freespace[index] {
                    return false;
                }
            } else {
                return false;
            }

            true
        });

        // step 2a resolve head-to-head collisions
        let mut keep = vec![true; step.snakes.len()];
        for (ai, a) in step.snakes.iter().enumerate() {
            for (bi, b) in step.snakes[ai + 1..].iter().enumerate() {
                if a.body[0] == b.body[0] {
                    if b.body.len() >= a.body.len() {
                        keep[ai] = false;
                    }
                    if a.body.len() >= b.body.len() {
                        keep[bi] = false;
                    }
                }
            }
        }
        let mut kill_iter = keep.iter();

        // FIXME: is there a clean way to bubble up results from
        // retain?
        #[allow(clippy::expect_used)]
        step.snakes.retain(|_| {
            *kill_iter
                .next()
                .expect("kill_iter must be the same length as step.snakes")
        });

        // step 3 - eat food
        step.prev_food.clear();
        step.prev_food.extend_from_slice(&step.food);
        step.food.retain(|food| {
            for snake in &mut step.snakes {
                if snake.body[0] == *food {
                    snake.health = MAX_HEALTH;

                    // FIXME: is there a clean way to bubble up results from
                    // retain?
                    #[allow(clippy::expect_used)]
                    snake.body.push_back(
                        *snake.body.back().expect("snake has no body"),
                    );
                    return false;
                }
            }
            true
        });

        // step 4 - spawn new food
        // we can't predict this. we assume none will spawn, and if it does then
        // we'll adapt to it on the next real turn.

        Ok(step)
    }

    pub fn score(&self, snake: &Snake) -> ScoreFactors {
        if !self.snakes.contains(snake) {
            // we really don't want to die
            return ScoreFactors::dead(snake.id, self.multisnake);
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

        let closest_smaller_snake = self
            .snakes
            .iter()
            .filter(|other| {
                other.id != snake.id && other.body.len() < snake.body.len()
            })
            .map(|other| manhattan_distance(head, other.body[0]))
            .min()
            .unwrap_or(0);

        ScoreFactors::alive(
            snake.id,
            snake.health,
            snake.body.len() as i64,
            closest_food,
            closest_larger_snake,
            closest_smaller_snake,
            self.snakes.len() as i64 - 1,
            self.multisnake,
        )
    }

    fn freespace_index(&self, coord: Coord) -> Result<Option<usize>> {
        if self.board.contains(coord) {
            Ok(Some(usize::try_from(coord.y * self.board.width + coord.x)?))
        } else {
            Ok(None)
        }
    }

    fn calculate_free_space(&self) -> Result<Vec<bool>> {
        let mut freespace =
            vec![true; usize::try_from(self.board.width * self.board.height)?];

        for snake in &self.snakes {
            for part in snake.body.iter().skip(1) {
                if let Some(index) = self.freespace_index(*part)? {
                    freespace[index] = false;
                }
            }
        }

        for hazard in &self.hazards {
            freespace[self
                .freespace_index(*hazard)?
                .ok_or(eyre!("hazards should never be off the board!"))?] =
                false;
        }

        Ok(freespace)
    }
}

impl TryFrom<GameState> for Game {
    type Error = Report;

    fn try_from(state: GameState) -> Result<Self> {
        // sorting the snakes to put us first makes minmaxing easier.
        let you_idx = state
            .board
            .snakes
            .iter()
            .position(|snake| snake.id == state.you.id)
            .ok_or(eyre!("you don't seem to be in the game state"))?;

        let mut snakes = state.board.snakes;
        snakes.swap(ME, you_idx);

        Ok(Self::new(
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
            state.board.hazards,
            Board {
                width:  state.board.width,
                height: state.board.height,
            },
        ))
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
