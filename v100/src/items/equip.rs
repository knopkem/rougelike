//! Equipment slots and the equipped set.

use super::item::Item;
use serde::{Deserialize, Serialize};

/// The equipment slots a player can fill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Slot {
    Weapon,
    Armor,
    RingLeft,
    RingRight,
    Amulet,
}

impl Slot {
    pub const ALL: [Slot; 5] = [
        Slot::Weapon,
        Slot::Armor,
        Slot::RingLeft,
        Slot::RingRight,
        Slot::Amulet,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Slot::Weapon => "weapon",
            Slot::Armor => "armor",
            Slot::RingLeft => "left ring",
            Slot::RingRight => "right ring",
            Slot::Amulet => "amulet",
        }
    }
}

/// The set of items currently equipped in each slot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<usize>,
    pub armor: Option<usize>,
    pub ring_left: Option<usize>,
    pub ring_right: Option<usize>,
    pub amulet: Option<usize>,
}

impl Equipment {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the inventory index equipped in a slot.
    pub fn get(&self, slot: Slot) -> Option<usize> {
        match slot {
            Slot::Weapon => self.weapon,
            Slot::Armor => self.armor,
            Slot::RingLeft => self.ring_left,
            Slot::RingRight => self.ring_right,
            Slot::Amulet => self.amulet,
        }
    }

    /// Set the inventory index equipped in a slot.
    pub fn set(&mut self, slot: Slot, index: Option<usize>) {
        match slot {
            Slot::Weapon => self.weapon = index,
            Slot::Armor => self.armor = index,
            Slot::RingLeft => self.ring_left = index,
            Slot::RingRight => self.ring_right = index,
            Slot::Amulet => self.amulet = index,
        }
    }

    /// All currently equipped inventory indices.
    pub fn equipped_indices(&self) -> Vec<usize> {
        let mut v = Vec::new();
        for idx in [
            self.weapon,
            self.armor,
            self.ring_left,
            self.ring_right,
            self.amulet,
        ]
        .into_iter()
        .flatten()
        {
            v.push(idx);
        }
        v
    }

    /// Whether a given inventory index is equipped in any slot.
    pub fn is_equipped(&self, index: usize) -> bool {
        self.equipped_indices().contains(&index)
    }

    /// Remove an item from all slots (e.g. when it's consumed or dropped).
    pub fn remove(&mut self, index: usize) {
        if self.weapon == Some(index) {
            self.weapon = None;
        }
        if self.armor == Some(index) {
            self.armor = None;
        }
        if self.ring_left == Some(index) {
            self.ring_left = None;
        }
        if self.ring_right == Some(index) {
            self.ring_right = None;
        }
        if self.amulet == Some(index) {
            self.amulet = None;
        }
    }

    /// The slot an item would occupy, based on its category.
    pub fn slot_for_item(item: &Item) -> Option<Slot> {
        match item.def().category {
            super::item::ItemCategory::Weapon => Some(Slot::Weapon),
            super::item::ItemCategory::Armor => Some(Slot::Armor),
            super::item::ItemCategory::Ring => Some(Slot::RingLeft),
            super::item::ItemCategory::Amulet => Some(Slot::Amulet),
            _ => None,
        }
    }

    /// Total AC bonus from equipped armor and rings.
    pub fn ac_bonus(&self, inventory: &[Item]) -> i32 {
        let mut ac = 0;
        if let Some(idx) = self.armor
            && let Some(item) = inventory.get(idx)
        {
            ac += item.def().ac_bonus + item.enchantment;
        }
        for slot in [Slot::RingLeft, Slot::RingRight] {
            if let Some(idx) = self.get(slot)
                && let Some(item) = inventory.get(idx)
                && item.def().ring_effect == Some(super::item::RingEffect::Protection)
            {
                ac += 2;
            }
        }
        ac
    }

    /// Total attack bonus from equipped weapon.
    pub fn attack_bonus(&self, inventory: &[Item]) -> i32 {
        if let Some(idx) = self.weapon
            && let Some(item) = inventory.get(idx)
        {
            return item.def().attack_bonus + item.enchantment;
        }
        0
    }

    /// The equipped weapon's damage die and bonus, or unarmed defaults.
    pub fn weapon_damage(&self, inventory: &[Item]) -> (u32, i32) {
        if let Some(idx) = self.weapon
            && let Some(item) = inventory.get(idx)
        {
            return (
                item.def().damage_die,
                item.def().damage_bonus + item.enchantment,
            );
        }
        // Unarmed: d4, +0
        (4, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::item::Item;

    #[test]
    fn equipment_starts_empty() {
        let eq = Equipment::new();
        assert!(eq.weapon.is_none());
        assert!(eq.armor.is_none());
        assert!(eq.equipped_indices().is_empty());
    }

    #[test]
    fn equip_and_remove() {
        let mut eq = Equipment::new();
        eq.set(Slot::Weapon, Some(0));
        assert_eq!(eq.get(Slot::Weapon), Some(0));
        assert!(eq.is_equipped(0));
        eq.remove(0);
        assert!(eq.get(Slot::Weapon).is_none());
    }

    #[test]
    fn slot_for_item_categories() {
        let sword = Item::new(0);
        assert_eq!(Equipment::slot_for_item(&sword), Some(Slot::Weapon));
        let armor = Item::new(100);
        assert_eq!(Equipment::slot_for_item(&armor), Some(Slot::Armor));
        let potion = Item::new(200);
        assert_eq!(Equipment::slot_for_item(&potion), None);
    }

    #[test]
    fn ac_bonus_from_armor() {
        let mut eq = Equipment::new();
        let inv = vec![Item::new(101)]; // chain mail, ac 6
        eq.set(Slot::Armor, Some(0));
        assert_eq!(eq.ac_bonus(&inv), 6);
    }

    #[test]
    fn weapon_damage_equipped() {
        let mut eq = Equipment::new();
        let inv = vec![Item::new(1)]; // longsword d8 +1
        eq.set(Slot::Weapon, Some(0));
        assert_eq!(eq.weapon_damage(&inv), (8, 1));
    }

    #[test]
    fn unarmed_default() {
        let eq = Equipment::new();
        let inv: Vec<Item> = vec![];
        assert_eq!(eq.weapon_damage(&inv), (4, 0));
    }
}
