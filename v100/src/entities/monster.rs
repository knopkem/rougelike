//! Monsters: a monster instance wrapping an entity with a definition.

use crate::core::events::Pos;
use crate::data::monsters::{Ability, MonsterDef};
use crate::entities::entity::Entity;
use crate::status::StatusSet;
use serde::{Deserialize, Serialize};

/// A monster instance on the level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Monster {
    /// Shared combat/position fields.
    pub entity: Entity,
    /// The monster's definition id.
    pub def_id: u32,
    /// Active status effects.
    pub statuses: StatusSet,
    /// Whether this monster has already acted this turn (for AI).
    pub acted: bool,
    /// Whether this monster is currently in "enraged" state.
    pub enraged: bool,
}

impl Monster {
    /// Create a monster from a definition at the given position.
    pub fn from_def(def: &MonsterDef, pos: Pos) -> Self {
        let entity = Entity::new(
            pos,
            def.hp,
            def.ac,
            def.attack,
            def.damage_die,
            def.damage_bonus,
        );
        Self {
            entity,
            def_id: def.id,
            statuses: StatusSet::new(),
            acted: false,
            enraged: false,
        }
    }

    /// The monster's definition.
    pub fn def(&self) -> &'static MonsterDef {
        MonsterDef::by_id(self.def_id).expect("valid monster def id")
    }

    /// The monster's name.
    pub fn name(&self) -> &'static str {
        self.def().name
    }

    /// The monster's position.
    pub fn pos(&self) -> Pos {
        self.entity.pos
    }

    /// Whether the monster is alive.
    pub fn is_alive(&self) -> bool {
        self.entity.is_alive()
    }

    /// Current hit points.
    pub fn hp(&self) -> u32 {
        self.entity.hp
    }

    /// Maximum hit points.
    pub fn max_hp(&self) -> u32 {
        self.entity.max_hp
    }

    /// Apply damage to the monster.
    pub fn damage(&mut self, amount: u32) -> u32 {
        self.entity.damage(amount)
    }

    /// Heal the monster.
    pub fn heal(&mut self, amount: u32) -> u32 {
        self.entity.heal(amount)
    }

    /// Move the monster.
    pub fn move_to(&mut self, pos: Pos) {
        self.entity.move_to(pos);
    }

    /// Whether the monster has a given ability.
    pub fn has_ability(&self, ability: Ability) -> bool {
        self.def().abilities.contains(&ability)
    }

    /// Whether the monster can attack at range.
    pub fn is_ranged(&self) -> bool {
        self.def().ranged
    }

    /// The monster's XP value.
    pub fn xp_value(&self) -> u32 {
        self.def().xp
    }

    /// The monster's tier.
    pub fn tier(&self) -> u8 {
        self.def().tier
    }

    /// Whether this is a unique (boss) monster.
    pub fn is_unique(&self) -> bool {
        self.def().unique
    }

    /// Reset the per-turn acted flag.
    pub fn reset_turn(&mut self) {
        self.acted = false;
    }

    /// Mark the monster as having acted.
    pub fn mark_acted(&mut self) {
        self.acted = true;
    }

    /// Whether the monster is currently unable to act (paralyzed/petrified/sleeping).
    pub fn is_stunned(&self) -> bool {
        self.statuses.has(crate::status::Status::Paralyzed)
            || self.statuses.has(crate::status::Status::Petrified)
            || self.statuses.has(crate::status::Status::Sleeping)
    }

    /// The monster's effective AC (with berserk penalty if applicable).
    pub fn effective_ac(&self) -> i32 {
        let mut ac = self.entity.ac;
        if self.statuses.has(crate::status::Status::Berserk) {
            ac -= 2;
        }
        ac
    }

    /// The monster's effective attack (with enrage bonus if applicable).
    pub fn effective_attack(&self) -> i32 {
        let mut atk = self.entity.attack;
        if self.enraged {
            atk += 3;
        }
        atk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_monster() -> Monster {
        let def = MonsterDef::by_id(3).unwrap(); // goblin
        Monster::from_def(def, Pos::new(5, 5))
    }

    #[test]
    fn from_def_uses_stats() {
        let m = test_monster();
        assert_eq!(m.name(), "goblin");
        assert_eq!(m.max_hp(), 5);
        assert!(m.is_alive());
    }

    #[test]
    fn damage_and_death() {
        let mut m = test_monster();
        m.damage(10);
        assert!(!m.is_alive());
        assert_eq!(m.hp(), 0);
    }

    #[test]
    fn abilities_lookup() {
        let def = MonsterDef::by_id(15).unwrap(); // ghoul, has Drain
        let m = Monster::from_def(def, Pos::new(0, 0));
        assert!(m.has_ability(Ability::Drain));
        assert!(!m.has_ability(Ability::FireBreath));
    }

    #[test]
    fn ranged_flag() {
        let def = MonsterDef::by_id(20).unwrap(); // harpy, ranged
        let m = Monster::from_def(def, Pos::new(0, 0));
        assert!(m.is_ranged());
    }

    #[test]
    fn stunned_when_paralyzed() {
        let mut m = test_monster();
        assert!(!m.is_stunned());
        m.statuses.apply(crate::status::Status::Paralyzed, 3);
        assert!(m.is_stunned());
    }

    #[test]
    fn enrage_increases_attack() {
        let mut m = test_monster();
        let base = m.effective_attack();
        m.enraged = true;
        assert_eq!(m.effective_attack(), base + 3);
    }

    #[test]
    fn xp_value_matches_def() {
        let m = test_monster();
        assert_eq!(m.xp_value(), m.def().xp);
    }
}
