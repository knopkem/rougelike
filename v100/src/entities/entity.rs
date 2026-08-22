//! The base entity: shared combat and position fields.

use crate::core::events::Pos;
use serde::{Deserialize, Serialize};

/// A base entity with position and combat statistics.
///
/// Both `Player` and `Monster` embed an `Entity` for their shared fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    /// Current position on the level.
    pub pos: Pos,
    /// Current hit points.
    pub hp: u32,
    /// Maximum hit points.
    pub max_hp: u32,
    /// Armor class.
    pub ac: i32,
    /// Base attack (to-hit bonus).
    pub attack: i32,
    /// Damage die (sides).
    pub damage_die: u32,
    /// Damage bonus.
    pub damage_bonus: i32,
    /// Whether this entity is alive.
    pub alive: bool,
}

impl Entity {
    /// Create a new entity at the given position.
    pub fn new(
        pos: Pos,
        hp: u32,
        ac: i32,
        attack: i32,
        damage_die: u32,
        damage_bonus: i32,
    ) -> Self {
        Self {
            pos,
            hp,
            max_hp: hp,
            ac,
            attack,
            damage_die,
            damage_bonus,
            alive: true,
        }
    }

    /// Whether the entity is alive.
    pub fn is_alive(&self) -> bool {
        self.alive && self.hp > 0
    }

    /// Apply damage, returning the amount actually dealt.
    pub fn damage(&mut self, amount: u32) -> u32 {
        let dealt = amount.min(self.hp);
        self.hp -= dealt;
        if self.hp == 0 {
            self.alive = false;
        }
        dealt
    }

    /// Heal, returning the amount actually restored.
    pub fn heal(&mut self, amount: u32) -> u32 {
        let restored = amount.min(self.max_hp.saturating_sub(self.hp));
        self.hp += restored;
        self.alive = self.hp > 0;
        restored
    }

    /// Move to a new position.
    pub fn move_to(&mut self, pos: Pos) {
        self.pos = pos;
    }

    /// The fraction of HP remaining (0.0 to 1.0).
    pub fn hp_fraction(&self) -> f32 {
        if self.max_hp == 0 {
            0.0
        } else {
            self.hp as f32 / self.max_hp as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_entity_is_alive() {
        let e = Entity::new(Pos::new(5, 5), 20, 5, 3, 6, 0);
        assert!(e.is_alive());
        assert_eq!(e.max_hp, 20);
    }

    #[test]
    fn damage_reduces_hp() {
        let mut e = Entity::new(Pos::new(0, 0), 10, 0, 0, 4, 0);
        let dealt = e.damage(4);
        assert_eq!(dealt, 4);
        assert_eq!(e.hp, 6);
    }

    #[test]
    fn lethal_damage_kills() {
        let mut e = Entity::new(Pos::new(0, 0), 5, 0, 0, 4, 0);
        e.damage(10);
        assert!(!e.is_alive());
        assert_eq!(e.hp, 0);
    }

    #[test]
    fn heal_capped_at_max() {
        let mut e = Entity::new(Pos::new(0, 0), 10, 0, 0, 4, 0);
        e.damage(5);
        let restored = e.heal(100);
        assert_eq!(restored, 5);
        assert_eq!(e.hp, 10);
    }

    #[test]
    fn hp_fraction() {
        let mut e = Entity::new(Pos::new(0, 0), 10, 0, 0, 4, 0);
        assert!((e.hp_fraction() - 1.0).abs() < f32::EPSILON);
        e.damage(5);
        assert!((e.hp_fraction() - 0.5).abs() < f32::EPSILON);
    }
}
