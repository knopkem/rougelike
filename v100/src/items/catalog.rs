//! The static item catalog: ~65 templates across all categories.
//!
//! Id scheme:
//! - Weapons: 0-99
//! - Armor: 100-199
//! - Potions: 200-299
//! - Wands: 300-399
//! - Scrolls: 400-499
//! - Rings: 500-599
//! - Food: 600-699
//! - Amulets: 700-799
//! - Misc: 800-899

use super::item::{ItemCategory, ItemDef, PotionEffect, RingEffect, ScrollEffect, WandEffect};
use crate::core::color::Color;

/// Build an `ItemDef` with sensible defaults, overriding only the given fields.
macro_rules! item {
    (
        id: $id:expr,
        name: $name:expr,
        glyph: $glyph:expr,
        color: $color:expr,
        category: $cat:expr,
        $( $field:ident : $val:expr ),* $(,)?
    ) => {
        ItemDef::base($id, $name, $glyph, $color, $cat)
            $( .$field($val) )*
    };
}

/// The full catalog of item definitions.
pub const ALL: &[ItemDef] = &[
    // ---- Weapons (0-99) ----
    item! { id: 0, name: "shortsword", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 6, damage_bonus: 0, attack_bonus: 2, base_value: 25, weight: 2, tier: 1 },
    item! { id: 1, name: "longsword", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 8, damage_bonus: 1, attack_bonus: 3, base_value: 50, weight: 3, tier: 1 },
    item! { id: 2, name: "dagger", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 4, damage_bonus: 0, attack_bonus: 4, base_value: 15, weight: 1, tier: 1 },
    item! { id: 3, name: "mace", glyph: '+', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 8, damage_bonus: 0, attack_bonus: 1, base_value: 35, weight: 4, tier: 1 },
    item! { id: 4, name: "warhammer", glyph: '+', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 10, damage_bonus: 1, attack_bonus: 0, base_value: 60, weight: 6, tier: 2 },
    item! { id: 5, name: "battle axe", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 10, damage_bonus: 0, attack_bonus: 2, base_value: 55, weight: 5, tier: 2 },
    item! { id: 6, name: "spear", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 8, damage_bonus: 1, attack_bonus: 2, base_value: 40, weight: 4, tier: 1 },
    item! { id: 7, name: "flail", glyph: '+', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 8, damage_bonus: 1, attack_bonus: 1, base_value: 45, weight: 4, tier: 2 },
    item! { id: 8, name: "two-handed sword", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 12, damage_bonus: 2, attack_bonus: 1, base_value: 90, weight: 7, tier: 3 },
    item! { id: 9, name: "staff", glyph: '|', color: Color::Brown, category: ItemCategory::Weapon, damage_die: 8, damage_bonus: 0, attack_bonus: 1, base_value: 30, weight: 3, tier: 1 },
    item! { id: 10, name: "crossbow", glyph: ']', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 8, damage_bonus: 1, attack_bonus: 5, base_value: 70, weight: 4, tier: 2 },
    item! { id: 11, name: "bow", glyph: ']', color: Color::Brown, category: ItemCategory::Weapon, damage_die: 6, damage_bonus: 1, attack_bonus: 6, base_value: 50, weight: 2, tier: 2 },
    item! { id: 12, name: "halberd", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 12, damage_bonus: 1, attack_bonus: 2, base_value: 80, weight: 7, tier: 3 },
    item! { id: 13, name: "trident", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 10, damage_bonus: 1, attack_bonus: 3, base_value: 65, weight: 5, tier: 2 },
    item! { id: 14, name: "morning star", glyph: '+', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 10, damage_bonus: 1, attack_bonus: 1, base_value: 60, weight: 5, tier: 2 },
    item! { id: 15, name: "scimitar", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 8, damage_bonus: 1, attack_bonus: 4, base_value: 55, weight: 3, tier: 2 },
    item! { id: 16, name: "war axe", glyph: '/', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 10, damage_bonus: 1, attack_bonus: 2, base_value: 60, weight: 5, tier: 2 },
    item! { id: 17, name: "great maul", glyph: '+', color: Color::Gray, category: ItemCategory::Weapon, damage_die: 12, damage_bonus: 3, attack_bonus: 0, base_value: 100, weight: 8, tier: 3 },
    item! { id: 18, name: "runed blade", glyph: '/', color: Color::Cyan, category: ItemCategory::Weapon, damage_die: 10, damage_bonus: 2, attack_bonus: 4, base_value: 150, weight: 4, tier: 4, min_depth: 16 },
    item! { id: 19, name: "abyssal sword", glyph: '/', color: Color::DarkMagenta, category: ItemCategory::Weapon, damage_die: 12, damage_bonus: 4, attack_bonus: 5, base_value: 400, weight: 5, tier: 5, min_depth: 21 },
    // ---- Armor (100-199) ----
    item! { id: 100, name: "leather armor", glyph: '[', color: Color::Brown, category: ItemCategory::Armor, ac_bonus: 3, base_value: 30, weight: 3, tier: 1 },
    item! { id: 101, name: "chain mail", glyph: '[', color: Color::Gray, category: ItemCategory::Armor, ac_bonus: 6, base_value: 80, weight: 8, tier: 1 },
    item! { id: 102, name: "studded leather", glyph: '[', color: Color::Brown, category: ItemCategory::Armor, ac_bonus: 4, base_value: 50, weight: 4, tier: 1 },
    item! { id: 103, name: "scale mail", glyph: '[', color: Color::Gray, category: ItemCategory::Armor, ac_bonus: 7, base_value: 110, weight: 9, tier: 2 },
    item! { id: 104, name: "plate mail", glyph: '[', color: Color::Gray, category: ItemCategory::Armor, ac_bonus: 10, base_value: 200, weight: 14, tier: 3, min_depth: 11 },
    item! { id: 105, name: "half plate", glyph: '[', color: Color::Gray, category: ItemCategory::Armor, ac_bonus: 8, base_value: 150, weight: 11, tier: 2, min_depth: 8 },
    item! { id: 106, name: "full plate", glyph: '[', color: Color::Gray, category: ItemCategory::Armor, ac_bonus: 12, base_value: 300, weight: 16, tier: 4, min_depth: 16 },
    item! { id: 107, name: "dragon scale", glyph: '[', color: Color::Green, category: ItemCategory::Armor, ac_bonus: 11, base_value: 250, weight: 10, tier: 4, min_depth: 16 },
    item! { id: 108, name: "mithril chain", glyph: '[', color: Color::Cyan, category: ItemCategory::Armor, ac_bonus: 9, base_value: 350, weight: 6, tier: 4, min_depth: 16 },
    item! { id: 109, name: "abyssal plate", glyph: '[', color: Color::DarkMagenta, category: ItemCategory::Armor, ac_bonus: 14, base_value: 600, weight: 12, tier: 5, min_depth: 21 },
    // ---- Potions (200-299) ----
    item! { id: 200, name: "small healing potion", glyph: '!', color: Color::Red, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Healing), base_value: 20, tier: 1 },
    item! { id: 201, name: "large healing potion", glyph: '!', color: Color::Red, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::FullHealing), base_value: 60, tier: 2, min_depth: 6 },
    item! { id: 202, name: "cure poison potion", glyph: '!', color: Color::Green, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::CurePoison), base_value: 30, tier: 1 },
    item! { id: 203, name: "restore energy potion", glyph: '!', color: Color::Blue, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::RestoreEp), base_value: 35, tier: 1 },
    item! { id: 204, name: "infravision potion", glyph: '!', color: Color::Orange, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Infravision), base_value: 25, tier: 1 },
    item! { id: 205, name: "energy potion", glyph: '!', color: Color::Yellow, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Energy), base_value: 40, tier: 2, min_depth: 6 },
    item! { id: 206, name: "experience potion", glyph: '!', color: Color::Magenta, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Experience), base_value: 80, tier: 3, min_depth: 11 },
    item! { id: 207, name: "berserk potion", glyph: '!', color: Color::DarkRed, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Berserk), base_value: 50, tier: 2, min_depth: 6 },
    item! { id: 208, name: "teleport potion", glyph: '!', color: Color::Cyan, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Teleport), base_value: 45, tier: 2, min_depth: 6 },
    item! { id: 209, name: "blindness potion", glyph: '!', color: Color::DarkGray, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Blindness), base_value: 20, tier: 1 },
    item! { id: 210, name: "confusion potion", glyph: '!', color: Color::DarkCyan, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Confusion), base_value: 20, tier: 1 },
    item! { id: 211, name: "antidote potion", glyph: '!', color: Color::Green, category: ItemCategory::Potion, potion_effect: Some(PotionEffect::Antidote), base_value: 15, tier: 1 },
    // ---- Wands (300-399) ----
    item! { id: 300, name: "wand of fire bolt", glyph: '~', color: Color::Orange, category: ItemCategory::Wand, wand_effect: Some(WandEffect::FireBolt), max_charges: 12, base_value: 120, tier: 1 },
    item! { id: 301, name: "wand of healing", glyph: '~', color: Color::Red, category: ItemCategory::Wand, wand_effect: Some(WandEffect::Healing), max_charges: 8, base_value: 100, tier: 1 },
    item! { id: 302, name: "wand of cold", glyph: '~', color: Color::Cyan, category: ItemCategory::Wand, wand_effect: Some(WandEffect::Cold), max_charges: 12, base_value: 130, tier: 2, min_depth: 6 },
    item! { id: 303, name: "wand of paralysis", glyph: '~', color: Color::Magenta, category: ItemCategory::Wand, wand_effect: Some(WandEffect::Paralysis), max_charges: 8, base_value: 150, tier: 2, min_depth: 6 },
    item! { id: 304, name: "wand of lightning", glyph: '~', color: Color::Yellow, category: ItemCategory::Wand, wand_effect: Some(WandEffect::Lightning), max_charges: 10, base_value: 160, tier: 3, min_depth: 11 },
    item! { id: 305, name: "wand of teleportation", glyph: '~', color: Color::Cyan, category: ItemCategory::Wand, wand_effect: Some(WandEffect::Teleport), max_charges: 6, base_value: 90, tier: 1 },
    item! { id: 306, name: "wand of sleep", glyph: '~', color: Color::DarkCyan, category: ItemCategory::Wand, wand_effect: Some(WandEffect::Sleep), max_charges: 10, base_value: 110, tier: 2, min_depth: 6 },
    item! { id: 307, name: "wand of disintegration", glyph: '~', color: Color::DarkMagenta, category: ItemCategory::Wand, wand_effect: Some(WandEffect::Disintegration), max_charges: 6, base_value: 300, tier: 4, min_depth: 16 },
    item! { id: 308, name: "wand of monster lightning", glyph: '~', color: Color::Yellow, category: ItemCategory::Wand, wand_effect: Some(WandEffect::MonsterLightning), max_charges: 8, base_value: 200, tier: 3, min_depth: 11 },
    // ---- Scrolls (400-499) ----
    item! { id: 400, name: "scroll of identify", glyph: '?', color: Color::White, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::Identify), base_value: 30, tier: 1 },
    item! { id: 401, name: "scroll of mapping", glyph: '?', color: Color::Cyan, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::Mapping), base_value: 40, tier: 1 },
    item! { id: 402, name: "scroll of enchant weapon", glyph: '?', color: Color::Yellow, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::EnchantWeapon), base_value: 80, tier: 2, min_depth: 6 },
    item! { id: 403, name: "scroll of enchant armor", glyph: '?', color: Color::Yellow, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::EnchantArmor), base_value: 80, tier: 2, min_depth: 6 },
    item! { id: 404, name: "scroll of teleportation", glyph: '?', color: Color::Cyan, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::Teleport), base_value: 50, tier: 1 },
    item! { id: 405, name: "scroll of blinking", glyph: '?', color: Color::Cyan, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::Blink), base_value: 45, tier: 1 },
    item! { id: 406, name: "scroll of creation", glyph: '?', color: Color::Magenta, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::Creation), base_value: 100, tier: 3, min_depth: 11 },
    item! { id: 407, name: "scroll of word of recall", glyph: '?', color: Color::White, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::WordOfRecall), base_value: 60, tier: 2, min_depth: 6 },
    item! { id: 408, name: "scroll of earthquake", glyph: '?', color: Color::Brown, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::Earthquake), base_value: 120, tier: 3, min_depth: 11 },
    item! { id: 409, name: "scroll of monster lightning", glyph: '?', color: Color::Yellow, category: ItemCategory::Scroll, scroll_effect: Some(ScrollEffect::MonsterLightning), base_value: 150, tier: 3, min_depth: 11 },
    // ---- Rings (500-599) ----
    item! { id: 500, name: "ring of strength", glyph: '(', color: Color::Red, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Strength), base_value: 150, tier: 2, min_depth: 6 },
    item! { id: 501, name: "ring of dexterity", glyph: '(', color: Color::Green, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Dexterity), base_value: 150, tier: 2, min_depth: 6 },
    item! { id: 502, name: "ring of constitution", glyph: '(', color: Color::Brown, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Constitution), base_value: 150, tier: 2, min_depth: 6 },
    item! { id: 503, name: "ring of intelligence", glyph: '(', color: Color::Blue, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Intelligence), base_value: 150, tier: 2, min_depth: 6 },
    item! { id: 504, name: "ring of wisdom", glyph: '(', color: Color::Cyan, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Wisdom), base_value: 150, tier: 2, min_depth: 6 },
    item! { id: 505, name: "ring of charisma", glyph: '(', color: Color::Magenta, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Charisma), base_value: 150, tier: 2, min_depth: 6 },
    item! { id: 506, name: "ring of regeneration", glyph: '(', color: Color::Green, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Regeneration), base_value: 300, tier: 3, min_depth: 11 },
    item! { id: 507, name: "ring of fire resistance", glyph: '(', color: Color::Orange, category: ItemCategory::Ring, ring_effect: Some(RingEffect::FireResist), base_value: 200, tier: 2, min_depth: 6 },
    item! { id: 508, name: "ring of stealth", glyph: '(', color: Color::DarkGray, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Stealth), base_value: 180, tier: 2, min_depth: 6 },
    item! { id: 509, name: "ring of luck", glyph: '(', color: Color::Yellow, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Luck), base_value: 250, tier: 3, min_depth: 11 },
    item! { id: 510, name: "ring of infravision", glyph: '(', color: Color::Orange, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Infravision), base_value: 120, tier: 1 },
    item! { id: 511, name: "ring of protection", glyph: '(', color: Color::Gray, category: ItemCategory::Ring, ring_effect: Some(RingEffect::Protection), base_value: 220, tier: 3, min_depth: 11 },
    // ---- Food (600-699) ----
    item! { id: 600, name: "trail rations", glyph: '%', color: Color::Brown, category: ItemCategory::Food, nutrition: 200, base_value: 5, tier: 1 },
    item! { id: 601, name: "apple", glyph: '%', color: Color::Red, category: ItemCategory::Food, nutrition: 80, base_value: 2, tier: 1 },
    item! { id: 602, name: "cheese", glyph: '%', color: Color::Yellow, category: ItemCategory::Food, nutrition: 120, base_value: 4, tier: 1 },
    item! { id: 603, name: "mushroom", glyph: '%', color: Color::Gray, category: ItemCategory::Food, nutrition: 60, base_value: 2, tier: 1 },
    item! { id: 604, name: "meat jerky", glyph: '%', color: Color::Brown, category: ItemCategory::Food, nutrition: 150, base_value: 6, tier: 1 },
    item! { id: 605, name: "honey cake", glyph: '%', color: Color::Yellow, category: ItemCategory::Food, nutrition: 180, base_value: 8, tier: 2, min_depth: 6 },
    item! { id: 606, name: "elven bread", glyph: '%', color: Color::Brown, category: ItemCategory::Food, nutrition: 220, base_value: 10, tier: 2, min_depth: 6 },
    item! { id: 607, name: "dwarven stew", glyph: '%', color: Color::Brown, category: ItemCategory::Food, nutrition: 250, base_value: 12, tier: 2, min_depth: 6 },
    item! { id: 608, name: "royal feast", glyph: '%', color: Color::Yellow, category: ItemCategory::Food, nutrition: 400, base_value: 30, tier: 3, min_depth: 11 },
    item! { id: 609, name: "ambrosia", glyph: '%', color: Color::Magenta, category: ItemCategory::Food, nutrition: 500, base_value: 60, tier: 4, min_depth: 16 },
    // ---- Amulets (700-799) ----
    item! { id: 700, name: "amulet of the abyss", glyph: '"', color: Color::DarkMagenta, category: ItemCategory::Amulet, base_value: 10000, tier: 5, min_depth: 25, max_depth: 25 },
    item! { id: 701, name: "amulet of protection", glyph: '"', color: Color::Gray, category: ItemCategory::Amulet, base_value: 300, tier: 3, min_depth: 11 },
    item! { id: 702, name: "amulet of life", glyph: '"', color: Color::Green, category: ItemCategory::Amulet, base_value: 500, tier: 4, min_depth: 16 },
    // ---- Misc (800-899) ----
    item! { id: 800, name: "signet ring", glyph: '(', color: Color::Yellow, category: ItemCategory::Misc, base_value: 50, tier: 1 },
    item! { id: 801, name: "iron key", glyph: '$', color: Color::Gray, category: ItemCategory::Misc, base_value: 20, tier: 2, min_depth: 13 },
    item! { id: 802, name: "torch", glyph: '$', color: Color::Orange, category: ItemCategory::Misc, base_value: 3, tier: 1 },
    item! { id: 803, name: "rope", glyph: '$', color: Color::Brown, category: ItemCategory::Misc, base_value: 5, tier: 1 },
    item! { id: 804, name: "lockpick", glyph: '$', color: Color::Gray, category: ItemCategory::Misc, base_value: 10, tier: 1 },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_expected_items() {
        assert!(ALL.len() >= 65);
    }

    #[test]
    fn ids_are_unique() {
        use std::collections::HashSet;
        let ids: HashSet<u32> = ALL.iter().map(|d| d.id).collect();
        assert_eq!(ids.len(), ALL.len());
    }

    #[test]
    fn by_id_lookup() {
        assert_eq!(ItemDef::by_id(0).unwrap().name, "shortsword");
        assert_eq!(ItemDef::by_id(700).unwrap().name, "amulet of the abyss");
        assert!(ItemDef::by_id(9999).is_none());
    }

    #[test]
    fn amulet_only_on_depth_25() {
        let amulet = ItemDef::by_id(700).unwrap();
        assert_eq!(amulet.min_depth, 25);
        assert_eq!(amulet.max_depth, 25);
    }

    #[test]
    fn weapons_have_damage() {
        for d in ALL {
            if d.category == ItemCategory::Weapon {
                assert!(d.damage_die > 0);
            }
        }
    }

    #[test]
    fn potions_have_effects() {
        for d in ALL {
            if d.category == ItemCategory::Potion {
                assert!(d.potion_effect.is_some());
            }
        }
    }
}
