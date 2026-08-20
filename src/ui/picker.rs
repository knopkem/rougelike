//! Item picker: prompts for an inventory slot (or equipment slot) to apply
//! an item action to. The inventory panel stays display-only; selection
//! happens here.

use crate::core::action::Action;
use crate::core::game::Game;
use crate::items::item::ItemKind;

/// Which item action the picker is collecting a selection for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    Use,
    Quaff,
    Eat,
    Drop,
    Wield,
    Wear,
    TakeOff,
    RingOn,
    RingOff,
    Read,
    Identify,
}

impl PickerKind {
    /// Prompt shown as the picker title.
    pub fn prompt(&self) -> &'static str {
        match self {
            PickerKind::Use => "Use which item?",
            PickerKind::Quaff => "Quaff which potion?",
            PickerKind::Eat => "Eat which food?",
            PickerKind::Drop => "Drop which item?",
            PickerKind::Wield => "Wield which weapon or shield?",
            PickerKind::Wear => "Wear which armor?",
            PickerKind::TakeOff => "Take off what?",
            PickerKind::RingOn => "Put on which ring?",
            PickerKind::RingOff => "Remove which ring?",
            PickerKind::Read => "Read which scroll?",
            PickerKind::Identify => "Identify which item?",
        }
    }

    /// Core action to send when the row carrying `slot` is selected.
    /// For `TakeOff`/`RingOff` the slot is an equipment slot (0 = wielded,
    /// 1 = armor, 2+ = ring); for everything else it is an inventory index.
    pub fn action(&self, slot: usize) -> Action {
        match self {
            PickerKind::Use => Action::UseItem(slot),
            PickerKind::Quaff => Action::Quaff(slot),
            PickerKind::Eat => Action::Eat(slot),
            PickerKind::Drop => Action::Drop(slot),
            PickerKind::Wield => Action::Wield(slot),
            PickerKind::Wear => Action::Wear(slot),
            PickerKind::TakeOff => Action::TakeOff(slot),
            PickerKind::RingOn => Action::RingOn(slot),
            PickerKind::RingOff => Action::RingOff(slot - 2),
            PickerKind::Read => Action::Read(slot),
            PickerKind::Identify => Action::Identify(slot),
        }
    }
}

/// One picker row: the core slot argument it maps to and its display label.
#[derive(Debug, Clone)]
pub struct PickerRow {
    pub slot: usize,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct Picker {
    pub kind: PickerKind,
    pub rows: Vec<PickerRow>,
    pub cursor: usize,
}

impl Picker {
    /// Build the picker for `kind`. Returns `None` when there is nothing to
    /// pick, so the caller does not open the prompt at all.
    pub fn new(kind: PickerKind, game: &Game) -> Option<Self> {
        let p = &game.player;
        let mut rows: Vec<PickerRow> = Vec::new();
        match kind {
            PickerKind::TakeOff => {
                if let Some(w) = &p.wielded {
                    rows.push(PickerRow {
                        slot: 0,
                        label: format!("wielded: {}", w.name()),
                    });
                }
                if let Some(a) = &p.armor {
                    rows.push(PickerRow {
                        slot: 1,
                        label: format!("armor:   {}", a.name()),
                    });
                }
                for (i, r) in p.rings.iter().enumerate() {
                    rows.push(PickerRow {
                        slot: i + 2,
                        label: format!("ring:    {}", r.name()),
                    });
                }
            }
            PickerKind::RingOff => {
                for (i, r) in p.rings.iter().enumerate() {
                    rows.push(PickerRow {
                        slot: i + 2,
                        label: r.name(),
                    });
                }
            }
            PickerKind::Wear => {
                for (i, item) in p.inventory.iter().enumerate() {
                    if matches!(item.kind, ItemKind::Armor(_)) {
                        rows.push(PickerRow {
                            slot: i,
                            label: format!("({}) {}", i, item.name()),
                        });
                    }
                }
            }
            PickerKind::RingOn => {
                for (i, item) in p.inventory.iter().enumerate() {
                    if matches!(item.kind, ItemKind::Ring(_)) {
                        rows.push(PickerRow {
                            slot: i,
                            label: format!("({}) {}", i, item.name()),
                        });
                    }
                }
            }
            _ => {
                for (i, item) in p.inventory.iter().enumerate() {
                    rows.push(PickerRow {
                        slot: i,
                        label: format!("({}) {}", i, item.name()),
                    });
                }
            }
        }
        if rows.is_empty() {
            return None;
        }
        Some(Self {
            kind,
            rows,
            cursor: 0,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Move the cursor down by one row, wrapping.
    pub fn move_down(&mut self) {
        if self.rows.len() > 1 {
            self.cursor = (self.cursor + 1) % self.rows.len();
        }
    }

    /// Move the cursor up by one row, wrapping.
    pub fn move_up(&mut self) {
        if self.rows.len() > 1 {
            self.cursor = self.cursor.wrapping_sub(1) % self.rows.len();
        }
    }

    /// The row under the cursor, if any.
    pub fn selected(&self) -> Option<&PickerRow> {
        self.rows.get(self.cursor)
    }

    /// Core action for the row under the cursor, if any.
    pub fn selected_action(&self) -> Option<Action> {
        self.selected().map(|r| self.kind.action(r.slot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game::Game;
    use crate::items::catalog;
    use crate::items::item::{
        ArmorKind, PotionKind, RingKind, ScrollKind, WeaponKind,
    };

    fn game_with(pick: fn(&mut Game)) -> Game {
        let mut g = Game::new_test("T", "Human", "Warrior", 42);
        g.monsters.clear();
        g.player.inventory.clear();
        pick(&mut g);
        g
    }

    fn add(p: &mut Game, item: crate::items::item::Item) {
        p.player.inventory.push(item);
    }

    #[test]
    fn inventory_picker_rows_match_slots() {
        let g = game_with(|g| {
            add(g, catalog::make_potion(PotionKind::Healing(true)));
            add(g, catalog::make_scroll(ScrollKind::Teleport));
        });
        let picker = Picker::new(PickerKind::Use, &g).unwrap();
        assert_eq!(picker.rows.len(), 2);
        assert_eq!(picker.rows[0].slot, 0);
        assert_eq!(picker.rows[1].slot, 1);
    }

    #[test]
    fn selection_maps_to_action_with_slot() {
        let g = game_with(|g| {
            add(g, catalog::make_potion(PotionKind::Healing(true)));
            add(g, catalog::make_scroll(ScrollKind::Teleport));
        });
        for (kind, expected) in [
            (PickerKind::Use, Action::UseItem(1)),
            (PickerKind::Quaff, Action::Quaff(1)),
            (PickerKind::Eat, Action::Eat(1)),
            (PickerKind::Drop, Action::Drop(1)),
            (PickerKind::Wield, Action::Wield(1)),
            (PickerKind::Read, Action::Read(1)),
            (PickerKind::Identify, Action::Identify(1)),
        ] {
            let mut picker = Picker::new(kind, &g).unwrap();
            picker.move_down();
            assert_eq!(picker.selected_action(), Some(expected.clone()));
        }
    }

    #[test]
    fn wear_lists_armor_only_and_ringon_lists_rings_only() {
        let g = game_with(|g| {
            add(g, catalog::make_potion(PotionKind::Healing(true)));
            add(g, catalog::make_armor(ArmorKind::Chainmail, 0, false));
            add(g, catalog::make_ring(RingKind::Protection));
        });
        let wear = Picker::new(PickerKind::Wear, &g).unwrap();
        assert_eq!(wear.rows.len(), 1);
        assert_eq!(wear.rows[0].slot, 1);
        assert_eq!(wear.selected_action(), Some(Action::Wear(1)));
        let ringon = Picker::new(PickerKind::RingOn, &g).unwrap();
        assert_eq!(ringon.rows.len(), 1);
        assert_eq!(ringon.rows[0].slot, 2);
        assert_eq!(ringon.selected_action(), Some(Action::RingOn(2)));
        let empty = game_with(|_| {});
        assert!(Picker::new(PickerKind::Wear, &empty).is_none());
        assert!(Picker::new(PickerKind::RingOn, &empty).is_none());
    }

    #[test]
    fn empty_inventory_yields_no_picker() {
        let g = game_with(|_| {});
        assert!(Picker::new(PickerKind::Drop, &g).is_none());
        assert!(Picker::new(PickerKind::Quaff, &g).is_none());
    }

    #[test]
    fn takeoff_lists_equipment_slots_only() {
        let g = game_with(|g| {
            g.player.wielded = Some(catalog::make_weapon(
                WeaponKind::Dagger,
                0,
                false,
                &mut g.rng,
            ));
            g.player.armor = Some(catalog::make_armor(ArmorKind::Chainmail, 0, false));
            g.player.rings.push(catalog::make_ring(RingKind::Protection));
            add(g, catalog::make_potion(PotionKind::Healing(true)));
        });
        let mut picker = Picker::new(PickerKind::TakeOff, &g).unwrap();
        assert_eq!(
            picker.rows.iter().map(|r| r.slot).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert_eq!(
            picker.selected_action(),
            Some(Action::TakeOff(0))
        );
        picker.move_down();
        picker.move_down();
        assert_eq!(picker.selected_action(), Some(Action::TakeOff(2)));
    }

    #[test]
    fn ringoff_maps_equipment_slot_to_ring_index() {
        let g = game_with(|g| {
            g.player.rings.push(catalog::make_ring(RingKind::Protection));
            g.player.rings.push(catalog::make_ring(RingKind::Energy));
        });
        let mut picker = Picker::new(PickerKind::RingOff, &g).unwrap();
        assert_eq!(picker.selected_action(), Some(Action::RingOff(0)));
        picker.move_down();
        assert_eq!(picker.selected_action(), Some(Action::RingOff(1)));
    }

    #[test]
    fn ringoff_without_rings_yields_no_picker() {
        let g = game_with(|_| {});
        assert!(Picker::new(PickerKind::RingOff, &g).is_none());
    }

    #[test]
    fn cursor_wraps_around() {
        let g = game_with(|g| {
            add(g, catalog::make_potion(PotionKind::Healing(true)));
            add(g, catalog::make_scroll(ScrollKind::Teleport));
        });
        let mut picker = Picker::new(PickerKind::Read, &g).unwrap();
        picker.move_down();
        assert_eq!(picker.cursor, 1);
        picker.move_down();
        assert_eq!(picker.cursor, 0, "wraps to first row");
        picker.move_up();
        assert_eq!(picker.cursor, 1, "wraps to last row");
    }

    #[test]
    fn single_row_cursor_is_stable() {
        let g = game_with(|g| add(g, catalog::make_potion(PotionKind::Healing(true))));
        let mut picker = Picker::new(PickerKind::Quaff, &g).unwrap();
        picker.move_down();
        picker.move_up();
        assert_eq!(picker.cursor, 0);
        assert_eq!(picker.selected_action(), Some(Action::Quaff(0)));
    }
}
