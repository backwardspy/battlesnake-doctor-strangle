use crate::fightsnake::types::Coord;

pub fn manhattan_distance(a: Coord, b: Coord) -> i64 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
