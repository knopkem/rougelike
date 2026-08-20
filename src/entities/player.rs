//! Player: stats, inventory, equipment, hunger, XP, status.

use serde::{Deserialize, Serialize};

use crate::items::item::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub race: String,
    pub class: String,
    pub pos: (u8, u8),
    pub level: u8,
    pub xp: u32,
    pub hp: u8,
    pub max_hp: u8,
    pub ep: u8,
    pub max_ep: u8,
    pub   hunger: u16,
    pub gold: u32,
    pub kills: u32,
    /// Attributes.
    pub str: u8,
    pub dex: u8,
    pub con: u8,
    pub int: u8,
    pub wis: u8,
    pub cha: u8,
    pub inventory: Vec<Item>,
    pub wielded: Option<Item>,
    pub armor: Option<Item>,
    pub rings: Vec<Item>,
}

impl Player {
    pub fn new(name: &str, race: &str, class: &str) -> Self {
        let mut str = 8;
        let mut dex = 8;
        let mut con = 8;
        let mut int = 8;
        let mut wis = 8;
        let mut cha = 8;
        let mut max_hp = 30;
        let mut max_ep = 10;

        // Race modifiers.
        match race {
            "Human" => {
                str += 1;
                dex += 1;
                con += 1;
                int += 1;
                wis += 1;
                cha += 1;
            }
            "Elf" => {
                dex += 2;
                int += 2;
            }
            "Dwarf" => {
                con += 2;
            }
            "Halfling" => {
                dex += 2;
                con += 1;
            }
            _ => {}
        }
        // Class modifiers.
        match class {
            "Warrior" => {
                str += 3;
                con += 1;
                max_hp += 10;
            }
            "Thief" => {
                dex += 3;
            }
            "Ranger" => {
                dex += 2;
                con += 1;
            }
            "Mage" => {
                int += 4;
                max_ep += 10;
            }
            "Cleric" => {
                wis += 3;
                max_hp += 5;
            }
            _ => {}
        }

        Self {
            name: name.to_string(),
            race: race.to_string(),
            class: class.to_string(),
            pos: (40, 12),
            level: 1,
            xp: 0,
            hp: max_hp,
            max_hp,
            ep: max_ep,
            max_ep,
            hunger: 1000,
            gold: 10,
            kills: 0,
            str,
            dex,
            con,
            int,
            wis,
            cha,
            inventory: Vec::new(),
            wielded: None,
            armor: None,
            rings: Vec::new(),
        }
    }

    pub fn to_hit(&self) -> u64 {
        let base = 50 + self.dex as u64 * 2;
        let weapon_bonus = self
            .wielded
            .as_ref()
            .map(|w| w.enchant as u64)
            .unwrap_or(0);
        base + weapon_bonus
    }

    pub fn ac(&self) -> u64 {
        let base = 10 + self.dex as u64 / 2;
        let armor = self
            .armor
            .as_ref()
            .map(|a| a.defense as u64)
            .unwrap_or(0);
        base + armor
    }

    pub fn crit_chance(&self) -> u64 {
        5 + self.dex as u64 / 2
    }

    pub fn darkvision_radius(&self) -> u8 {
        let base = 8;
        let bonus = if self.race == "Elf" { 2 } else { 0 };
        base + bonus as u8
    }

    /// XP needed for next level.
    pub fn xp_next(&self) -> u32 {
        100 * self.level as u32
    }

    pub fn gain_xp(&mut self, amount: u32) -> bool {
        self.xp += amount;
        if self.xp >= self.xp_next() {
            self.xp -= self.xp_next();
            self.level += 1;
            self.max_hp += 2;
            self.max_ep += 2;
            self.hp = self.max_hp;
            self.ep = self.max_ep;
            true
        } else {
            false
        }
    }
}
