use crate::fightsnake::types::Coord;

#[derive(Clone, Debug)]
pub struct Board {
    pub width:  i64,
    pub height: i64,
}

impl Board {
    pub fn contains(&self, coord: Coord) -> bool {
        coord.x >= 0
            && coord.y >= 0
            && coord.x < self.width
            && coord.y < self.height
    }
}
