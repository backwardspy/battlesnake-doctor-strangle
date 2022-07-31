use std::process::Command;

use color_eyre::Result;
use reqwest::Url;

enum GameMode {
    Solo,
}

impl ToString for GameMode {
    fn to_string(&self) -> String {
        match self {
            Self::Solo => "solo".to_owned(),
        }
    }
}

struct PlayOptions {
    board_width:  u64,
    board_height: u64,
    mode:         GameMode,
}

struct Snake {
    name: String,
    url:  Url,
}

fn make_play_command(play_options: &PlayOptions, snakes: &[Snake]) -> Command {
    let mut cmd = Command::new("battlesnake");
    cmd.arg("play");

    cmd.arg("--width");
    cmd.arg(play_options.board_width.to_string());
    cmd.arg("--height");
    cmd.arg(play_options.board_height.to_string());

    for snake in snakes {
        cmd.arg("--name");
        cmd.arg(&snake.name);

        cmd.arg("--url");
        cmd.arg(snake.url.to_string());
    }

    cmd.arg("--gametype");
    cmd.arg(play_options.mode.to_string());

    cmd.arg("--browser");

    cmd
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let snakes = &mut [Snake {
        name: "🧙 doctor strangle".to_owned(),
        url:  "http://localhost:6502".parse()?,
    }];

    let mut play = make_play_command(
        &PlayOptions {
            board_width:  11,
            board_height: 11,
            mode:         GameMode::Solo,
        },
        snakes,
    );

    play.status()?;

    Ok(())
}
