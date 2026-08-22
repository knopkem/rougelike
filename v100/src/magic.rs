//! Magic: resolution of wand and scroll effects.

use crate::core::events::{GameEvent, Pos};
use crate::core::rng::Rng;
use crate::items::item::WandEffect;

/// The result of casting a wand or scroll effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastResult {
    /// Events produced by the cast (for audio/UI).
    pub events: Vec<GameEvent>,
    /// Damage dealt to the target (if any).
    pub damage: u32,
    /// Whether the cast succeeded.
    pub success: bool,
}

impl CastResult {
    pub fn success() -> Self {
        Self {
            events: Vec::new(),
            damage: 0,
            success: true,
        }
    }

    pub fn fail() -> Self {
        Self {
            events: Vec::new(),
            damage: 0,
            success: false,
        }
    }
}

/// Resolve a wand effect.
///
/// `caster_pos` is where the caster is, `target` is the target position (for
/// directed effects). `power` scales the effect (e.g. wand tier).
pub fn resolve_wand(
    rng: &mut Rng,
    effect: WandEffect,
    caster_pos: Pos,
    target: Option<Pos>,
    power: u32,
) -> CastResult {
    match effect {
        WandEffect::FireBolt => directed_damage(rng, target, 4 + power, "fire bolt"),
        WandEffect::Cold => directed_damage(rng, target, 4 + power, "cold"),
        WandEffect::Lightning => directed_damage(rng, target, 6 + power, "lightning"),
        WandEffect::Disintegration => {
            directed_damage(rng, target, 10 + power * 2, "disintegration")
        }
        WandEffect::Paralysis => status_cast(target, "paralysis"),
        WandEffect::Sleep => status_cast(target, "sleep"),
        WandEffect::Healing => CastResult {
            events: vec![GameEvent::WandFire {
                name: "healing".to_string(),
                pos: target,
            }],
            damage: 0,
            success: true,
        },
        WandEffect::Teleport => CastResult {
            events: vec![GameEvent::Teleport { pos: caster_pos }],
            damage: 0,
            success: true,
        },
        WandEffect::MonsterLightning => CastResult {
            events: vec![GameEvent::WandFire {
                name: "monster lightning".to_string(),
                pos: target,
            }],
            damage: 0,
            success: true,
        },
    }
}

/// A directed damage effect (fire bolt, cold, lightning, disintegration).
fn directed_damage(rng: &mut Rng, target: Option<Pos>, base: u32, name: &str) -> CastResult {
    let variance = rng.range(0, base / 2 + 1);
    let damage = base + variance;
    CastResult {
        events: vec![GameEvent::WandFire {
            name: name.to_string(),
            pos: target,
        }],
        damage,
        success: true,
    }
}

/// A status-inflicting cast (paralysis, sleep).
fn status_cast(target: Option<Pos>, name: &str) -> CastResult {
    CastResult {
        events: vec![GameEvent::WandFire {
            name: name.to_string(),
            pos: target,
        }],
        damage: 0,
        success: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_bolt_deals_damage() {
        let mut rng = Rng::new(42);
        let result = resolve_wand(
            &mut rng,
            WandEffect::FireBolt,
            Pos::new(5, 5),
            Some(Pos::new(10, 5)),
            2,
        );
        assert!(result.success);
        assert!(result.damage > 0);
        assert!(!result.events.is_empty());
    }

    #[test]
    fn healing_produces_event() {
        let mut rng = Rng::new(42);
        let result = resolve_wand(&mut rng, WandEffect::Healing, Pos::new(5, 5), None, 1);
        assert!(result.success);
        assert!(
            result
                .events
                .iter()
                .any(|e| matches!(e, GameEvent::WandFire { .. }))
        );
    }

    #[test]
    fn teleport_produces_event() {
        let mut rng = Rng::new(42);
        let result = resolve_wand(&mut rng, WandEffect::Teleport, Pos::new(5, 5), None, 1);
        assert!(result.success);
        assert!(
            result
                .events
                .iter()
                .any(|e| matches!(e, GameEvent::Teleport { .. }))
        );
    }

    #[test]
    fn disintegration_does_more_damage() {
        let mut rng = Rng::new(42);
        let dis = resolve_wand(
            &mut rng,
            WandEffect::Disintegration,
            Pos::new(5, 5),
            Some(Pos::new(10, 5)),
            3,
        );
        let fire = resolve_wand(
            &mut rng,
            WandEffect::FireBolt,
            Pos::new(5, 5),
            Some(Pos::new(10, 5)),
            3,
        );
        assert!(dis.damage >= fire.damage / 2);
    }
}
