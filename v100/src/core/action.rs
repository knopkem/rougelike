//! Player actions: the only way game state changes is via `Game::do_turn`.

use crate::core::events::Pos;
use serde::{Deserialize, Serialize};

/// A direction of movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
        }
    }

    /// All 8 directions.
    pub const ALL: [Direction; 8] = [
        Direction::North,
        Direction::NorthEast,
        Direction::East,
        Direction::SouthEast,
        Direction::South,
        Direction::SouthWest,
        Direction::West,
        Direction::NorthWest,
    ];
}

/// An action the player takes. Each action consumes one turn (except `None`,
/// which is used for "no action" bookkeeping).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Move in a direction (bump-to-attack if a monster is in the way).
    Move(Direction),
    /// Wait one turn.
    Wait,
    /// Descend the stairs (explicit confirm happens in the UI).
    StairsDown,
    /// Ascend the stairs.
    StairsUp,
    /// Pick up the item on the current tile.
    Pickup,
    /// Drop the inventory item at the given index.
    Drop { index: usize },
    /// Use (quaff) the inventory item at the given index.
    Quaff { index: usize },
    /// Eat the inventory item at the given index.
    Eat { index: usize },
    /// Read the inventory item at the given index.
    Read { index: usize },
    /// Fire the wand at the given index toward a target position.
    WandFire { index: usize, target: Pos },
    /// Wield the weapon at the given index.
    Wield { index: usize },
    /// Wear the armor at the given index.
    Wear { index: usize },
    /// Take off the equipped item at the given index.
    TakeOff { index: usize },
    /// Put on the ring at the given index.
    RingOn { index: usize },
    /// Take off the ring at the given index.
    RingOff { index: usize },
    /// Open or close the door on the current tile.
    ToggleDoor,
    /// Attempt to pick the lock of the door on the current tile.
    PickLock,
    /// Buy the shop item at the given index.
    ShopBuy { index: usize },
    /// Sell the inventory item at the given index.
    ShopSell { index: usize },
    /// Have the wizard identify the inventory item at the given index.
    Identify { index: usize },
    /// Accept the quest at the given index (from a quest giver).
    AcceptQuest { index: usize },
    /// Turn in the quest at the given index.
    TurnInQuest { index: usize },
    /// Save and quit (handled by the UI; no turn consumed).
    SaveQuit,
    /// Abort without saving (handled by the UI; no turn consumed).
    Abort,
}
