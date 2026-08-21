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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonsterDefData {
    pub name: String,
    pub glyph: char,
    pub tier: u8,
    pub rarity: Rarity,
    pub hp: u8,
    pub xp: u32,
    pub damage_die: u8,
    pub attack: u8,
    pub ac: u8,
    pub speed: u8,
}

impl From<MonsterDef> for MonsterDefData {
    fn from(d: MonsterDef) -> Self {
        Self {
            name: d.name.to_string(),
            glyph: d.glyph,
            tier: d.tier,
            rarity: d.rarity,
            hp: d.hp,
            xp: d.xp,
            damage_die: d.damage_die,
            attack: d.attack,
            ac: d.ac,
            speed: d.speed,
        }
    }
}

impl From<&MonsterDef> for MonsterDefData {
    fn from(d: &MonsterDef) -> Self {
        Self {
            name: d.name.to_string(),
            glyph: d.glyph,
            tier: d.tier,
            rarity: d.rarity,
            hp: d.hp,
            xp: d.xp,
            damage_die: d.damage_die,
            attack: d.attack,
            ac: d.ac,
            speed: d.speed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub def: MonsterDefData,
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

impl Monster {
    pub fn new(def: MonsterDef, pos: (u8, u8)) -> Self {
        let max_hp = def.hp;
        let xp = def.xp;
        Self {
            name: def.name.to_string(),
            def: def.into(),
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

impl Monster {
    pub fn tier(&self) -> u8 {
        self.def.tier
    }
}

/// Spawn a monster for the given depth.
pub fn spawn_monster(rng: &mut Rng, depth: u8, endless: bool) -> Monster {
    let tier = tier_for_depth(depth);
    let defs: Vec<&MonsterDef> = crate::data::monsters::MONSTERS
        .iter()
        .filter(|d| d.tier == tier)
        .collect();
    let def = match rng.pick(&defs) {
        Some(d) => d.clone(),
        None => crate::data::monsters::MONSTERS[0].clone(),
    };
    let mut m = Monster::new(def, (3, 3));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::monsters::MONSTERS;

    fn sample() -> Monster {
        let mut m = Monster::new(MONSTERS[4].clone(), (12, 34));
        m.name = "Foul hobgoblin".to_string();
        m.hp = 4;
        m.dead = true;
        m.is_unique = true;
        m.ability_cooldown = 3;
        m
    }

    #[test]
    fn spawn_picks_varied_species_within_tier() {
        let mut rng = Rng::new(12345);
        let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for _ in 0..100 {
            let m = spawn_monster(&mut rng, 1, false);
            assert_eq!(m.def.tier, 1);
            names.insert(m.def.name.clone());
        }
        assert!(
            names.len() > 1,
            "tier 1 spawns should include multiple species, got {names:?}"
        );
    }

    #[test]
    fn spawn_keeps_tier_correct() {
        let mut rng = Rng::new(7);
        for depth in [1u8, 7, 12, 17, 22, 40] {
            for _ in 0..20 {
                let m = spawn_monster(&mut rng, depth, false);
                assert_eq!(m.def.tier, tier_for_depth(depth));
            }
        }
    }

    #[test]
    fn endless_scaling_applies_beyond_depth_25() {
        let non_unique = |endless: bool| {
            let mut rng = Rng::new(99);
            loop {
                let m = spawn_monster(&mut rng, 30, endless);
                if !m.is_unique {
                    return m;
                }
            }
        };
        // Endless: +5 HP and +10 XP per depth past 25.
        let m = non_unique(true);
        assert_eq!(m.max_hp, m.def.hp + 25, "endless HP scaling");
        assert_eq!(m.hp, m.max_hp);
        assert_eq!(m.xp, m.def.xp + 50, "endless XP scaling");
        // Not endless: base stats unchanged beyond the cap.
        let m = non_unique(false);
        assert_eq!(m.max_hp, m.def.hp, "no scaling without endless mode");
        assert_eq!(m.xp, m.def.xp, "no scaling without endless mode");
    }

    #[test]
    fn monster_survives_json_roundtrip() {
        let m = sample();
        let json = serde_json::to_string(&m).unwrap();
        let back: Monster = serde_json::from_str(&json).unwrap();
        assert_eq!(back.def, m.def);
        assert_eq!(back.name, m.name);
        assert_eq!(back.pos, m.pos);
        assert_eq!(back.hp, m.hp);
        assert_eq!(back.max_hp, m.max_hp);
        assert_eq!(back.xp, m.xp);
        assert_eq!(back.dead, m.dead);
        assert_eq!(back.is_unique, m.is_unique);
        assert_eq!(back.is_boss, m.is_boss);
        assert_eq!(back.ability_cooldown, m.ability_cooldown);
    }
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
