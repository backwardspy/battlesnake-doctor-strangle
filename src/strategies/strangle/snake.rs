use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

use super::{board::Board, SnakeID};
use crate::fightsnake::types::{Coord, Direction};

#[derive(Clone, Debug, Eq)]
pub struct Snake {
    pub id:     SnakeID,
    pub body:   VecDeque<Coord>,
    pub health: i64,
}

impl Snake {
    pub fn facing(&self) -> Option<Direction> {
        Direction::between(self.body[1], self.body[0])
    }

    pub fn possible_directions(&self, board: &Board) -> Vec<Direction> {
        Direction::iter()
            .copied()
            .filter(|d| {
                if let Some(facing) = self.facing() && facing.opposite() == *d {
                    // filter out our neck
                    return false;
                }
                board.contains(self.body[0].neighbour(*d))
            })
            .collect()
    }
}

impl PartialEq for Snake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Snake {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        for c in &self.body {
            c.hash(state);
        }
        self.health.hash(state);
    }
}
