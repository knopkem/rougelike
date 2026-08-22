//! Entities: the base entity, player, monsters, AI, and NPCs.

pub mod ai;
pub mod entity;
pub mod monster;
pub mod npc;
pub mod player;

pub use entity::Entity;
pub use monster::Monster;
pub use npc::Npc;
pub use player::Player;
