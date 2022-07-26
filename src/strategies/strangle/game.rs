use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

use color_eyre::{eyre::eyre, Report, Result};

use super::{
    board::Board,
    score_factors::ScoreFactors,
    snake::Snake,
    SnakeID,
    ME,
};
use crate::{
    fightsnake::{
        constants::MAX_HEALTH,
        models::GameState,
        types::{Coord, Direction},
        utils::manhattan_distance,
    },
    strategies::strangle::score_factors::DeathKind,
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

    pub fn step(
        &self,
        moves: &HashMap<SnakeID, Direction>,
    ) -> Result<(Self, Vec<bool>, HashMap<SnakeID, DeathKind>)> {
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
        let mut death_kind_map = HashMap::new();

        step.prev_snakes.clear();
        step.prev_snakes.extend_from_slice(&step.snakes);

        step.snakes.retain(|snake| {
            if snake.health <= 0 {
                death_kind_map.insert(snake.id, DeathKind::Normal);
                return false;
            }

            #[allow(clippy::expect_used)] // inside retain again...
            if let Some(index) = self
                .freespace_index(snake.body[0])
                .expect("invalid freespace index")
            {
                if !freespace[index] {
                    death_kind_map.insert(snake.id, DeathKind::Normal);
                    return false;
                }
            } else {
                death_kind_map.insert(snake.id, DeathKind::Normal);
                return false;
            }

            true
        });

        // step 2a resolve head-to-head collisions
        let mut keep = vec![true; step.snakes.len()];
        for (ai, a) in step.snakes.iter().enumerate() {
            for (bi, b) in step.snakes.iter().enumerate().skip(ai + 1) {
                if a.body[0] == b.body[0] {
                    // horrible hack alert!
                    // we pretend that only we can die by same-size head-to-head
                    // collisions. if we don't do this, the
                    // snake assumes that nobody else would ever go for a
                    // same-size head-to-head, and therefore that space is safe
                    // to move into. it's often not.
                    if a.body.len() == b.body.len() {
                        // b can never be ME, because i'm always first in the
                        // list.
                        if a.id == ME {
                            death_kind_map.insert(a.id, DeathKind::Honourable);
                            keep[ai] = false;
                        }
                    } else {
                        if b.body.len() >= a.body.len() {
                            death_kind_map.insert(a.id, DeathKind::Honourable);
                            keep[ai] = false;
                        }
                        if a.body.len() >= b.body.len() {
                            death_kind_map.insert(b.id, DeathKind::Honourable);
                            keep[bi] = false;
                        }
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

        Ok((step, freespace, death_kind_map))
    }

    pub fn score(
        &self,
        snake: &Snake,
        freespace: &[bool],
        death_kind: DeathKind,
    ) -> Result<ScoreFactors> {
        if !self.snakes.contains(snake) {
            // we really don't want to die
            return Ok(ScoreFactors::dead(
                snake.id,
                death_kind,
                self.multisnake,
            ));
        }

        // floodfill is too expensive to run with more than 4 snakes.
        let available_squares = if self.snakes.len() <= 4 {
            self.floodfill(freespace, snake.body[0])?
        } else {
            0
        };

        let center_dist = manhattan_distance(
            snake.body[0],
            Coord {
                x: self.board.width / 2,
                y: self.board.height / 2,
            },
        );

        Ok(ScoreFactors::alive(
            snake.id,
            snake.health,
            snake.body.len() as i64,
            center_dist,
            self.snakes.len() as i64 - 1,
            available_squares,
            self.multisnake,
        ))
    }

    fn index(&self, c: Coord) -> Result<usize> {
        Ok(usize::try_from(c.x + c.y * self.board.width)?)
    }

    fn freespace_index(&self, coord: Coord) -> Result<Option<usize>> {
        if self.board.contains(coord) {
            Ok(Some(self.index(coord)?))
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

    fn is_space_free(&self, freespace: &[bool], coord: Coord) -> Result<bool> {
        Ok(self
            .freespace_index(coord)?
            .map_or(false, |index| freespace[index]))
    }

    fn floodfill(&self, freespace: &[bool], seed: Coord) -> Result<i64> {
        let size = usize::try_from(self.board.width * self.board.height)?;
        let mut visited = vec![false; size];
        let mut queue = VecDeque::with_capacity(size);
        queue.push_back(seed);

        while let Some(c) = queue.pop_front() {
            visited[self.index(c)?] = true;
            for d in Direction::iter() {
                let neighbour = c.neighbour(*d);
                if self.is_space_free(freespace, neighbour)?
                    && !visited[self.index(neighbour)?]
                {
                    queue.push_back(neighbour);
                }
            }
        }

        Ok(visited.iter().filter(|v| **v).count() as i64)
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
