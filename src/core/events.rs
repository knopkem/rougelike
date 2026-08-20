//! Game events emitted by the core. The UI maps these to audio and effects.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameEvent {
    Footstep,
    Hit { crit: bool },
    Miss,
    MonsterDeath { tier: u8 },
    PlayerDeath,
    Pickup,
    Drop,
    Equip,
    Quaff,
    Eat,
    WandCast { kind: String },
    PotionSplash,
    ScrollRead,
    LevelUp,
    Stairs,
    Door { opened: bool, locked: bool },
    Trap,
    Quest { kind: QuestEvent },
    Victory,
    Coin,
    Teleport,
    Poisoned,
    Healed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestEvent {
    Offered,
    Accepted,
    Completed,
    Failed,
}
