//! Per-depth loot tables with rarity tiers.

use crate::core::rng::Rng;
use crate::entities::monster::Rarity;
use crate::items::catalog;
use crate::items::item::Item;

/// Roll an item found on the ground at the given depth.
pub fn roll_ground(rng: &mut Rng, depth: u8) -> Item {
    let roll = rng.int(0..100);
    if roll < 40 {
        catalog::random_weapon(rng, depth)
    } else if roll < 65 {
        catalog::random_armor(rng, depth)
    } else if roll < 75 {
        catalog::random_wand(rng)
    } else if roll < 85 {
        catalog::random_potion(rng)
    } else if roll < 92 {
        catalog::random_scroll(rng)
    } else if roll < 96 {
        catalog::random_ring(rng)
    } else {
        catalog::random_food(rng)
    }
}

/// Chance a monster of the given tier drops loot.
pub fn maybe_drop(rng: &mut Rng, tier: u8) -> bool {
    rng.chance(30 + tier as u64 * 5)
}

/// Roll a drop for a killed monster.
pub fn roll_drop(rng: &mut Rng, depth: u8) -> Item {
    let roll = rng.int(0..100);
    if roll < 50 {
        catalog::random_weapon(rng, depth)
    } else if roll < 80 {
        catalog::random_armor(rng, depth)
    } else if roll < 90 {
        catalog::random_potion(rng)
    } else if roll < 95 {
        catalog::random_ring(rng)
    } else {
        catalog::random_food(rng)
    }
}

#[allow(dead_code)]
pub fn rarity_roll(rng: &mut Rng) -> Rarity {
    let roll = rng.int(0..100);
    if roll < 60 {
        Rarity::Common
    } else if roll < 85 {
        Rarity::Uncommon
    } else if roll < 97 {
        Rarity::Rare
    } else {
        Rarity::Legendary
    }
}
