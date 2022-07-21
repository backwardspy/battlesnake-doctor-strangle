use crate::fightsnake::{
    models::{Board, GameState, Snake},
    types::{Coord, Direction},
    utils::manhattan_distance,
};
use std::fmt::Write;

use super::Strategy;

const SEARCH_DEPTH: u64 = 1;

pub struct StrangleStrategy;

fn is_out_of_bounds(coord: &Coord, board: &Board) -> bool {
    coord.x < 0 || coord.y < 0 || coord.x >= board.width || coord.y >= board.height
}

fn is_colliding(snake: &Snake, board: &Board) -> bool {
    snake.body.iter().skip(1).any(|c| *c == snake.head)
        || board
            .snakes
            .iter()
            .any(|other| other != snake && other.body.contains(&snake.head))
}

fn game_is_over(board: &Board, snake_idx: usize) -> bool {
    let snake = &board.snakes[snake_idx];
    let nobody_left = board.snakes.len() <= 1;
    let left_board = is_out_of_bounds(&snake.head, board);
    let collided = is_colliding(snake, board);
    println!("\x1B[41;30msnake #{snake_idx} checking if game is over:");
    dbg!(nobody_left);
    dbg!(left_board);
    dbg!(collided);
    println!(
        "game over: {}\x1B[0m",
        nobody_left || left_board || collided
    );
    nobody_left || left_board || collided
}

fn score(state: &GameState, snake: &Snake) -> i64 {
    if is_out_of_bounds(&snake.head, &state.board) || is_colliding(snake, &state.board) {
        // dying is always a bad idea
        println!("snake {} dies in this future", snake.id);
        return i64::MIN;
    }

    let mut score = 1337;

    // being closer to food is a good idea
    score -= state
        .board
        .food
        .iter()
        .map(|food| manhattan_distance(&snake.head, food))
        .min()
        .unwrap_or(0);

    println!("closest food is {} moves away", 1337 - score);

    score
}

fn move_snake(state: &GameState, snake: &Snake, direction: Direction) -> GameState {
    let mut new_state = state.clone();
    let snake = new_state
        .board
        .snakes
        .iter_mut()
        .find(|s| *s == snake)
        .unwrap();

    // move the snake
    snake.head = snake.head.neighbour(direction);
    snake.body.push_front(snake.head);
    snake.body.pop_back();

    // update the copy if necessary
    if *snake == state.you {
        new_state.you = snake.clone();
    }

    new_state
}

fn remove_food(state: &GameState) -> GameState {
    let mut eaten = vec![];
    for food in &state.board.food {
        for snake in &state.board.snakes {
            if *food == snake.head {
                eaten.push(*food);
            }
        }
    }

    let mut new_state = state.clone();
    new_state.board.food.retain(|food| !eaten.contains(&food));
    new_state
}

fn indent(depth: u64, start_depth: u64) -> String {
    let level = start_depth.saturating_sub(depth);
    let mut res = String::with_capacity(level as usize * 4);

    write!(res, "{}", depth).unwrap();

    if level == 0 {
        write!(res, "┌─").unwrap();
        return res;
    }

    write!(res, "├─").unwrap();
    for _ in 0..level {
        write!(res, "──").unwrap();
    }
    res
}

fn possible_directions(facing: Option<Direction>) -> Vec<Direction> {
    match facing {
        Some(next_facing) => Direction::iter()
            .copied()
            .filter(|d| *d != next_facing.opposite())
            .collect(),
        None => Direction::iter().copied().collect(),
    }
}

fn find_best_move(state: &GameState, snake: &Snake) -> GameState {
    let index = state.board.snakes.iter().position(|s| s == snake).unwrap();
    println!("finding best moves for snake {}", snake.id);
    possible_directions(snake.facing())
        .iter()
        .map(|direction| {
            println!("trying {direction}");
            move_snake(state, snake, *direction)
        })
        .max_by_key(|state| score(state, &state.board.snakes[index]))
        .unwrap()
}

fn search_the_future(state: &GameState, depth: u64, start_depth: u64) -> Direction {
    // generate a new state by asking each snake to make its best next move.
    let new_state = state
        .board
        .snakes
        .iter()
        .fold(state.clone(), |state, snake| find_best_move(&state, snake));
    let new_state = remove_food(&new_state);

    // figure out where we went.
    Direction::between(&state.you.head, &new_state.you.head)
        .expect("find_best_move has to actually move the snake")
}

impl Strategy for StrangleStrategy {
    fn get_movement(&self, state: &GameState) -> Direction {
        println!("searching to depth {SEARCH_DEPTH}");

        let direction = search_the_future(state, SEARCH_DEPTH, SEARCH_DEPTH);

        println!("going {direction}");
        direction
    }
}
