#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used
)]
#![allow(
    clippy::implicit_hasher,    // fixing this one is beyond me
)]
pub mod fightsnake;
pub mod strategies;
