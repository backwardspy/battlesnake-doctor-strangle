use std::fmt;

use super::SnakeID;

#[derive(Debug, Clone, Copy)]
pub struct ScoreFactors {
    pub snake_id:              SnakeID,
    pub health:                i64,
    pub length:                i64,
    pub dead:                  bool,
    pub closest_food:          i64,
    pub closest_larger_snake:  i64,
    pub closest_smaller_snake: i64,
    pub remaining_opponents:   i64,
    pub multisnake:            bool,
}

impl ScoreFactors {
    const CLOSEST_FOOD_WEIGHT: i64 = -250;
    const DEPTH_WEIGHT: i64 = 100;
    const HEALTH_WEIGHT: i64 = 500;
    const LARGE_SNAKE_DISTANCE_MAX: i64 = 3;
    const LARGE_SNAKE_DISTANCE_WEIGHT: i64 = 500;
    const LENGTH_WEIGHT: i64 = 1000;
    const REMAINING_OPPONENTS_WEIGHT: i64 = -10_000;
    const SMALL_SNAKE_DISTANCE_WEIGHT: i64 = 2500;

    #[allow(clippy::too_many_arguments)]
    pub const fn alive(
        snake_id: SnakeID,
        health: i64,
        length: i64,
        closest_food: i64,
        closest_larger_snake: i64,
        closest_smaller_snake: i64,
        remaining_opponents: i64,
        multisnake: bool,
    ) -> Self {
        Self {
            snake_id,
            health,
            length,
            dead: false,
            closest_food,
            closest_larger_snake,
            closest_smaller_snake,
            remaining_opponents,
            multisnake,
        }
    }

    pub const fn dead(snake_id: SnakeID, multisnake: bool) -> Self {
        Self {
            snake_id,
            health: 0,
            length: 0,
            dead: true,
            closest_food: 0,
            closest_larger_snake: 0,
            closest_smaller_snake: 0,
            remaining_opponents: 0,
            multisnake,
        }
    }

    pub fn calculate(&self, depth: u64) -> i64 {
        let depth = i64::try_from(depth).unwrap_or(i64::MAX);
        if self.dead {
            // die as late as possible
            -100_000_000 + depth * Self::DEPTH_WEIGHT
        } else if self.remaining_opponents == 0 && self.multisnake {
            // win as early as possible
            10_000_000 - depth * Self::DEPTH_WEIGHT
        } else {
            // otherwise, try to stay alive
            self.health * Self::HEALTH_WEIGHT
                + self.length * Self::LENGTH_WEIGHT
                + self.closest_food * Self::CLOSEST_FOOD_WEIGHT
                + self
                    .closest_larger_snake
                    .min(Self::LARGE_SNAKE_DISTANCE_MAX)
                    * Self::LARGE_SNAKE_DISTANCE_WEIGHT
                + self.closest_smaller_snake * Self::SMALL_SNAKE_DISTANCE_WEIGHT
                + self.remaining_opponents * Self::REMAINING_OPPONENTS_WEIGHT
                + depth * Self::DEPTH_WEIGHT
        }
    }
}

impl fmt::Display for ScoreFactors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.dead {
            write!(f, "snake {} is freakin dead dude", self.snake_id)
        } else {
            write!(
                f,
                "snake {}:\n\
                * {} health\n\
                * {} length\n\
                * {} turns from closest food\n\
                * {} turns from closest larger snake (limited to: {})\n\
                * {} turns from closest smaller snake\n\
                * {} remaining opponents)",
                self.snake_id,
                self.health,
                self.length,
                self.closest_food,
                self.closest_larger_snake,
                self.closest_larger_snake.min(Self::LARGE_SNAKE_DISTANCE_MAX),
                self.closest_smaller_snake,
                self.remaining_opponents
            )
        }
    }
}
