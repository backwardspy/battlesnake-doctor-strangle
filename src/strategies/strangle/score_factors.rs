use std::fmt;

use super::SnakeID;

#[derive(Debug, Clone, Copy)]
pub struct ScoreFactors {
    pub snake_id:             SnakeID,
    pub health:               i64,
    pub dead:                 bool,
    pub closest_food:         i64,
    pub closest_larger_snake: i64,
    pub remaining_opponents:  i64,
    pub depth:                i64,
}

impl ScoreFactors {
    const CLOSEST_FOOD_WEIGHT: i64 = -100;
    const DEPTH_WEIGHT: i64 = 1000;
    const HEALTH_WEIGHT: i64 = 100;
    const LARGE_SNAKE_DISTANCE_MAX: i64 = 3;
    const LARGE_SNAKE_DISTANCE_WEIGHT: i64 = 10000;
    const REMAINING_OPPONENTS_WEIGHT: i64 = -100000;

    pub fn alive(
        snake_id: SnakeID,
        health: i64,
        closest_food: i64,
        closest_larger_snake: i64,
        remaining_opponents: i64,
        depth: u64,
    ) -> ScoreFactors {
        ScoreFactors {
            snake_id,
            health,
            dead: false,
            closest_food,
            closest_larger_snake,
            remaining_opponents,
            depth: depth as i64,
        }
    }

    pub fn dead(snake_id: SnakeID, depth: u64) -> Self {
        Self {
            snake_id,
            health: 0,
            dead: true,
            closest_food: 0,
            closest_larger_snake: 0,
            remaining_opponents: 0,
            depth: depth as i64,
        }
    }

    pub fn calculate(&self) -> i64 {
        if self.dead {
            -100000000 + self.depth as i64 * Self::DEPTH_WEIGHT
        } else {
            self.health * Self::HEALTH_WEIGHT
                + self.closest_food * Self::CLOSEST_FOOD_WEIGHT
                + self
                    .closest_larger_snake
                    .min(Self::LARGE_SNAKE_DISTANCE_MAX)
                    * Self::LARGE_SNAKE_DISTANCE_WEIGHT
                + self.remaining_opponents * Self::REMAINING_OPPONENTS_WEIGHT
                + self.depth as i64 * Self::DEPTH_WEIGHT
        }
    }
}

impl fmt::Display for ScoreFactors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.dead {
            write!(
                f,
                "{} @ d{} (snake {} is freakin dead dude)",
                self.calculate(),
                self.depth,
                self.snake_id
            )
        } else {
            write!(
                f,
                "{} @ d{} (snake {}):
                * {} health
                * {} turns from closest food
                * {} turns from closest larger snake (limit: {}),
                * {} remaining opponents)",
                self.calculate(),
                self.depth,
                self.snake_id,
                self.health,
                self.closest_food,
                self.closest_larger_snake,
                Self::LARGE_SNAKE_DISTANCE_MAX,
                self.remaining_opponents
            )
        }
    }
}
