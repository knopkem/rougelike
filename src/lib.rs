// Deepdelve — a terminal roguelike in Rust.

pub mod audio;
pub mod combat;
pub mod core;
pub mod data;
pub mod entities;
pub mod items;
pub mod map;
pub mod hiscore;
pub mod magic;
pub mod quest;
pub mod save;
pub mod shop;
pub mod status;
pub mod ui;

pub use combat::Combat;
pub use core::game::Game;
pub use core::message::MessageLog;

#[cfg(test)]
pub mod tests_harness {
    //! Shared test helpers for integration tests.
    pub use crate::core::game::Game;
    pub use crate::core::rng::Rng;

    pub fn new_game(seed: u64) -> Game {
        Game::new_test("Test", "Human", "Warrior", seed)
    }
}
