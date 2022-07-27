use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use rand::Rng;

use super::{snake::Snake, SnakeID};
use crate::{
    fightsnake::types::Coord,
    strategies::strangle::{board::Board, brain::bigbrain, game::Game},
};

fn make_snake(
    id: SnakeID,
    board_width: i64,
    board_height: i64,
    num_players: u64,
) -> Snake {
    let spacing = board_width / num_players as i64;
    let offset = spacing / 2;

    let xpos = offset + spacing * id as i64;

    let body: VecDeque<_> = (2..board_height - 2)
        .map(|y| Coord { x: xpos, y })
        .collect();

    Snake {
        id,
        body,
        health: 100,
    }
}

#[allow(dead_code)]
pub fn benchmark_game(
    num_players: u64,
    board_width: i64,
    board_height: i64,
) -> u64 {
    const LIMIT_MEAN: f64 = 250.0; // millis
    const RUNS: u64 = 3;

    let mut rng = rand::thread_rng();

    let game = Game::new(
        (0..num_players)
            .map(|id| {
                make_snake(
                    id as SnakeID,
                    board_width,
                    board_height,
                    num_players,
                )
            })
            .collect(),
        (5..rng.gen_range(0..10))
            .map(|_| Coord {
                x: rng.gen_range(0..board_width),
                y: rng.gen_range(0..board_height),
            })
            .collect(),
        Board {
            width:  board_width,
            height: board_height,
        },
    );

    println!(
        "measuring performance for a {num_players} player game with {RUNS} \
         runs per depth..."
    );

    for depth in 1..=20 {
        let millis = (0..RUNS)
            .map(|_| {
                let now = Instant::now();
                bigbrain(&game, 0, 0, depth, &HashMap::new(), false);
                now.elapsed().as_millis() as f64
            })
            .sum::<f64>()
            / RUNS as f64;

        if millis >= LIMIT_MEAN {
            let chosen_depth = (depth - 1).max(1);
            println!(
                "reached the limit of {LIMIT_MEAN} ms at depth {depth} (took \
                 {millis:.2} ms). going with a max depth of {chosen_depth}"
            );
            return chosen_depth;
        }
    }

    println!(
        "we somehow managed all tests without timing out, so going with a max \
         depth of 20."
    );
    println!("consider testing even further..?");
    20
}
