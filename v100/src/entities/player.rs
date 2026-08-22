//! The player: attributes, inventory, equipment, and progression.

use crate::core::events::Pos;
use crate::data::classes::{Attributes, Class, Race};
use crate::entities::entity::Entity;
use crate::items::equip::Equipment;
use crate::items::item::Item;
use crate::status::StatusSet;
use serde::{Deserialize, Serialize};

/// The player character.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    /// Shared combat/position fields.
    pub entity: Entity,
    /// The player's race.
    pub race: Race,
    /// The player's class.
    pub class: Class,
    /// The six attributes.
    pub attributes: Attributes,
    /// Character level (1-based).
    pub level: u32,
    /// Experience points.
    pub xp: u32,
    /// Energy points (for magic).
    pub ep: u32,
    /// Maximum energy points.
    pub max_ep: u32,
    /// Hunger (0 = starving, 1200 = full).
    pub hunger: u32,
    /// Gold carried.
    pub gold: u32,
    /// Inventory of items.
    pub inventory: Vec<Item>,
    /// Equipment slots.
    pub equipment: Equipment,
    /// Active status effects.
    pub statuses: StatusSet,
    /// Total monsters killed.
    pub kills: u32,
    /// Luck (from rings, etc.).
    pub luck: i32,
}

impl Player {
    /// Create a new player from a race and class.
    pub fn new(pos: Pos, race: Race, class: Class) -> Self {
        let attributes = crate::data::classes::final_attributes(race, class);
        let base_hp = class.base_hp() + attributes.con / 2;
        let base_ep = class.base_ep();
        let entity = Entity::new(pos, base_hp, 10, 0, 4, 0);
        let mut player = Self {
            entity,
            race,
            class,
            attributes,
            level: 1,
            xp: 0,
            ep: base_ep,
            max_ep: base_ep,
            hunger: 1200,
            gold: 0,
            inventory: Vec::new(),
            equipment: Equipment::new(),
            statuses: StatusSet::new(),
            kills: 0,
            luck: 0,
        };
        player.recompute_ac();
        player
    }

    /// The player's position.
    pub fn pos(&self) -> Pos {
        self.entity.pos
    }

    /// Whether the player is alive.
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

    /// Recompute the player's AC from attributes, race, and equipment.
    pub fn recompute_ac(&mut self) {
        let base = 10 + self.attributes.dex as i32 / 2;
        let race_bonus = self.race.ac_bonus();
        let equip_bonus = self.equipment.ac_bonus(&self.inventory);
        let berserk_penalty = if self.statuses.has(crate::status::Status::Berserk) {
            -2
        } else {
            0
        };
        self.entity.ac = base + race_bonus + equip_bonus + berserk_penalty;
    }

    /// Recompute the player's attack bonus from equipment.
    pub fn recompute_attack(&mut self) {
        let equip_bonus = self.equipment.attack_bonus(&self.inventory);
        let berserk_bonus = if self.statuses.has(crate::status::Status::Berserk) {
            3
        } else {
            0
        };
        self.entity.attack = equip_bonus + berserk_bonus;
    }

    /// The weapon damage (die, bonus) for the currently equipped weapon.
    pub fn weapon_damage(&self) -> (u32, i32) {
        self.equipment.weapon_damage(&self.inventory)
    }

    /// Apply damage to the player.
    pub fn damage(&mut self, amount: u32) -> u32 {
        self.entity.damage(amount)
    }

    /// Heal the player.
    pub fn heal(&mut self, amount: u32) -> u32 {
        self.entity.heal(amount)
    }

    /// Move the player.
    pub fn move_to(&mut self, pos: Pos) {
        self.entity.move_to(pos);
    }

    /// Add an item to the inventory, returning its index.
    pub fn add_item(&mut self, item: Item) -> usize {
        self.inventory.push(item);
        self.inventory.len() - 1
    }

    /// Remove an item from the inventory by index, returning it.
    pub fn remove_item(&mut self, index: usize) -> Option<Item> {
        if index < self.inventory.len() {
            let item = self.inventory.remove(index);
            self.equipment.remove(index);
            // Re-index equipment slots that referenced higher indices.
            self.reindex_equipment(index);
            Some(item)
        } else {
            None
        }
    }

    /// Re-index equipment slots after an item removal.
    fn reindex_equipment(&mut self, removed_index: usize) {
        let adjust = |slot: &mut Option<usize>| {
            if let Some(idx) = *slot {
                if idx == removed_index {
                    *slot = None;
                } else if idx > removed_index {
                    *slot = Some(idx - 1);
                }
            }
        };
        adjust(&mut self.equipment.weapon);
        adjust(&mut self.equipment.armor);
        adjust(&mut self.equipment.ring_left);
        adjust(&mut self.equipment.ring_right);
        adjust(&mut self.equipment.amulet);
    }

    /// Gain experience, handling level-ups. Returns the number of levels gained.
    pub fn gain_xp(&mut self, amount: u32) -> u32 {
        self.xp += amount;
        let mut levels_gained = 0;
        while self.xp >= self.xp_for_next_level() {
            self.xp -= self.xp_for_next_level();
            self.level += 1;
            levels_gained += 1;
            self.on_level_up();
        }
        levels_gained
    }

    /// XP required to reach the next level.
    pub fn xp_for_next_level(&self) -> u32 {
        // Tunable XP curve: level N requires N^2 * 20 XP.
        self.level * self.level * 20
    }

    /// Apply the effects of leveling up.
    fn on_level_up(&mut self) {
        // Gain HP and EP based on class and CON.
        let hp_gain = 3 + self.attributes.con / 4;
        let ep_gain = if self.class.magic_user() { 2 } else { 1 };
        self.entity.max_hp += hp_gain;
        self.entity.hp = self.entity.max_hp; // full heal on level up
        self.max_ep += ep_gain;
        self.ep = self.max_ep;
    }

    /// Spend energy points, returning false if insufficient.
    pub fn spend_ep(&mut self, amount: u32) -> bool {
        if self.ep >= amount {
            self.ep -= amount;
            true
        } else {
            false
        }
    }

    /// Recover energy points.
    pub fn recover_ep(&mut self, amount: u32) {
        self.ep = (self.ep + amount).min(self.max_ep);
    }

    /// Decrease hunger by one turn's worth.
    pub fn tick_hunger(&mut self) {
        self.hunger = self.hunger.saturating_sub(1);
        // Starving (hunger == 0) causes damage.
        if self.hunger == 0 {
            self.entity.damage(1);
        }
    }

    /// Eat food, restoring hunger.
    pub fn eat(&mut self, nutrition: u32) {
        self.hunger = (self.hunger + nutrition).min(1200);
    }

    /// The player's total carrying weight.
    pub fn carrying_weight(&self) -> u32 {
        self.inventory.iter().map(|i| i.total_weight()).sum()
    }

    /// The player's carrying capacity based on STR.
    pub fn carrying_capacity(&self) -> u32 {
        50 + self.attributes.str * 5
    }

    /// Whether the player is overencumbered.
    pub fn is_overencumbered(&self) -> bool {
        self.carrying_weight() > self.carrying_capacity()
    }

    /// The player's to-hit chance (percent) against a target AC.
    pub fn to_hit_chance(&self, target_ac: i32) -> u32 {
        let base = 50;
        let dex_bonus = self.attributes.dex as i32 * 2;
        let attack = self.entity.attack;
        let chance = base + dex_bonus + attack - target_ac;
        chance.clamp(5, 95) as u32
    }

    /// The player's critical chance (percent).
    pub fn crit_chance(&self) -> u32 {
        let base = 5;
        let race_bonus = self.race.crit_bonus();
        let luck_bonus = self.luck.max(0) as u32;
        base + race_bonus + luck_bonus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_player() -> Player {
        Player::new(Pos::new(10, 10), Race::Human, Class::Warrior)
    }

    #[test]
    fn new_player_is_alive() {
        let p = test_player();
        assert!(p.is_alive());
        assert_eq!(p.level, 1);
        assert_eq!(p.hunger, 1200);
    }

    #[test]
    fn player_has_attributes() {
        let p = test_player();
        assert!(p.attributes.str > 0);
        assert!(p.attributes.dex > 0);
    }

    #[test]
    fn damage_and_heal() {
        let mut p = test_player();
        let max = p.max_hp();
        p.damage(5);
        assert_eq!(p.hp(), max - 5);
        p.heal(5);
        assert_eq!(p.hp(), max);
    }

    #[test]
    fn add_and_remove_item() {
        let mut p = test_player();
        let idx = p.add_item(Item::new(0));
        assert_eq!(idx, 0);
        assert_eq!(p.inventory.len(), 1);
        let removed = p.remove_item(0);
        assert!(removed.is_some());
        assert!(p.inventory.is_empty());
    }

    #[test]
    fn gain_xp_levels_up() {
        let mut p = test_player();
        let before = p.level;
        // Give enough XP to level up.
        p.gain_xp(p.xp_for_next_level());
        assert_eq!(p.level, before + 1);
    }

    #[test]
    fn hunger_decreases() {
        let mut p = test_player();
        let h = p.hunger;
        p.tick_hunger();
        assert_eq!(p.hunger, h - 1);
    }

    #[test]
    fn eating_restores_hunger() {
        let mut p = test_player();
        p.hunger = 100;
        p.eat(200);
        assert_eq!(p.hunger, 300);
    }

    #[test]
    fn to_hit_chance_clamped() {
        let p = test_player();
        assert!(p.to_hit_chance(0) >= 5);
        assert!(p.to_hit_chance(100) <= 95);
    }

    #[test]
    fn equipment_reindex_on_remove() {
        let mut p = test_player();
        // Add weapon and armor.
        p.add_item(Item::new(1)); // longsword at index 0
        p.add_item(Item::new(101)); // chain mail at index 1
        p.equipment.set(crate::items::equip::Slot::Weapon, Some(0));
        p.equipment.set(crate::items::equip::Slot::Armor, Some(1));
        // Remove the weapon (index 0); armor should shift to index 0.
        p.remove_item(0);
        assert_eq!(p.equipment.armor, Some(0));
        assert!(p.equipment.weapon.is_none());
    }
}
