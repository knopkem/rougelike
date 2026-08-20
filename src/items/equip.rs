//! Equipment slots, wield/wear rules.

use crate::items::item::Item;

/// True if two items occupy the same equipment slot.
pub fn same_slot(a: &Item, b: &Item) -> bool {
    matches!(
        (&a.kind, &b.kind),
        (
            crate::items::item::ItemKind::Weapon(_),
            crate::items::item::ItemKind::Weapon(_)
        ) | (
            crate::items::item::ItemKind::Armor(_),
            crate::items::item::ItemKind::Armor(_)
        ) | (
            crate::items::item::ItemKind::Shield(_),
            crate::items::item::ItemKind::Shield(_)
        ) | (
            crate::items::item::ItemKind::Ring(_),
            crate::items::item::ItemKind::Ring(_)
        )
    )
}
