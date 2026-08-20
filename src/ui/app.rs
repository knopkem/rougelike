//! App state machine and input → Action mapping.

use crate::core::action::Action;
use crate::core::game::Game;
use crate::data::classes::ClassId;
use crate::data::races::RaceId;
use crate::ui::panels::Panel;
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
    pub targeting: Option<Targeting>,
    pub creation: Creation,
    pub muted: bool,
    pub quit_requested: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Title,
            game: None,
            panel: None,
            targeting: None,
            creation: Creation::default(),
            muted: false,
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

    /// Map a key event to a game action during play.
    pub fn handle_play_key(&mut self, key: KeyEvent) -> Option<Action> {
        if key.kind != KeyEventKind::Press {
            return None;
        }
        // Targeting mode takes priority.
        {
            let mut t = self.targeting.clone();
            if let Some(t) = t.as_mut() {
                let result = self.handle_targeting_key(key, t);
                self.targeting = Some(t.clone());
                return result;
            }
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
                self.muted = !self.muted;
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
            _ => None,
        }
    }

    fn handle_targeting_key(&mut self, key: KeyEvent, t: &mut Targeting) -> Option<Action> {
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
            KeyCode::Esc => {
                self.targeting = None;
            }
            _ => {}
        }
        None
    }
}
