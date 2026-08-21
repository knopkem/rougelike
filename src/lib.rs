// Deepdelve — a terminal roguelike in Rust.

pub mod audio;
pub mod cli;
pub mod combat;
pub mod core;
pub mod data;
pub mod entities;
pub mod hiscore;
pub mod items;
pub mod magic;
pub mod map;
pub mod quest;
pub mod save;
pub mod shop;
pub mod status;
pub mod terminal;
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

    // Tests that touch the data dir must run one at a time: the dir is
    // selected via the process-wide XDG_DATA_HOME variable.
    static DATA_DIR_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    pub fn with_isolated_data_dir(name: &str, f: impl FnOnce()) {
        let _guard = match DATA_DIR_LOCK.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let dir =
            std::env::temp_dir().join(format!("deepdelve-test-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let previous = std::env::var("XDG_DATA_HOME").ok();
        std::env::set_var("XDG_DATA_HOME", &dir);
        f();
        match previous {
            Some(value) => std::env::set_var("XDG_DATA_HOME", value),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
