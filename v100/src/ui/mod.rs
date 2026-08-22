//! The UI layer: terminal rendering (ratatui), input (crossterm), and menus.
//!
//! This is the only part of the codebase that touches the terminal. It maps
//! input to `Action`s, calls `Game::do_turn`, renders `Game` state, and
//! consumes `GameEvent`s to drive the audio engine.

pub mod app;
pub mod menu;
pub mod palette;
pub mod render;

pub use app::App;
