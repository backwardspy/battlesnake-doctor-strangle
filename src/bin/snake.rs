use anyhow::Result;
use battlesnake_doctor_strangle::{
    fightsnake::{
        models::{GameState, Movement, Status},
        types::{APIVersion, Head, Tail},
    },
    strategies::{Strangle, Strategy},
};
use log::{error, info};
use warp::{http::Method, Filter};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
struct InternalError;
impl warp::reject::Reject for InternalError {}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    #[cfg(debug_assertions)]
    info!("running in debug mode");

    #[cfg(not(debug_assertions))]
    info!("running in release mode");

    let cors = warp::cors()
        .allow_method(Method::GET)
        .allow_method(Method::POST)
        .allow_header("content-type")
        .allow_any_origin();

    let logging = warp::log(NAME);

    let healthz = warp::get().and(warp::path::end().map(|| {
        warp::reply::json(&Status {
            apiversion: APIVersion::One,
            author:     AUTHOR.to_owned(),
            color:      "#AB4377".to_owned(),
            head:       Head::TransRightsScarf,
            tail:       Tail::MysticMoon,
            version:    VERSION.to_owned(),
        })
    }));

    let start = warp::post()
        .and(warp::path("start"))
        .and(warp::body::json())
        .map(|_state: GameState| "".to_owned());

    let do_move = warp::post()
        .and(warp::path("move"))
        .and(warp::body::json())
        .and_then(|game_state: GameState| async move {
            Strangle
                .get_movement(game_state)
                .map(|movement| {
                    warp::reply::json(&Movement {
                        movement,
                        shout: None,
                    })
                })
                .map_err(|e| {
                    error!("failed to get move: {}", e);
                    warp::reject::custom(InternalError)
                })
        });

    let api = healthz.or(start).or(do_move).with(cors).with(logging);

    warp::serve(api).run(([0, 0, 0, 0], 6502)).await;

    Ok(())
}
