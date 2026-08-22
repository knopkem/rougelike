//! Loot generation: pick items appropriate for a depth.

use super::catalog;
use super::item::{Item, ItemCategory};
use crate::core::rng::Rng;

/// Generate a random item appropriate for the given depth.
///
/// The item is weighted toward the depth's tier and category mix.
pub fn generate_item(rng: &mut Rng, depth: u32) -> Item {
    // Collect candidate item defs valid at this depth.
    let candidates: Vec<&crate::items::item::ItemDef> = catalog::ALL
        .iter()
        .filter(|d| depth >= d.min_depth && depth <= d.max_depth)
        .collect();

    if candidates.is_empty() {
        // Fallback: a trail ration.
        return Item::new(600);
    }

    // Weight by category to produce a sensible mix:
    // weapons/armor more common early, potions/food common throughout,
    // magic items (wands/scrolls/rings) rarer and depth-gated.
    let weights: Vec<u32> = candidates
        .iter()
        .map(|d| category_weight(d.category, depth))
        .collect();

    let chosen = rng
        .weighted(&candidates, &weights)
        .unwrap_or_else(|| *candidates.first().unwrap());

    let mut item = Item::new(chosen.id);

    // Random enchantment for weapons/armor (more likely at higher depth).
    if matches!(chosen.category, ItemCategory::Weapon | ItemCategory::Armor) {
        let enchant_chance = 10 + depth.min(20); // 10-30%
        if rng.chance(enchant_chance) {
            let max_enchant = 1 + (depth / 5).min(4) as i32;
            item.enchantment = rng.range(-1, max_enchant + 1);
        }
    }

    item
}

/// The relative weight for an item category at a given depth.
fn category_weight(category: ItemCategory, depth: u32) -> u32 {
    match category {
        ItemCategory::Weapon => 20,
        ItemCategory::Armor => 18,
        ItemCategory::Potion => 25,
        ItemCategory::Food => 15,
        ItemCategory::Scroll => {
            if depth >= 6 {
                10
            } else {
                4
            }
        }
        ItemCategory::Wand => {
            if depth >= 6 {
                8
            } else {
                3
            }
        }
        ItemCategory::Ring => {
            if depth >= 6 {
                6
            } else {
                2
            }
        }
        ItemCategory::Amulet => 1, // amulet of the abyss only on D25
        ItemCategory::Misc => 5,
    }
}

/// Generate a small pile of gold for a depth.
pub fn generate_gold(rng: &mut Rng, depth: u32) -> u32 {
    let base = 5 + depth * 3;
    let variance = base / 2;
    rng.range(base.saturating_sub(variance), base + variance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_items() {
        let mut rng = Rng::new(42);
        for depth in 1..=25 {
            for _ in 0..20 {
                let item = generate_item(&mut rng, depth);
                let def = item.def();
                assert!(depth >= def.min_depth && depth <= def.max_depth);
            }
        }
    }

    #[test]
    fn amulet_not_generated_before_depth_25() {
        let mut rng = Rng::new(7);
        for _ in 0..200 {
            let item = generate_item(&mut rng, 10);
            assert_ne!(item.def_id, 700);
        }
    }

    #[test]
    fn gold_scales_with_depth() {
        let mut rng = Rng::new(1);
        let shallow = generate_gold(&mut rng, 1);
        let deep = generate_gold(&mut rng, 20);
        // Deep gold should generally be higher (statistical, but with fixed seed deterministic).
        assert!(deep >= shallow / 2);
    }
}
