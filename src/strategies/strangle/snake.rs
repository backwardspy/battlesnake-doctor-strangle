use std::collections::VecDeque;

use super::{board::Board, SnakeID};
use crate::fightsnake::types::{Coord, Direction};

#[derive(Clone, Debug)]
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
        match self.facing() {
            Some(facing) => Direction::iter()
                .copied()
                .filter(|d| {
                    *d != facing.opposite()
                        && board.contains(self.body[0].neighbour(*d))
                })
                .collect(),
            None => Direction::iter().copied().collect(),
        }
    }
}

impl PartialEq for Snake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
