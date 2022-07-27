use std::{collections::VecDeque, fmt};

use serde::{de, Deserialize, Deserializer, Serialize};

use crate::fightsnake::types::{APIVersion, Coord, Direction, Head, Tail};

struct DeserializeU64OrStringVisitor;

impl<'de> de::Visitor<'de> for DeserializeU64OrStringVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or string")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.parse::<u64>().unwrap_or(0))
    }
}

fn from_string_or_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(DeserializeU64OrStringVisitor)
}

#[derive(Serialize, Debug)]
pub struct Status {
    pub apiversion: APIVersion,
    pub author:     String,
    pub color:      String,
    pub head:       Head,
    pub tail:       Tail,
    pub version:    String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub food_spawn_chance:      Option<u64>,
    pub minimum_food:           Option<u64>,
    pub hazard_damage_per_turn: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ruleset {
    pub name:     String,
    pub version:  String,
    pub settings: Option<Settings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Game {
    pub id:      String,
    pub ruleset: Ruleset,
    pub map:     Option<String>,
    pub source:  Option<String>,
    pub timeout: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Board {
    pub height:  i64,
    pub width:   i64,
    pub food:    Vec<Coord>,
    pub hazards: Vec<Coord>,
    pub snakes:  Vec<Snake>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Customizations {
    pub color: String,
    pub head:  String,
    pub tail:  String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Snake {
    pub id:             String,
    pub name:           String,
    pub health:         i64,
    pub body:           VecDeque<Coord>,
    #[serde(deserialize_with = "from_string_or_u64")]
    pub latency:        u64,
    pub head:           Coord,
    pub length:         u64,
    pub shout:          String,
    pub squad:          String,
    pub customizations: Option<Customizations>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameState {
    pub game:  Game,
    pub turn:  u64,
    pub board: Board,
    pub you:   Snake,
}

#[derive(Serialize, Debug)]
pub struct Movement {
    #[serde(rename = "move")]
    pub movement: Direction,
    pub shout:    Option<String>,
}
