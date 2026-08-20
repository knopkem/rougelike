//! SFX engine: procedural sound playback with BEL fallback.

use crate::core::events::GameEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Footstep,
    Hit,
    Miss,
    Crit,
    MonsterDeath,
    PlayerDeath,
    Pickup,
    Equip,
    Quaff,
    Eat,
    WandCast,
    PotionSplash,
    ScrollRead,
    LevelUp,
    Stairs,
    Door,
    Trap,
    Quest,
    Victory,
    Coin,
    Teleport,
}

impl Sfx {
    pub fn from_event(ev: &GameEvent) -> Option<Sfx> {
        match ev {
            GameEvent::Footstep => Some(Sfx::Footstep),
            GameEvent::Hit { crit } => {
                if *crit {
                    Some(Sfx::Crit)
                } else {
                    Some(Sfx::Hit)
                }
            }
            GameEvent::Miss => Some(Sfx::Miss),
            GameEvent::MonsterDeath { .. } => Some(Sfx::MonsterDeath),
            GameEvent::PlayerDeath => Some(Sfx::PlayerDeath),
            GameEvent::Pickup => Some(Sfx::Pickup),
            GameEvent::Drop => Some(Sfx::Pickup),
            GameEvent::Equip => Some(Sfx::Equip),
            GameEvent::Quaff => Some(Sfx::Quaff),
            GameEvent::Eat => Some(Sfx::Eat),
            GameEvent::WandCast { .. } => Some(Sfx::WandCast),
            GameEvent::PotionSplash => Some(Sfx::PotionSplash),
            GameEvent::ScrollRead => Some(Sfx::ScrollRead),
            GameEvent::LevelUp => Some(Sfx::LevelUp),
            GameEvent::Stairs => Some(Sfx::Stairs),
            GameEvent::Door { .. } => Some(Sfx::Door),
            GameEvent::Trap => Some(Sfx::Trap),
            GameEvent::Quest { .. } => Some(Sfx::Quest),
            GameEvent::Victory => Some(Sfx::Victory),
            GameEvent::Coin => Some(Sfx::Coin),
            GameEvent::Teleport => Some(Sfx::Teleport),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioBackend {
    #[default]
    Bell,
    Rodio,
}

/// SFX engine with mute support and backend fallback.
pub struct SfxEngine {
    muted: bool,
    backend: AudioBackend,
}

impl SfxEngine {
    pub fn new() -> Self {
        Self {
            muted: false,
            backend: AudioBackend::Bell,
        }
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.muted
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Play an SFX (no-op if muted).
    pub fn play(&mut self, sfx: Sfx) {
        if self.muted {
            return;
        }
        match self.backend {
            AudioBackend::Bell => {
                // Terminal BEL for critical cues.
                if matches!(sfx, Sfx::PlayerDeath | Sfx::Victory | Sfx::Crit) {
                    print!("\x07");
                }
            }
            AudioBackend::Rodio => {
                #[cfg(feature = "audio")]
                {
                    // Real rodio playback would go here.
                    let _ = sfx;
                }
                #[cfg(not(feature = "audio"))]
                {
                    let _ = sfx;
                }
            }
        }
    }

    pub fn play_event(&mut self, ev: &GameEvent) {
        if let Some(sfx) = Sfx::from_event(ev) {
            self.play(sfx);
        }
    }
}

impl Default for SfxEngine {
    fn default() -> Self {
        Self::new()
    }
}
