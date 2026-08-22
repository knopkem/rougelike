//! Game events: the single hook through which the core notifies the UI and
//! audio layers of things that happened. The core never touches the terminal
//! or the sound card directly.

use serde::{Deserialize, Serialize};

/// A tile position (x, y) within a level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

impl Pos {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub const fn add(self, dx: i32, dy: i32) -> Self {
        let x = self.x as i32 + dx;
        let y = self.y as i32 + dy;
        Self {
            x: (if x < 0 {
                0
            } else if x > 79 {
                79
            } else {
                x
            }) as u32,
            y: (if y < 0 {
                0
            } else if y > 24 {
                24
            } else {
                y
            }) as u32,
        }
    }

    pub const fn manhattan(self, other: Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

/// Events emitted by the core during a turn. The UI renders effects from these
/// and the audio engine maps them to SFX.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEvent {
    /// The player moved (footstep).
    PlayerMoved { pos: Pos },
    /// The player attacked and hit.
    PlayerHit { pos: Pos, crit: bool },
    /// The player attacked and missed.
    PlayerMiss { pos: Pos },
    /// A monster attacked and hit the player.
    MonsterHitPlayer { crit: bool },
    /// A monster attacked and missed.
    MonsterMissPlayer,
    /// A monster died.
    MonsterDied { pos: Pos, name: String },
    /// The player died.
    PlayerDied { cause: String },
    /// The player picked up an item.
    Pickup { name: String },
    /// The player dropped an item.
    Drop { name: String },
    /// The player equipped an item.
    Equip { name: String },
    /// The player drank a potion.
    Quaff { name: String },
    /// The player ate food.
    Eat { name: String },
    /// The player read a scroll.
    Read { name: String },
    /// A wand was fired.
    WandFire { name: String, pos: Option<Pos> },
    /// The player gained a level.
    LevelUp { level: u32 },
    /// The player descended a level.
    StairsDown { depth: u32 },
    /// The player ascended a level.
    StairsUp { depth: u32 },
    /// A door was opened or closed.
    Door { pos: Pos, opened: bool },
    /// A trap triggered.
    Trap { pos: Pos, name: String },
    /// A quest was completed.
    QuestComplete { name: String },
    /// A quest was accepted.
    QuestAccepted { name: String },
    /// The player won the game.
    Victory { score: u64 },
    /// The player teleported.
    Teleport { pos: Pos },
    /// Gold changed hands (shop or pickup).
    Coin { amount: u32 },
    /// A monster used a special ability.
    MonsterAbility { name: String, pos: Pos },
    /// A generic system notice (no specific SFX).
    Notice { text: String },
}
