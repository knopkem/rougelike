//! Deep Delve — a production-quality, console-only roguelike.
//!
//! The game core is 100% UI-agnostic and headless-testable. `core::Game::do_turn`
//! is the only way game state changes. The UI maps terminal input to `Action`s,
//! renders `Game` state, and consumes `GameEvent`s (the single hook for audio
//! and UI effects).

pub mod audio;
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
pub mod ui;
