use std::{fmt, slice::Iter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub enum APIVersion {
    #[serde(rename = "1")]
    One,
}

#[derive(Serialize, Debug, Copy, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn iter() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction; 4] = [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ];
        DIRECTIONS.iter()
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    pub fn between(from: &Coord, to: &Coord) -> Option<Direction> {
        // gets the most significant direction between two coordinates.
        // if the coordinates are the same, assumes up.
        // if the coordinates are exactly diagonal, gives the vertical direction
        // priority.
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        if dx.abs() > dy.abs() {
            match dx.signum() {
                1 => Some(Direction::Right),
                -1 => Some(Direction::Left),
                _ => None,
            }
        } else {
            match dy.signum() {
                1 => Some(Direction::Up),
                -1 => Some(Direction::Down),
                _ => None,
            }
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Direction::Left => "Left",
                Direction::Right => "Right",
                Direction::Up => "Up",
                Direction::Down => "Down",
            }
        )
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Coord {
    pub x: i64,
    pub y: i64,
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Coord {
    pub fn neighbour(&self, direction: Direction) -> Coord {
        Coord {
            x: self.x
                + match direction {
                    Direction::Right => 1,
                    Direction::Left => -1,
                    _ => 0,
                },
            y: self.y
                + match direction {
                    Direction::Up => 1,
                    Direction::Down => -1,
                    _ => 0,
                },
        }
    }
}

#[derive(Serialize, Debug)]
pub enum Head {
    #[serde(rename = "do-sammy")]
    DoSammy,
    #[serde(rename = "beach-puffin")]
    BeachPuffin,
    #[serde(rename = "cosmic-horror")]
    CosmicHorror,
    #[serde(rename = "crystal-power")]
    CrystalPower,
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "beluga")]
    Beluga,
    #[serde(rename = "bendr")]
    Bendr,
    #[serde(rename = "dead")]
    Dead,
    #[serde(rename = "evil")]
    Evil,
    #[serde(rename = "fang")]
    Fang,
    #[serde(rename = "pixel")]
    Pixel,
    #[serde(rename = "safe")]
    Safe,
    #[serde(rename = "sand-worm")]
    SandWorm,
    #[serde(rename = "shades")]
    Shades,
    #[serde(rename = "silly")]
    Silly,
    #[serde(rename = "smile")]
    Smile,
    #[serde(rename = "tongue")]
    Tongue,
    #[serde(rename = "rbc-bowler")]
    RbcBowler,
    #[serde(rename = "replit-mark")]
    ReplitMark,
    #[serde(rename = "all-seeing")]
    AllSeeing,
    #[serde(rename = "smart-caterpillar")]
    SmartCaterpillar,
    #[serde(rename = "trans-rights-scarf")]
    TransRightsScarf,
    #[serde(rename = "bonhomme")]
    Bonhomme,
    #[serde(rename = "earmuffs")]
    Earmuffs,
    #[serde(rename = "rudolph")]
    Rudolph,
    #[serde(rename = "scarf")]
    Scarf,
    #[serde(rename = "ski")]
    Ski,
    #[serde(rename = "snowman")]
    Snowman,
    #[serde(rename = "snow-worm")]
    SnowWorm,
    #[serde(rename = "caffeine")]
    Caffeine,
    #[serde(rename = "gamer")]
    Gamer,
    #[serde(rename = "tiger-king")]
    TigerKing,
    #[serde(rename = "workout")]
    Workout,
    #[serde(rename = "jackolantern")]
    Jackolantern,
    #[serde(rename = "pumpkin")]
    Pumpkin,
    #[serde(rename = "alligator")]
    Alligator,
    #[serde(rename = "comet")]
    Comet,
    #[serde(rename = "football")]
    Football,
    #[serde(rename = "iguana")]
    Iguana,
    #[serde(rename = "lantern-fish")]
    LanternFish,
    #[serde(rename = "mask")]
    Mask,
    #[serde(rename = "missile")]
    Missile,
    #[serde(rename = "moto-helmet")]
    MotoHelmet,
    #[serde(rename = "moustache")]
    Moustache,
    #[serde(rename = "rocket-helmet")]
    RocketHelmet,
    #[serde(rename = "snail")]
    Snail,
    #[serde(rename = "space-helmet")]
    SpaceHelmet,
    #[serde(rename = "chomp")]
    Chomp,
    #[serde(rename = "orca")]
    Orca,
    #[serde(rename = "pixel-round")]
    PixelRound,
    #[serde(rename = "sneaky")]
    Sneaky,
    #[serde(rename = "villain")]
    Villain,
    #[serde(rename = "viper")]
    Viper,
    #[serde(rename = "happy")]
    Happy,
    #[serde(rename = "whale")]
    Whale,
}

#[derive(Serialize, Debug)]
pub enum Tail {
    #[serde(rename = "do-sammy")]
    DoSammy,
    #[serde(rename = "beach-puffin")]
    BeachPuffin,
    #[serde(rename = "cosmic-horror")]
    CosmicHorror,
    #[serde(rename = "crystal-power")]
    CrystalPower,
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "block-bum")]
    BlockBum,
    #[serde(rename = "bolt")]
    Bolt,
    #[serde(rename = "curled")]
    Curled,
    #[serde(rename = "fat-rattle")]
    FatRattle,
    #[serde(rename = "freckled")]
    Freckled,
    #[serde(rename = "hook")]
    Hook,
    #[serde(rename = "pixel")]
    Pixel,
    #[serde(rename = "round-bum")]
    RoundBum,
    #[serde(rename = "sharp")]
    Sharp,
    #[serde(rename = "skinny")]
    Skinny,
    #[serde(rename = "small-rattle")]
    SmallRattle,
    #[serde(rename = "rbc-necktie")]
    RbcNecktie,
    #[serde(rename = "replit-notmark")]
    ReplitNotmark,
    #[serde(rename = "mystic-moon")]
    MysticMoon,
    #[serde(rename = "bonhomme")]
    Bonhomme,
    #[serde(rename = "flake")]
    Flake,
    #[serde(rename = "ice-skate")]
    IceSkate,
    #[serde(rename = "present")]
    Present,
    #[serde(rename = "coffee")]
    Coffee,
    #[serde(rename = "mouse")]
    Mouse,
    #[serde(rename = "tiger-tail")]
    TigerTail,
    #[serde(rename = "weight")]
    Weight,
    #[serde(rename = "leaf")]
    Leaf,
    #[serde(rename = "pumpkin")]
    Pumpkin,
    #[serde(rename = "alligator")]
    Alligator,
    #[serde(rename = "comet")]
    Comet,
    #[serde(rename = "fish")]
    Fish,
    #[serde(rename = "flame")]
    Flame,
    #[serde(rename = "football")]
    Football,
    #[serde(rename = "iguana")]
    Iguana,
    #[serde(rename = "ion")]
    Ion,
    #[serde(rename = "missile")]
    Missile,
    #[serde(rename = "skinny-jeans")]
    SkinnyJeans,
    #[serde(rename = "snail")]
    Snail,
    #[serde(rename = "tire")]
    Tire,
    #[serde(rename = "virus")]
    Virus,
    #[serde(rename = "ghost")]
    Ghost,
    #[serde(rename = "pixel-round")]
    PixelRound,
    #[serde(rename = "swirl")]
    Swirl,
    #[serde(rename = "swoop")]
    Swoop,
    #[serde(rename = "rattle")]
    Rattle,
    #[serde(rename = "rocket")]
    Rocket,
    #[serde(rename = "offroad")]
    Offroad,
    #[serde(rename = "shiny")]
    Shiny,
}
