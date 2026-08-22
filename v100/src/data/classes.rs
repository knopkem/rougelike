//! Player races and classes: base stats, derived bonuses, and starting kits.

use serde::{Deserialize, Serialize};

/// The six core attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes {
    pub str: u32,
    pub dex: u32,
    pub con: u32,
    pub int: u32,
    pub wis: u32,
    pub cha: u32,
}

impl Attributes {
    pub const fn new(str: u32, dex: u32, con: u32, int: u32, wis: u32, cha: u32) -> Self {
        Self {
            str,
            dex,
            con,
            int,
            wis,
            cha,
        }
    }

    /// The attribute at the given index (0=STR, 1=DEX, 2=CON, 3=INT, 4=WIS, 5=CHA).
    pub fn get(&self, idx: u8) -> u32 {
        match idx {
            0 => self.str,
            1 => self.dex,
            2 => self.con,
            3 => self.int,
            4 => self.wis,
            _ => self.cha,
        }
    }

    /// Increase the attribute at the given index by `amount`.
    pub fn inc(&mut self, idx: u8, amount: u32) {
        match idx {
            0 => self.str += amount,
            1 => self.dex += amount,
            2 => self.con += amount,
            3 => self.int += amount,
            4 => self.wis += amount,
            _ => self.cha += amount,
        }
    }

    /// Set the attribute at the given index to an exact value (clamped to 1..=30).
    pub fn set(&mut self, idx: u8, value: u32) {
        let v = value.clamp(1, 30);
        match idx {
            0 => self.str = v,
            1 => self.dex = v,
            2 => self.con = v,
            3 => self.int = v,
            4 => self.wis = v,
            _ => self.cha = v,
        }
    }

    pub fn name(idx: u8) -> &'static str {
        match idx {
            0 => "STR",
            1 => "DEX",
            2 => "CON",
            3 => "INT",
            4 => "WIS",
            _ => "CHA",
        }
    }
}

/// A playable race.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    Halfling,
}

impl Race {
    pub const ALL: [Race; 4] = [Race::Human, Race::Elf, Race::Dwarf, Race::Halfling];

    pub fn name(self) -> &'static str {
        match self {
            Race::Human => "Human",
            Race::Elf => "Elf",
            Race::Dwarf => "Dwarf",
            Race::Halfling => "Halfling",
        }
    }

    /// Attribute bonuses for this race.
    pub fn bonuses(self) -> (i32, i32, i32, i32, i32, i32) {
        match self {
            Race::Human => (1, 1, 1, 1, 1, 1),
            Race::Elf => (0, 2, 0, 2, 0, 0),
            Race::Dwarf => (1, 0, 2, 0, 0, 0),
            Race::Halfling => (0, 2, 1, 0, 0, 0),
        }
    }

    /// Base armor-class bonus.
    pub fn ac_bonus(self) -> i32 {
        match self {
            Race::Dwarf => 2,
            _ => 0,
        }
    }

    /// Base darkvision radius bonus (added to the theme radius).
    pub fn darkvision_bonus(self) -> u32 {
        match self {
            Race::Elf => 2,
            _ => 0,
        }
    }

    /// Base stealth bonus.
    pub fn stealth_bonus(self) -> i32 {
        match self {
            Race::Halfling => 3,
            Race::Elf => 1,
            _ => 0,
        }
    }

    /// Base critical-chance bonus (percent).
    pub fn crit_bonus(self) -> u32 {
        match self {
            Race::Halfling => 2,
            _ => 0,
        }
    }

    /// Fire resistance (Dwarf).
    pub fn fire_resist(self) -> bool {
        self == Race::Dwarf
    }
}

/// A playable class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    Warrior,
    Thief,
    Ranger,
    Mage,
    Cleric,
}

impl Class {
    pub const ALL: [Class; 5] = [
        Class::Warrior,
        Class::Thief,
        Class::Ranger,
        Class::Mage,
        Class::Cleric,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Class::Warrior => "Warrior",
            Class::Thief => "Thief",
            Class::Ranger => "Ranger",
            Class::Mage => "Mage",
            Class::Cleric => "Cleric",
        }
    }

    /// Base attributes for this class (before race bonuses).
    pub fn base_attributes(self) -> Attributes {
        match self {
            Class::Warrior => Attributes::new(14, 10, 13, 8, 9, 10),
            Class::Thief => Attributes::new(9, 15, 11, 10, 9, 10),
            Class::Ranger => Attributes::new(11, 13, 11, 10, 10, 9),
            Class::Mage => Attributes::new(7, 11, 9, 16, 12, 8),
            Class::Cleric => Attributes::new(10, 9, 12, 11, 15, 10),
        }
    }

    /// Base maximum hit points.
    pub fn base_hp(self) -> u32 {
        match self {
            Class::Warrior => 30,
            Class::Thief => 22,
            Class::Ranger => 26,
            Class::Mage => 18,
            Class::Cleric => 24,
        }
    }

    /// Base maximum energy points.
    pub fn base_ep(self) -> u32 {
        match self {
            Class::Warrior => 10,
            Class::Thief => 12,
            Class::Ranger => 12,
            Class::Mage => 24,
            Class::Cleric => 20,
        }
    }

    /// Lockpicking skill (Thief is best).
    pub fn lockpick(self) -> u32 {
        match self {
            Class::Thief => 60,
            Class::Ranger => 20,
            _ => 5,
        }
    }

    /// Stealth bonus.
    pub fn stealth(self) -> i32 {
        match self {
            Class::Thief => 4,
            Class::Ranger => 2,
            _ => 0,
        }
    }

    /// Whether this class can use wands and scrolls effectively.
    pub fn magic_user(self) -> bool {
        matches!(self, Class::Mage | Class::Cleric)
    }
}

/// A starting-kit item reference (item id from the catalog).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KitItem {
    pub item_id: u32,
    pub quantity: u32,
}

/// The starting kit for a class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StartingKit {
    pub items: &'static [KitItem],
    pub gold: u32,
}

impl Class {
    /// The starting kit for this class. Item ids reference the item catalog.
    ///
    /// Item id conventions (see `items::catalog`):
    /// - 0 = shortsword, 1 = longsword, 2 = dagger
    /// - 100 = leather armor, 101 = chain mail
    /// - 200 = small healing potion
    /// - 300 = wand of fire bolt, 301 = wand of healing
    /// - 400 = scroll of identify
    /// - 600 = trail rations
    pub fn starting_kit(self) -> StartingKit {
        match self {
            Class::Warrior => StartingKit {
                items: &[
                    KitItem {
                        item_id: 1,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 101,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 200,
                        quantity: 2,
                    },
                    KitItem {
                        item_id: 600,
                        quantity: 3,
                    },
                ],
                gold: 20,
            },
            Class::Thief => StartingKit {
                items: &[
                    KitItem {
                        item_id: 2,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 100,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 200,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 600,
                        quantity: 3,
                    },
                    KitItem {
                        item_id: 400,
                        quantity: 1,
                    },
                ],
                gold: 30,
            },
            Class::Ranger => StartingKit {
                items: &[
                    KitItem {
                        item_id: 0,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 100,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 200,
                        quantity: 2,
                    },
                    KitItem {
                        item_id: 600,
                        quantity: 3,
                    },
                ],
                gold: 25,
            },
            Class::Mage => StartingKit {
                items: &[
                    KitItem {
                        item_id: 2,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 300,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 301,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 200,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 600,
                        quantity: 2,
                    },
                ],
                gold: 15,
            },
            Class::Cleric => StartingKit {
                items: &[
                    KitItem {
                        item_id: 0,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 100,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 301,
                        quantity: 1,
                    },
                    KitItem {
                        item_id: 200,
                        quantity: 2,
                    },
                    KitItem {
                        item_id: 600,
                        quantity: 2,
                    },
                ],
                gold: 20,
            },
        }
    }
}

/// Compute the final attributes for a race + class combination.
pub fn final_attributes(race: Race, class: Class) -> Attributes {
    let base = class.base_attributes();
    let (s, d, c, i, w, ch) = race.bonuses();
    Attributes::new(
        base.str.saturating_add(s.max(0) as u32),
        base.dex.saturating_add(d.max(0) as u32),
        base.con.saturating_add(c.max(0) as u32),
        base.int.saturating_add(i.max(0) as u32),
        base.wis.saturating_add(w.max(0) as u32),
        base.cha.saturating_add(ch.max(0) as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_races_and_classes_exist() {
        assert_eq!(Race::ALL.len(), 4);
        assert_eq!(Class::ALL.len(), 5);
    }

    #[test]
    fn human_is_versatile() {
        let (s, d, c, i, w, ch) = Race::Human.bonuses();
        assert_eq!((s, d, c, i, w, ch), (1, 1, 1, 1, 1, 1));
    }

    #[test]
    fn dwarf_has_ac_and_fire_resist() {
        assert_eq!(Race::Dwarf.ac_bonus(), 2);
        assert!(Race::Dwarf.fire_resist());
        assert!(!Race::Elf.fire_resist());
    }

    #[test]
    fn elf_has_darkvision() {
        assert_eq!(Race::Elf.darkvision_bonus(), 2);
        assert_eq!(Race::Human.darkvision_bonus(), 0);
    }

    #[test]
    fn thief_is_best_at_lockpicking() {
        assert!(Class::Thief.lockpick() > Class::Warrior.lockpick());
        assert!(Class::Thief.lockpick() > Class::Mage.lockpick());
    }

    #[test]
    fn final_attributes_apply_race_bonuses() {
        let human_warrior = final_attributes(Race::Human, Class::Warrior);
        let base = Class::Warrior.base_attributes();
        assert_eq!(human_warrior.str, base.str + 1);
        assert_eq!(human_warrior.dex, base.dex + 1);
    }

    #[test]
    fn every_class_has_a_kit() {
        for class in Class::ALL {
            let kit = class.starting_kit();
            assert!(!kit.items.is_empty());
        }
    }
}
