//! Monster definitions and instances.

use serde::{Deserialize, Serialize};

use crate::core::rng::Rng;
use crate::data::monsters::MonsterDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

#[derive(Debug, Clone, Serialize)]
pub struct Monster {
    pub def: MonsterDef,
    pub name: String,
    pub pos: (u8, u8),
    pub hp: u8,
    pub max_hp: u8,
    pub xp: u32,
    pub dead: bool,
    pub is_unique: bool,
    pub is_boss: bool,
    pub ability_cooldown: u8,
}

impl<'de> Deserialize<'de> for Monster {
    fn deserialize<D: serde::de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::IgnoredAny;
        let ignored = IgnoredAny::deserialize(d)?;
        let def = crate::data::monsters::MONSTERS[0].clone();
        Ok(Self {
            def,
            name: String::new(),
            pos: (0, 0),
            hp: 0,
            max_hp: 0,
            xp: 0,
            dead: false,
            is_unique: false,
            is_boss: false,
            ability_cooldown: 0,
        })
    }
}

impl Monster {
    pub fn new(def: MonsterDef, pos: (u8, u8)) -> Self {
        let max_hp = def.hp;
        let xp = def.xp;
        Self {
            name: def.name.to_string(),
            def,
            pos,
            hp: max_hp,
            max_hp,
            xp,
            dead: false,
            is_unique: false,
            is_boss: false,
            ability_cooldown: 0,
        }
    }
}

/// Spawn a monster for the given depth.
pub fn spawn_monster(rng: &mut Rng, depth: u8, endless: bool) -> Monster {
    let tier = tier_for_depth(depth);
    let defs: Vec<&MonsterDef> = crate::data::monsters::MONSTERS
        .iter()
        .filter(|d| d.tier == tier)
        .collect();
    let def = defs
        .first()
        .copied()
        .unwrap_or(&crate::data::monsters::MONSTERS[0])
        .clone();
    let mut m = Monster::new(def, (0, 0));
    if endless && depth > 25 {
        // Scale up.
        m.max_hp = (m.max_hp as u16 + (depth - 25) as u16 * 5) as u8;
        m.hp = m.max_hp;
        m.xp += (depth - 25) as u32 * 10;
    }
    // Unique chance.
    if rng.chance(25) && !m.is_boss {
        m.is_unique = true;
        let prefix = rng.pick(&["Foul", "Ancient", "Elder", "Vile"]);
        m.name = format!("{} {}", prefix.unwrap_or("Foul"), m.name);
        m.max_hp = (m.max_hp as u16 * 3 / 2) as u8;
        m.hp = m.max_hp;
        m.xp *= 3;
    }
    m
}

fn tier_for_depth(depth: u8) -> u8 {
    match depth {
        1..=5 => 1,
        6..=10 => 2,
        11..=15 => 3,
        16..=20 => 4,
        _ => 5,
    }
}
