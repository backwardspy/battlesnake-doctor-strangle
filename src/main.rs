mod fightsnake;
mod strategies;

use anyhow::Result;
use fightsnake::{
    models::{GameState, Movement, Status},
    types::{APIVersion, Head, Tail},
};
use warp::http::Method;
use warp::Filter;

use strategies::{StrangleStrategy, Strategy};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let cors = warp::cors()
        .allow_method(Method::GET)
        .allow_method(Method::POST)
        .allow_header("content-type")
        .allow_any_origin();

    let logging = warp::log(NAME);

    let healthz = warp::get().and(warp::path::end().map(|| {
        warp::reply::json(&Status {
            apiversion: APIVersion::One,
            author: AUTHOR.to_owned(),
            color: "#A18CD1".to_owned(),
            head: Head::TransRightsScarf,
            tail: Tail::MysticMoon,
            version: VERSION.to_owned(),
        })
    }));

    let start = warp::post()
        .and(warp::path("start"))
        .and(warp::body::json())
        .map(|_state: GameState| "".to_owned());

    let do_move = warp::post()
        .and(warp::path("move"))
        .and(warp::body::json())
        .map(|state: GameState| {
            let movement = StrangleStrategy.get_movement(&state);
            warp::reply::json(&Movement {
                movement,
                shout: None,
            })
        });

    let api = healthz.or(start).or(do_move).with(cors).with(logging);

    warp::serve(api).run(([0, 0, 0, 0], 6502)).await;

    Ok(())
}
