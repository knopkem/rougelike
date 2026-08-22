//! Sound effect identifiers. The UI maps `GameEvent`s to these, and the audio
//! engine synthesizes each one on demand.

/// A sound effect to play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    /// Footstep.
    Step,
    /// The player hits a monster.
    PlayerHit,
    /// The player lands a critical hit.
    PlayerCrit,
    /// The player misses.
    Miss,
    /// A monster hits the player.
    MonsterHit,
    /// A monster dies.
    MonsterDeath,
    /// The player dies.
    PlayerDeath,
    /// Picking up an item.
    Pickup,
    /// Dropping an item.
    Drop,
    /// Equipping an item.
    Equip,
    /// Drinking a potion.
    Quaff,
    /// Eating food.
    Eat,
    /// Reading a scroll.
    Read,
    /// Firing a wand.
    WandFire,
    /// Leveling up.
    LevelUp,
    /// Descending stairs.
    StairsDown,
    /// Ascending stairs.
    StairsUp,
    /// Opening/closing a door.
    Door,
    /// A trap triggers.
    Trap,
    /// A quest is completed.
    QuestComplete,
    /// A quest is accepted.
    QuestAccepted,
    /// Victory.
    Victory,
    /// Teleport.
    Teleport,
    /// A coin changes hands.
    Coin,
    /// A monster uses a special ability.
    MonsterAbility,
    /// A generic blip.
    Blip,
}

impl Sfx {
    /// Map a game event to the sound effect it should produce.
    pub fn from_event(event: &crate::core::events::GameEvent) -> Option<Sfx> {
        use crate::core::events::GameEvent;
        match event {
            GameEvent::PlayerMoved { .. } => Some(Sfx::Step),
            GameEvent::PlayerHit { crit, .. } => {
                if *crit {
                    Some(Sfx::PlayerCrit)
                } else {
                    Some(Sfx::PlayerHit)
                }
            }
            GameEvent::PlayerMiss { .. } => Some(Sfx::Miss),
            GameEvent::MonsterHitPlayer { .. } => Some(Sfx::MonsterHit),
            GameEvent::MonsterMissPlayer => Some(Sfx::Miss),
            GameEvent::MonsterDied { .. } => Some(Sfx::MonsterDeath),
            GameEvent::PlayerDied { .. } => Some(Sfx::PlayerDeath),
            GameEvent::Pickup { .. } => Some(Sfx::Pickup),
            GameEvent::Drop { .. } => Some(Sfx::Drop),
            GameEvent::Equip { .. } => Some(Sfx::Equip),
            GameEvent::Quaff { .. } => Some(Sfx::Quaff),
            GameEvent::Eat { .. } => Some(Sfx::Eat),
            GameEvent::Read { .. } => Some(Sfx::Read),
            GameEvent::WandFire { .. } => Some(Sfx::WandFire),
            GameEvent::LevelUp { .. } => Some(Sfx::LevelUp),
            GameEvent::StairsDown { .. } => Some(Sfx::StairsDown),
            GameEvent::StairsUp { .. } => Some(Sfx::StairsUp),
            GameEvent::Door { .. } => Some(Sfx::Door),
            GameEvent::Trap { .. } => Some(Sfx::Trap),
            GameEvent::QuestComplete { .. } => Some(Sfx::QuestComplete),
            GameEvent::QuestAccepted { .. } => Some(Sfx::QuestAccepted),
            GameEvent::Victory { .. } => Some(Sfx::Victory),
            GameEvent::Teleport { .. } => Some(Sfx::Teleport),
            GameEvent::Coin { .. } => Some(Sfx::Coin),
            GameEvent::MonsterAbility { .. } => Some(Sfx::MonsterAbility),
            GameEvent::Notice { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::{GameEvent, Pos};

    #[test]
    fn event_maps_to_sfx() {
        let sfx = Sfx::from_event(&GameEvent::LevelUp { level: 2 }).unwrap();
        assert_eq!(sfx, Sfx::LevelUp);
    }

    #[test]
    fn notice_has_no_sfx() {
        let sfx = Sfx::from_event(&GameEvent::Notice {
            text: "hi".to_string(),
        });
        assert!(sfx.is_none());
    }

    #[test]
    fn crit_maps_to_player_crit() {
        let sfx = Sfx::from_event(&GameEvent::PlayerHit {
            pos: Pos::new(1, 1),
            crit: true,
        })
        .unwrap();
        assert_eq!(sfx, Sfx::PlayerCrit);
    }
}
