//! App state machine and input → Action mapping.

use crate::audio::sfx::SfxEngine;
use crate::core::action::Action;
use crate::core::game::Game;
use crate::data::classes::ClassId;
use crate::data::races::RaceId;
use crate::ui::panels::Panel;
use crate::ui::picker::{Picker, PickerKind};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Title,
    Creation,
    Play,
    Death,
    Victory,
}

#[derive(Debug, Clone)]
pub struct Targeting {
    pub x: u8,
    pub y: u8,
    pub wand_slot: usize,
}

#[derive(Debug, Clone)]
pub struct Creation {
    pub name: String,
    pub race: RaceId,
    pub class: ClassId,
}

impl Default for Creation {
    fn default() -> Self {
        Self {
            name: "Aldric".to_string(),
            race: RaceId::Human,
            class: ClassId::Warrior,
        }
    }
}

#[derive(Debug, Default)]
pub struct App {
    pub screen: Screen,
    pub game: Option<Game>,
    pub panel: Option<Panel>,
    pub picker: Option<Picker>,
    pub targeting: Option<Targeting>,
    pub creation: Creation,
    pub sfx: SfxEngine,
    pub quit_requested: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Title,
            game: None,
            panel: None,
            picker: None,
            targeting: None,
            creation: Creation::default(),
            sfx: SfxEngine::new(),
            quit_requested: false,
        }
    }

    pub fn start_game(&mut self, seed: u64) {
        let c = &self.creation;
        let game = Game::new(
            seed,
            &c.name,
            c.race.name(),
            c.class.name(),
        );
        self.game = Some(game);
        self.screen = Screen::Play;
    }

    pub fn load_game(&mut self, game: Game) {
        self.game = Some(game);
        self.screen = Screen::Play;
    }

    /// Resume the run in endless mode from the victory screen: the descent
    /// past the max depth is unlocked and the spawn scaling goes live.
    /// (No autosave: a won run is excluded from autosaves by design.)
    pub fn continue_endless(&mut self) {
        if let Some(game) = &mut self.game {
            game.endless = true;
            game.log(
                crate::core::message::MessageKind::System,
                "Endless descent. The dungeon stretches on below.",
            );
        }
        self.screen = Screen::Play;
    }

    /// Map a key event to a game action during play.
    pub fn handle_play_key(&mut self, key: KeyEvent) -> Option<Action> {
        if key.kind != KeyEventKind::Press {
            return None;
        }
        // The item picker is modal: it eats all keys until Enter or Esc.
        {
            let mut p = self.picker.clone();
            if let Some(p) = p.as_mut() {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => p.move_down(),
                    KeyCode::Char('k') | KeyCode::Up => p.move_up(),
                    KeyCode::Enter => {
                        let action = p.selected_action();
                        self.picker = None;
                        return match action {
                            // Using a wand switches to targeting mode.
                            Some(Action::UseItem(s))
                                if self
                                    .game
                                    .as_ref()
                                    .and_then(|g| g.player.inventory.get(s))
                                    .map(|i| matches!(i.kind, crate::items::item::ItemKind::Wand(_)))
                                    .unwrap_or(false) =>
                            {
                                if let Some(game) = &self.game {
                                    let (x, y) = game.player.pos;
                                    self.targeting = Some(Targeting {
                                        x,
                                        y,
                                        wand_slot: s,
                                    });
                                }
                                None
                            }
                            other => other,
                        };
                    }
                    KeyCode::Esc => {
                        self.picker = None;
                        return None;
                    }
                    _ => {}
                }
                self.picker = Some(p.clone());
                return None;
            }
        }
        // Targeting mode takes priority.
        if self.targeting.is_some() {
            return self.handle_targeting_key(key);
        }
        match key.code {
            KeyCode::Char('i') => {
                self.panel = Some(Panel::Inventory);
                return None;
            }
            KeyCode::Char('c') => {
                self.panel = Some(Panel::Character);
                return None;
            }
            KeyCode::Char('H') => {
                self.panel = Some(Panel::History);
                return None;
            }
            KeyCode::Char('?') => {
                self.panel = Some(Panel::Help);
                return None;
            }
            KeyCode::Char('M') => {
                self.sfx.toggle_mute();
                return None;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.quit_requested = true;
                return None;
            }
            KeyCode::Esc => {
                self.panel = None;
                return None;
            }
            KeyCode::Char('h') | KeyCode::Left => Some(Action::Move(-1, 0)),
            KeyCode::Char('l') | KeyCode::Right => Some(Action::Move(1, 0)),
            KeyCode::Char('j') | KeyCode::Down => Some(Action::Move(0, 1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::Move(0, -1)),
            KeyCode::Char('y') => Some(Action::Move(-1, -1)),
            KeyCode::Char('u') => Some(Action::Move(1, -1)),
            KeyCode::Char('b') => Some(Action::Move(-1, 1)),
            KeyCode::Char('n') => Some(Action::Move(1, 1)),
            KeyCode::Char('.') => Some(Action::Wait),
            KeyCode::Char('5') => Some(Action::Wait),
            KeyCode::Char('>') => Some(Action::StairsDown),
            KeyCode::Char('<') => Some(Action::StairsUp),
            KeyCode::Char('g') => Some(Action::Pickup),
            KeyCode::Char('U') => self.begin_picker(PickerKind::Use),
            KeyCode::Char('V') => self.begin_picker(PickerKind::Quaff),
            KeyCode::Char('E') => self.begin_picker(PickerKind::Eat),
            KeyCode::Char('D') => self.begin_picker(PickerKind::Drop),
            KeyCode::Char('W') => self.begin_picker(PickerKind::Wield),
            KeyCode::Char('Y') => self.begin_picker(PickerKind::Wear),
            KeyCode::Char('T') => self.begin_picker(PickerKind::TakeOff),
            KeyCode::Char('P') => self.begin_picker(PickerKind::RingOn),
            KeyCode::Char('O') => self.begin_picker(PickerKind::RingOff),
            KeyCode::Char('R') => self.begin_picker(PickerKind::Read),
            KeyCode::Char('I') => self.begin_picker(PickerKind::Identify),
            // Wand picker: selecting a wand enters targeting mode.
            KeyCode::Char('Z') => self.begin_picker(PickerKind::Wand),
            _ => None,
        }
    }

    /// Open the item picker for `kind`; no prompt if there is nothing to pick.
    fn begin_picker(&mut self, kind: PickerKind) -> Option<Action> {
        if let Some(game) = &self.game {
            if let Some(picker) = Picker::new(kind, game) {
                self.picker = Some(picker);
            }
        }
        None
    }

    fn handle_targeting_key(&mut self, key: KeyEvent) -> Option<Action> {
        let t = self.targeting.as_mut()?;
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => t.x = t.x.saturating_sub(1),
            KeyCode::Char('l') | KeyCode::Right => t.x = (t.x as u16 + 1).min(79) as u8,
            KeyCode::Char('j') | KeyCode::Down => t.y = (t.y as u16 + 1).min(24) as u8,
            KeyCode::Char('k') | KeyCode::Up => t.y = t.y.saturating_sub(1),
            KeyCode::Enter => {
                let slot = t.wand_slot;
                let (x, y) = (t.x, t.y);
                self.targeting = None;
                return Some(Action::FireWand(slot, x, y));
            }
            KeyCode::Esc => self.targeting = None,
            _ => {}
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::catalog;
    use crate::items::item::{
        ArmorKind, Item, PotionKind, RingKind, ScrollKind, WandKind, WeaponKind,
    };
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn app_with(inventory: Vec<Item>, equip: Option<(Item, Item, Vec<Item>)>) -> App {
        let mut app = App::new();
        let mut game = Game::new_test("T", "Human", "Warrior", 42);
        game.monsters.clear();
        game.player.inventory = inventory;
        if let Some((w, a, rings)) = equip {
            game.player.wielded = Some(w);
            game.player.armor = Some(a);
            game.player.rings = rings;
        }
        app.game = Some(game);
        app.screen = Screen::Play;
        app
    }

    #[test]
    fn new_bindings_open_the_right_picker() {
        let potion = catalog::make_potion(PotionKind::Healing(true));
        let cases: [(KeyCode, PickerKind); 12] = [
            (KeyCode::Char('U'), PickerKind::Use),
            (KeyCode::Char('V'), PickerKind::Quaff),
            (KeyCode::Char('E'), PickerKind::Eat),
            (KeyCode::Char('D'), PickerKind::Drop),
            (KeyCode::Char('W'), PickerKind::Wield),
            (KeyCode::Char('Y'), PickerKind::Wear),
            (KeyCode::Char('T'), PickerKind::TakeOff),
            (KeyCode::Char('P'), PickerKind::RingOn),
            (KeyCode::Char('O'), PickerKind::RingOff),
            (KeyCode::Char('R'), PickerKind::Read),
            (KeyCode::Char('I'), PickerKind::Identify),
            (KeyCode::Char('Z'), PickerKind::Wand),
        ];
        for (code, kind) in cases {
            let mut app = app_with(
                vec![
                    potion.clone(),
                    catalog::make_armor(ArmorKind::Plate, 0, false),
                    catalog::make_ring(RingKind::Protection),
                    catalog::make_wand(WandKind::FireBolt, 0),
                ],
                Some((
                    catalog::make_weapon(
                        WeaponKind::Dagger,
                        0,
                        false,
                        &mut crate::core::rng::Rng::new(1),
                    ),
                    catalog::make_armor(ArmorKind::Chainmail, 0, false),
                    vec![catalog::make_ring(RingKind::Protection)],
                )),
            );
            assert_eq!(app.handle_play_key(key(code)), None);
            assert_eq!(app.picker.as_ref().unwrap().kind, kind);
        }
    }

    #[test]
    fn existing_bindings_keep_their_meaning() {
        let mut app = app_with(vec![], None);
        assert_eq!(
            app.handle_play_key(key(KeyCode::Char('h'))),
            Some(Action::Move(-1, 0))
        );
        assert_eq!(
            app.handle_play_key(key(KeyCode::Char('u'))),
            Some(Action::Move(1, -1))
        );
        assert_eq!(app.handle_play_key(key(KeyCode::Char('g'))), Some(Action::Pickup));
        assert_eq!(app.handle_play_key(key(KeyCode::Char('.'))), Some(Action::Wait));
        assert!(!app.quit_requested);
        assert_eq!(app.handle_play_key(key(KeyCode::Char('q'))), None);
        assert!(app.quit_requested, "quit stays on q");
    }

    #[test]
    fn picker_enter_returns_action_for_selected_slot() {
        let mut app = app_with(vec![catalog::make_potion(PotionKind::Healing(true))], None);
        assert_eq!(app.handle_play_key(key(KeyCode::Char('D'))), None);
        assert_eq!(
            app.handle_play_key(key(KeyCode::Enter)),
            Some(Action::Drop(0))
        );
        assert!(app.picker.is_none());
    }

    #[test]
    fn picker_moves_cursor_before_selecting() {
        let items = vec![
            catalog::make_potion(PotionKind::Healing(true)),
            catalog::make_scroll(ScrollKind::Teleport),
        ];
        let mut app = app_with(items, None);
        app.handle_play_key(key(KeyCode::Char('D')));
        assert_eq!(app.handle_play_key(key(KeyCode::Char('j'))), None);
        assert_eq!(app.handle_play_key(key(KeyCode::Enter)), Some(Action::Drop(1)));
    }

    #[test]
    fn picker_cancel_does_nothing() {
        let mut app = app_with(vec![catalog::make_potion(PotionKind::Healing(true))], None);
        app.handle_play_key(key(KeyCode::Char('D')));
        assert_eq!(app.handle_play_key(key(KeyCode::Esc)), None);
        assert!(app.picker.is_none());
    }

    #[test]
    fn continue_endless_from_victory_sets_flag_and_resumes_play() {
        let mut app = app_with(vec![], None);
        let mut game = app.game.take().unwrap();
        game.won = true;
        app.game = Some(game);
        app.screen = Screen::Victory;
        assert!(!app.game.as_ref().unwrap().endless);
        app.continue_endless();
        assert_eq!(app.screen, Screen::Play);
        assert!(
            app.game.as_ref().unwrap().endless,
            "continuing from victory must enable endless mode"
        );
    }

    #[test]
    fn picker_key_with_empty_inventory_opens_no_picker() {
        let mut app = app_with(vec![], None);
        assert_eq!(app.handle_play_key(key(KeyCode::Char('D'))), None);
        assert!(app.picker.is_none());
        assert_eq!(app.handle_play_key(key(KeyCode::Char('O'))), None);
        assert!(app.picker.is_none());
    }

    #[test]
    fn takeoff_picker_targets_equipment_slots() {
        let w = catalog::make_weapon(WeaponKind::Dagger, 0, false, &mut crate::core::rng::Rng::new(1));
        let a = catalog::make_armor(ArmorKind::Chainmail, 0, false);
        let r1 = catalog::make_ring(RingKind::Protection);
        let r2 = catalog::make_ring(RingKind::Energy);
        let mut app = app_with(
            vec![catalog::make_potion(PotionKind::Healing(true))],
            Some((w, a, vec![r1, r2])),
        );
        app.handle_play_key(key(KeyCode::Char('T')));
        let slots: Vec<usize> = app
            .picker
            .as_ref()
            .unwrap()
            .rows
            .iter()
            .map(|r| r.slot)
            .collect();
        assert_eq!(slots, vec![0, 1, 2, 3]);
        assert_eq!(
            app.handle_play_key(key(KeyCode::Enter)),
            Some(Action::TakeOff(0))
        );
        // Reopen, walk to the first ring (equipment slot 2).
        app.handle_play_key(key(KeyCode::Char('T')));
        assert_eq!(app.handle_play_key(key(KeyCode::Char('j'))), None);
        assert_eq!(app.handle_play_key(key(KeyCode::Char('j'))), None);
        assert_eq!(
            app.handle_play_key(key(KeyCode::Enter)),
            Some(Action::TakeOff(2))
        );
    }

    #[test]
    fn ringoff_picker_maps_to_ring_indices() {
        let r1 = catalog::make_ring(RingKind::Protection);
        let r2 = catalog::make_ring(RingKind::Energy);
        let mut app = app_with(
            vec![],
            Some(
                (
                    catalog::make_weapon(WeaponKind::Dagger, 0, false, &mut crate::core::rng::Rng::new(1)),
                    catalog::make_armor(ArmorKind::Chainmail, 0, false),
                    vec![r1, r2],
                ),
            ),
        );
        app.handle_play_key(key(KeyCode::Char('O')));
        assert_eq!(app.handle_play_key(key(KeyCode::Enter)), Some(Action::RingOff(0)));
        app.handle_play_key(key(KeyCode::Char('O')));
        assert_eq!(app.handle_play_key(key(KeyCode::Char('j'))), None);
        assert_eq!(app.handle_play_key(key(KeyCode::Enter)), Some(Action::RingOff(1)));
    }

    #[test]
    fn using_a_wand_enters_targeting_mode() {
        let wand = catalog::make_wand(WandKind::FireBolt, 0);
        let mut app = app_with(vec![wand], None);
        let pos = app.game.as_ref().unwrap().player.pos;
        app.handle_play_key(key(KeyCode::Char('U')));
        assert_eq!(app.handle_play_key(key(KeyCode::Enter)), None);
        let t = app.targeting.clone().unwrap();
        assert_eq!(t.wand_slot, 0);
        assert_eq!((t.x, t.y), pos);
        // Cancel targeting: no action, mode cleared.
        assert_eq!(app.handle_play_key(key(KeyCode::Esc)), None);
        assert!(app.targeting.is_none());
    }

    #[test]
    fn firing_a_wand_completes_via_targeting() {
        let wand = catalog::make_wand(WandKind::FireBolt, 0);
        let mut app = app_with(vec![wand], None);
        app.handle_play_key(key(KeyCode::Char('U')));
        app.handle_play_key(key(KeyCode::Enter));
        assert!(app.targeting.is_some());
        // Confirm the target with Enter.
        let (x, y) = (
            app.targeting.as_ref().unwrap().x,
            app.targeting.as_ref().unwrap().y,
        );
        assert_eq!(
            app.handle_play_key(key(KeyCode::Enter)),
            Some(Action::FireWand(0, x, y))
        );
        assert!(app.targeting.is_none());
    }

    #[test]
    fn use_on_non_wand_is_sent_immediately() {
        let scroll = catalog::make_scroll(ScrollKind::Teleport);
        let mut app = app_with(vec![scroll], None);
        app.handle_play_key(key(KeyCode::Char('U')));
        assert_eq!(
            app.handle_play_key(key(KeyCode::Enter)),
            Some(Action::UseItem(0))
        );
        assert!(app.targeting.is_none());
    }

    #[test]
    fn wand_key_opens_wand_filtered_picker() {
        let items = vec![
            catalog::make_potion(PotionKind::Healing(true)),
            catalog::make_wand(WandKind::FireBolt, 0),
            catalog::make_scroll(ScrollKind::Teleport),
            catalog::make_wand(WandKind::Lightning, 0),
        ];
        let mut app = app_with(items, None);
        assert_eq!(app.handle_play_key(key(KeyCode::Char('Z'))), None);
        let slots: Vec<usize> = app
            .picker
            .as_ref()
            .unwrap()
            .rows
            .iter()
            .map(|r| r.slot)
            .collect();
        // Only the two wands (indices 1 and 3) are listed.
        assert_eq!(slots, vec![1, 3]);
    }

    #[test]
    fn wand_key_with_no_wands_opens_no_picker() {
        let items = vec![
            catalog::make_potion(PotionKind::Healing(true)),
            catalog::make_scroll(ScrollKind::Teleport),
        ];
        let mut app = app_with(items, None);
        assert_eq!(app.handle_play_key(key(KeyCode::Char('Z'))), None);
        assert!(app.picker.is_none());
    }

    #[test]
    fn wand_picker_enters_targeting_and_fires() {
        let items = vec![
            catalog::make_potion(PotionKind::Healing(true)),
            catalog::make_wand(WandKind::FireBolt, 0),
        ];
        let mut app = app_with(items, None);
        assert_eq!(app.handle_play_key(key(KeyCode::Char('Z'))), None);
        // Move the cursor to the wand (index 1) and select it.
        assert_eq!(app.handle_play_key(key(KeyCode::Char('j'))), None);
        assert_eq!(app.handle_play_key(key(KeyCode::Enter)), None);
        assert_eq!(app.targeting.as_ref().unwrap().wand_slot, 1);
        // Aim and fire.
        let (x, y) = (
            app.targeting.as_ref().unwrap().x,
            app.targeting.as_ref().unwrap().y,
        );
        assert_eq!(
            app.handle_play_key(key(KeyCode::Enter)),
            Some(Action::FireWand(1, x, y))
        );
        assert!(app.targeting.is_none());
    }

    #[test]
    fn wand_picker_cancel_leaves_targeting_cleared() {
        let items = vec![catalog::make_wand(WandKind::FireBolt, 0)];
        let mut app = app_with(items, None);
        assert_eq!(app.handle_play_key(key(KeyCode::Char('Z'))), None);
        assert_eq!(app.handle_play_key(key(KeyCode::Esc)), None);
        assert!(app.picker.is_none());
        assert!(app.targeting.is_none());
    }
}
