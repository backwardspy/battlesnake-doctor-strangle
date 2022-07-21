use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::fightsnake::types::{APIVersion, Coord, Direction, Head, Tail};

#[derive(Serialize, Debug)]
pub struct Status {
    pub apiversion: APIVersion,
    pub author: String,
    pub color: String,
    pub head: Head,
    pub tail: Tail,
    pub version: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Royale {
    pub shrink_every_n_turns: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Squad {
    pub allow_body_collisions: bool,
    pub shared_elimination: bool,
    pub shared_health: bool,
    pub shared_length: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub food_spawn_chance: u64,
    pub minimum_food: u64,
    pub hazard_damage_per_turn: u64,
    pub royale: Royale,
    pub squad: Squad,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
    pub settings: Option<Settings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub map: Option<String>,
    pub source: Option<String>,
    pub timeout: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Board {
    pub height: i64,
    pub width: i64,
    pub food: Vec<Coord>,
    pub hazards: Vec<Coord>,
    pub snakes: Vec<Snake>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Customizations {
    pub color: String,
    pub head: String,
    pub tail: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Snake {
    pub id: String,
    pub name: String,
    pub health: u64,
    pub body: VecDeque<Coord>,
    pub latency: u64,
    pub head: Coord,
    pub length: u64,
    pub shout: String,
    pub squad: String,
    pub customizations: Option<Customizations>,
}

impl Snake {
    pub fn facing(&self) -> Option<Direction> {
        Direction::between(&self.body[self.body.len() - 2], &self.head)
    }
}

impl PartialEq for Snake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameState {
    pub game: Game,
    pub turn: u64,
    pub board: Board,
    pub you: Snake,
}

#[derive(Serialize, Debug)]
pub struct Movement {
    #[serde(rename = "move")]
    pub movement: Direction,
    pub shout: Option<String>,
}
