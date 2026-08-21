//! Classes: 5 classes with distinct stats and kits.

use crate::items::catalog;
use crate::items::item::{
    ArmorKind, FoodKind, Item, PotionKind, ScrollKind, ShieldKind, WandKind, WeaponKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassId {
    Warrior,
    Thief,
    Ranger,
    Mage,
    Cleric,
}

impl ClassId {
    pub const ALL: [ClassId; 5] = [
        ClassId::Warrior,
        ClassId::Thief,
        ClassId::Ranger,
        ClassId::Mage,
        ClassId::Cleric,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ClassId::Warrior => "Warrior",
            ClassId::Thief => "Thief",
            ClassId::Ranger => "Ranger",
            ClassId::Mage => "Mage",
            ClassId::Cleric => "Cleric",
        }
    }

    pub fn desc(&self) -> &'static str {
        match self {
            ClassId::Warrior => "Heavy hitter, high HP.",
            ClassId::Thief => "Stealth, lockpicking, crits.",
            ClassId::Ranger => "Ranged combat, balanced.",
            ClassId::Mage => "High EP, magic power.",
            ClassId::Cleric => "Healing, wisdom, solid HP.",
        }
    }

    pub fn from_name(name: &str) -> Option<ClassId> {
        Some(match name {
            "Warrior" => ClassId::Warrior,
            "Thief" => ClassId::Thief,
            "Ranger" => ClassId::Ranger,
            "Mage" => ClassId::Mage,
            "Cleric" => ClassId::Cleric,
            _ => return None,
        })
    }

    /// The class's starting kit: weapon is wielded, armor is worn, the rest
    /// is carried in the inventory.
    pub const fn kit(&self) -> Kit {
        match self {
            ClassId::Warrior => Kit {
                weapon: WeaponKind::Longsword,
                armor: ArmorKind::Chainmail,
                items: &[
                    KitEntry::Shield(ShieldKind::Small),
                    KitEntry::Potion(PotionKind::Healing(true)),
                    KitEntry::Potion(PotionKind::Energy),
                    KitEntry::Potion(PotionKind::Energy),
                    KitEntry::Food(FoodKind::TrailRations),
                    KitEntry::Food(FoodKind::TrailRations),
                ],
            },
            ClassId::Thief => Kit {
                weapon: WeaponKind::Dagger,
                armor: ArmorKind::Leather,
                items: &[
                    KitEntry::Potion(PotionKind::Healing(true)),
                    KitEntry::Potion(PotionKind::Energy),
                    KitEntry::Scroll(ScrollKind::Teleport),
                    KitEntry::Food(FoodKind::TrailRations),
                    KitEntry::Food(FoodKind::TrailRations),
                ],
            },
            ClassId::Ranger => Kit {
                weapon: WeaponKind::Spear,
                armor: ArmorKind::Leather,
                items: &[
                    KitEntry::Potion(PotionKind::Healing(true)),
                    KitEntry::Potion(PotionKind::Energy),
                    KitEntry::Potion(PotionKind::CurePoison),
                    KitEntry::Food(FoodKind::TrailRations),
                    KitEntry::Food(FoodKind::TrailRations),
                ],
            },
            ClassId::Mage => Kit {
                weapon: WeaponKind::Dagger,
                armor: ArmorKind::Leather,
                items: &[
                    KitEntry::Wand(WandKind::Lightning),
                    KitEntry::Wand(WandKind::FireBolt),
                    KitEntry::Potion(PotionKind::Energy),
                    KitEntry::Potion(PotionKind::Energy),
                    KitEntry::Food(FoodKind::TrailRations),
                ],
            },
            ClassId::Cleric => Kit {
                weapon: WeaponKind::Mace,
                armor: ArmorKind::Leather,
                items: &[
                    KitEntry::Wand(WandKind::Healing),
                    KitEntry::Potion(PotionKind::Healing(true)),
                    KitEntry::Potion(PotionKind::CurePoison),
                    KitEntry::Food(FoodKind::TrailRations),
                    KitEntry::Food(FoodKind::TrailRations),
                ],
            },
        }
    }
}

/// The class's starting kit. The weapon is wielded and the armor is worn on a
/// new player; everything in `items` is carried in the inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kit {
    pub weapon: WeaponKind,
    pub armor: ArmorKind,
    pub items: &'static [KitEntry],
}

/// One carried item of a starting kit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KitEntry {
    Weapon(WeaponKind),
    Armor(ArmorKind),
    Shield(ShieldKind),
    Wand(WandKind),
    Potion(PotionKind),
    Scroll(ScrollKind),
    Food(FoodKind),
}

impl Kit {
    /// (wielded weapon, worn armor, carried inventory) for a new player.
    pub fn equip(&self) -> (Item, Item, Vec<Item>) {
        (
            KitEntry::Weapon(self.weapon).make(),
            KitEntry::Armor(self.armor).make(),
            self.items.iter().copied().map(|e| e.make()).collect(),
        )
    }
}

impl KitEntry {
    /// Build the concrete kit item: +0 enchant, never cursed, already
    /// identified (the player knows what they were given).
    pub fn make(self) -> Item {
        let item = match self {
            KitEntry::Weapon(k) => catalog::make_weapon(k, 0, false),
            KitEntry::Armor(k) => catalog::make_armor(k, 0, false),
            KitEntry::Shield(k) => catalog::make_shield(k, 0, false),
            KitEntry::Wand(k) => catalog::make_wand(k, 0),
            KitEntry::Potion(k) => catalog::make_potion(k),
            KitEntry::Scroll(k) => catalog::make_scroll(k),
            KitEntry::Food(k) => catalog::make_food(k),
        };
        Item {
            identified: true,
            ..item
        }
    }
}

/// Kit selection: a pure function of the class name string.
pub fn kit_for(class: &str) -> Option<Kit> {
    ClassId::from_name(class).map(|c| c.kit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_name_round_trips_all_classes() {
        for c in ClassId::ALL {
            assert_eq!(ClassId::from_name(c.name()), Some(c));
        }
        assert_eq!(ClassId::from_name("Bard"), None);
        assert_eq!(kit_for(""), None);
    }

    #[test]
    fn every_class_has_a_kit() {
        for c in ClassId::ALL {
            let kit = c.kit();
            assert!(!kit.items.is_empty(), "{} kit must carry items", c.name());
            for entry in kit.items {
                let item = entry.make();
                assert!(item.identified, "kit items must be identified");
                assert!(!item.cursed, "kit items must not be cursed");
                assert_eq!(item.enchant, 0, "kit items must be +0");
            }
        }
    }

    #[test]
    fn kits_match_the_table() {
        let w = ClassId::Warrior.kit();
        assert_eq!(w.weapon, WeaponKind::Longsword);
        assert_eq!(w.armor, ArmorKind::Chainmail);
        assert_eq!(w.items.len(), 6);

        let t = ClassId::Thief.kit();
        assert_eq!(t.weapon, WeaponKind::Dagger);
        assert_eq!(t.armor, ArmorKind::Leather);

        let m = ClassId::Mage.kit();
        assert_eq!(m.weapon, WeaponKind::Dagger);
        assert!(m.items
            .iter()
            .any(|e| *e == KitEntry::Wand(WandKind::Lightning)));

        let cl = ClassId::Cleric.kit();
        assert_eq!(cl.weapon, WeaponKind::Mace);
        assert!(cl.items
            .iter()
            .any(|e| *e == KitEntry::Wand(WandKind::Healing)));
    }
}
