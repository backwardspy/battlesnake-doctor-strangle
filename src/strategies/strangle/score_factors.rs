use std::fmt;

use super::SnakeID;

#[derive(Debug)]
pub struct ScoreFactors {
    snake_id:             SnakeID,
    health:               i64,
    dead:                 bool,
    closest_food:         i64,
    closest_larger_snake: i64,
    remaining_opponents:  i64,
    depth:                i64,
}

impl ScoreFactors {
    const CLOSEST_FOOD_WEIGHT: i64 = -100;
    const CLOSEST_LARGER_SNAKE_MAX: i64 = 3;
    const CLOSEST_LARGER_SNAKE_WEIGHT: i64 = 1500;
    const DEPTH_WEIGHT: i64 = 1000;
    const HEALTH_WEIGHT: i64 = 100;
    const REMAINING_OPPONENTS_WEIGHT: i64 = -10000;

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
            -1000000 + self.depth as i64 * Self::DEPTH_WEIGHT
        } else {
            self.health * Self::HEALTH_WEIGHT
                + self.closest_food * Self::CLOSEST_FOOD_WEIGHT
                + self
                    .closest_larger_snake
                    .min(Self::CLOSEST_LARGER_SNAKE_MAX)
                    * Self::CLOSEST_LARGER_SNAKE_WEIGHT
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
                "{} (snake {} is freakin dead dude)",
                self.calculate(),
                self.snake_id
            )
        } else {
            write!(
                f,
                "{} (snake {} @ {} health, {} turns from closest food, {} \
                 turns from closest larger snake (limit: {}), {} remaining \
                 opponents)",
                self.calculate(),
                self.snake_id,
                self.health,
                self.closest_food,
                self.closest_larger_snake,
                Self::CLOSEST_LARGER_SNAKE_MAX,
                self.remaining_opponents
            )
        }
    }
}
