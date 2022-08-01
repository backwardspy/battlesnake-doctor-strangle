use std::fmt;

use super::SnakeID;

#[derive(Debug, Clone, Copy)]
pub enum DeathKind {
    Normal,
    Honourable,
}

#[derive(Debug, Clone, Copy)]
pub struct ScoreFactors {
    pub snake_id:            SnakeID,
    pub health:              i64,
    pub length:              i64,
    pub center_dist:         i64,
    pub dead:                bool,
    pub death_kind:          DeathKind,
    pub remaining_opponents: i64,
    pub available_squares:   i64,
    pub multisnake:          bool,
}

impl ScoreFactors {
    const AVAILABLE_SQUARES_WEIGHT: i64 = 2500;
    const CENTER_DIST_WEIGHT: i64 = 250;
    const DEPTH_WEIGHT: i64 = 100;
    const HEALTH_WEIGHT: i64 = 200;
    const LENGTH_WEIGHT: i64 = 1500;
    const REMAINING_OPPONENTS_WEIGHT: i64 = 10_000;

    #[allow(clippy::too_many_arguments)]
    pub const fn alive(
        snake_id: SnakeID,
        health: i64,
        length: i64,
        center_dist: i64,
        remaining_opponents: i64,
        available_squares: i64,
        multisnake: bool,
    ) -> Self {
        Self {
            snake_id,
            health,
            length,
            center_dist,
            dead: false,
            death_kind: DeathKind::Normal,
            remaining_opponents,
            available_squares,
            multisnake,
        }
    }

    pub const fn dead(
        snake_id: SnakeID,
        death_kind: DeathKind,
        multisnake: bool,
    ) -> Self {
        Self {
            snake_id,
            health: 0,
            length: 0,
            center_dist: 0,
            dead: true,
            death_kind,
            remaining_opponents: 0,
            available_squares: 0,
            multisnake,
        }
    }

    pub fn calculate(&self, depth: u64) -> i64 {
        let depth = i64::try_from(depth).unwrap_or(i64::MAX);
        if self.dead {
            // die as late as possible
            match self.death_kind {
                DeathKind::Normal => -100_000_000 + depth * Self::DEPTH_WEIGHT,
                DeathKind::Honourable => {
                    -50_000_000 + depth * Self::DEPTH_WEIGHT
                },
            }
        } else if self.remaining_opponents == 0 && self.multisnake {
            // win as early as possible
            10_000_000 - depth * Self::DEPTH_WEIGHT
        } else {
            // otherwise, try to stay alive
            self.health * Self::HEALTH_WEIGHT
                + self.length * Self::LENGTH_WEIGHT
                - self.center_dist * Self::CENTER_DIST_WEIGHT
                - self.remaining_opponents * Self::REMAINING_OPPONENTS_WEIGHT
                + self.available_squares * Self::AVAILABLE_SQUARES_WEIGHT
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
                "snake {}:\n* {} health\n* {} length\n* {} turns from \
                 center\n* {} remaining opponents\n* {} available squares",
                self.snake_id,
                self.health,
                self.length,
                self.center_dist,
                self.remaining_opponents,
                self.available_squares,
            )
        }
    }
}
