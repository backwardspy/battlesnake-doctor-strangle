use std::collections::VecDeque;

use rand::Rng;

use super::{snake::Snake, SnakeID};
use crate::{
    fightsnake::types::Coord,
    strategies::strangle::{board::Board, game::Game},
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

pub fn make_game(
    num_players: u64,
    board_width: i64,
    board_height: i64,
) -> Game {
    let mut rng = rand::thread_rng();
    Game::new(
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
    )
}
