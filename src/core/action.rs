//! Player actions — the vocabulary the UI sends to the core.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Move (dx, dy) by one tile. Bump-to-attack.
    Move(i32, i32),
    Wait,
    StairsDown,
    StairsUp,
    Pickup,
    UseItem(usize),
    Drop(usize),
    Wield(usize),
    Wear(usize),
    TakeOff(usize),
    RingOn(usize),
    RingOff(usize),
    Eat(usize),
    Quaff(usize),
    Read(usize),
    /// Fire a wand at a tile.
    FireWand(usize, u8, u8),
    Talk(String),
    Buy { item: usize, price: u32 },
    Sell { item: usize, price: u32 },
    Identify(usize),
}
