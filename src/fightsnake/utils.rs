use crate::fightsnake::types::Coord;

#[must_use]
pub const fn manhattan_distance(a: Coord, b: Coord) -> i64 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}
