//! Core game state and turn pump. UI-agnostic and headless-testable.

pub mod action;
pub mod color;
pub mod events;
pub mod game;
pub mod message;
pub mod rng;
pub mod score;

pub use action::Action;
pub use color::Color;
pub use events::GameEvent;
pub use game::Game;
pub use message::MessageLog;
pub use rng::Rng;
pub use score::Score;
