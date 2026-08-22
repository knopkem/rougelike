//! The application: the main loop that ties the core, UI, and audio together.
//!
//! Responsibilities:
//! - Set up the terminal (raw mode, alternate screen) and restore it on clean
//!   exit AND on panic (via a panic hook + a drop guard).
//! - Map terminal input to `Action`s.
//! - Call `Game::do_turn`, render the resulting state, and consume `GameEvent`s
//!   to drive the audio engine.
//! - Handle menus (character creation, inventory, shop, quests) and end states
//!   (death, victory, quit).

use crate::audio::AudioEngine;
use crate::audio::sfx::Sfx;
use crate::core::action::{Action, Direction};
use crate::core::game::Game;
use crate::data::classes::{Class, Race};
use crate::hiscore::HighScoreEntry;
use crate::save;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Stdout;

use super::menu::{self, InventoryVerb};
use super::render;

/// The high-level application state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    CharacterCreation,
    Playing,
    Inventory {
        selected: usize,
    },
    #[allow(dead_code)]
    Shop {
        selected: usize,
    },
    Quests {
        selected: usize,
    },
    Help,
    Dead,
    Victory,
    Quit,
}

/// The application.
pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    game: Option<Game>,
    audio: AudioEngine,
    state: AppState,
    race_idx: usize,
    class_idx: usize,
    /// The seed for the run (None = random).
    seed: Option<u64>,
    /// Whether to record a high score on exit.
    record_score: bool,
    /// A pending inventory verb (set when the user presses a verb key).
    pending_verb: Option<InventoryVerb>,
}

impl App {
    /// Create a new application.
    pub fn new(seed: Option<u64>) -> Self {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend).expect("failed to create terminal");
        Self {
            terminal,
            game: None,
            audio: AudioEngine::new(),
            state: AppState::CharacterCreation,
            race_idx: 0,
            class_idx: 0,
            seed,
            record_score: true,
            pending_verb: None,
        }
    }

    /// Run the application's main loop.
    pub fn run(&mut self) {
        // Set up the terminal.
        enable_raw_mode().expect("failed to enable raw mode");
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen)
            .expect("failed to enter alternate screen");

        // Install a panic hook that restores the terminal before panicking.
        let old_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            eprintln!(
                "\nDeep Delve panicked: {}",
                info.payload()
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_else(|| format!(
                        "{:?}",
                        info.payload()
                            .downcast_ref::<&str>()
                            .copied()
                            .unwrap_or("unknown")
                    ))
            );
            old_hook(info);
        }));

        // The main loop.
        loop {
            if let Err(e) = self.frame() {
                eprintln!("render error: {}", e);
            }
            match self.state {
                AppState::Dead | AppState::Victory | AppState::Quit => break,
                _ => {}
            }
            if let Err(e) = self.handle_input() {
                eprintln!("input error: {}", e);
                break;
            }
        }

        // Restore the terminal.
        restore_terminal();
        let _ = crossterm::execute!(std::io::stdout(), LeaveAlternateScreen);
        disable_raw_mode().ok();

        // Record the high score if the run ended.
        if self.record_score {
            let is_over = self.game.as_ref().map(|g| g.is_over()).unwrap_or(false);
            if is_over {
                self.record_high_score();
            }
        }
    }

    /// Render one frame.
    fn frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state;
        let race_idx = self.race_idx;
        let class_idx = self.class_idx;
        let game = self.game.as_ref();
        self.terminal.draw(|f| {
            match state {
                AppState::CharacterCreation => {
                    menu::render_character_creation(f, f.area(), race_idx, class_idx);
                }
                AppState::Help => {
                    menu::render_overlay(
                        f,
                        f.area(),
                        " Help ",
                        &help_lines(),
                        ratatui::style::Color::Cyan,
                    );
                }
                AppState::Dead => {
                    if let Some(g) = game {
                        let cause = g
                            .cause_of_death
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string());
                        let score = g.score();
                        let body = vec![
                            format!("You died: {}", cause),
                            format!("Final score: {}", score.total),
                            format!("Depth reached: {}", g.depth),
                            format!("Monsters slain: {}", g.player.kills),
                            String::from("Press Enter to continue..."),
                        ];
                        menu::render_overlay(
                            f,
                            f.area(),
                            " Death ",
                            &body,
                            ratatui::style::Color::Red,
                        );
                    }
                }
                AppState::Victory => {
                    if let Some(g) = game {
                        let score = g.score();
                        let body = vec![
                            String::from("You claim the Amulet of the Abyss!"),
                            format!("Final score: {}", score.total),
                            format!("Depth reached: {}", g.depth),
                            format!("Monsters slain: {}", g.player.kills),
                            String::from("Press Enter to continue..."),
                        ];
                        menu::render_overlay(
                            f,
                            f.area(),
                            " Victory ",
                            &body,
                            ratatui::style::Color::Yellow,
                        );
                    }
                }
                AppState::Quit => {
                    menu::render_overlay(
                        f,
                        f.area(),
                        " Quit ",
                        &[String::from("Game saved. Goodbye!")],
                        ratatui::style::Color::Gray,
                    );
                }
                _ => {
                    if let Some(g) = game {
                        render::render(f, g);
                        // Draw any active menu overlay.
                        match state {
                            AppState::Inventory { selected } => {
                                let items: Vec<String> = g
                                    .player
                                    .inventory
                                    .iter()
                                    .enumerate()
                                    .map(|(i, item)| {
                                        let marker = if g.player.equipment.is_equipped(i) {
                                            "*"
                                        } else {
                                            " "
                                        };
                                        format!("{}{} {}", marker, i, item.name())
                                    })
                                    .collect();
                                let area = centered_rect(50, 70, f.area());
                                menu::render_list_menu(f, area, " Inventory ", &items, selected);
                            }
                            AppState::Shop { selected } => {
                                let items: Vec<String> = vec![String::from("(no shop here)")];
                                let area = centered_rect(50, 70, f.area());
                                menu::render_list_menu(f, area, " Shop ", &items, selected);
                            }
                            AppState::Quests { selected } => {
                                let items: Vec<String> = g
                                    .quests
                                    .quests
                                    .iter()
                                    .enumerate()
                                    .map(|(i, q)| {
                                        format!("{}. {} [{}]", i, q.def().name, q.state.name())
                                    })
                                    .collect();
                                let area = centered_rect(50, 70, f.area());
                                menu::render_list_menu(f, area, " Quests ", &items, selected);
                            }
                            _ => {}
                        }
                    }
                }
            }
        })?;
        Ok(())
    }

    /// Handle one input event and advance the game.
    fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let event = event::read()?;
        if let Event::Key(key) = event {
            self.on_key(key)?;
        }
        Ok(())
    }

    /// Process a key event based on the current state.
    fn on_key(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        match self.state {
            AppState::CharacterCreation => self.on_char_creation_key(key),
            AppState::Playing => self.on_playing_key(key)?,
            AppState::Inventory { selected } => self.on_inventory_key(key, selected)?,
            AppState::Shop { selected } => self.on_shop_key(key, selected)?,
            AppState::Quests { selected } => self.on_quest_key(key, selected)?,
            AppState::Help => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') {
                    self.state = AppState::Playing;
                }
            }
            AppState::Dead | AppState::Victory => {
                if key.code == KeyCode::Enter {
                    self.state = AppState::Quit;
                }
            }
            AppState::Quit => {}
        }
        Ok(())
    }

    /// Handle a key in character creation.
    fn on_char_creation_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                self.race_idx = (self.race_idx + Race::ALL.len() - 1) % Race::ALL.len();
            }
            KeyCode::Down => {
                self.race_idx = (self.race_idx + 1) % Race::ALL.len();
            }
            KeyCode::Left => {
                self.class_idx = (self.class_idx + Class::ALL.len() - 1) % Class::ALL.len();
            }
            KeyCode::Right => {
                self.class_idx = (self.class_idx + 1) % Class::ALL.len();
            }
            KeyCode::Enter => {
                let race = Race::ALL[self.race_idx];
                let class = Class::ALL[self.class_idx];
                let seed = self.seed.unwrap_or_else(|| {
                    // Derive a seed from the OS entropy via a fresh Rng.
                    let mut r = crate::core::rng::Rng::random();
                    r.range(0, u64::MAX)
                });
                self.game = Some(Game::new(seed, race, class));
                self.state = AppState::Playing;
            }
            KeyCode::Esc => {
                self.state = AppState::Quit;
            }
            _ => {}
        }
    }

    /// Handle a key while playing.
    fn on_playing_key(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        // Compute the action (or a special control flow) without holding a
        // borrow of `self.game`, so we can later mutate `self` freely.
        enum Cmd {
            Turn(Action),
            SaveQuit,
            Abort,
            Help,
            Inventory,
            Quests,
            Verb(InventoryVerb),
            None,
        }
        let cmd = match key.code {
            KeyCode::Up | KeyCode::Char('k') => Cmd::Turn(Action::Move(Direction::North)),
            KeyCode::Down | KeyCode::Char('j') => Cmd::Turn(Action::Move(Direction::South)),
            KeyCode::Right | KeyCode::Char('l') => Cmd::Turn(Action::Move(Direction::East)),
            KeyCode::Left | KeyCode::Char('h') => Cmd::Turn(Action::Move(Direction::West)),
            KeyCode::Char('u') => Cmd::Turn(Action::Move(Direction::NorthEast)),
            KeyCode::Char('y') => Cmd::Turn(Action::Move(Direction::NorthWest)),
            KeyCode::Char('n') => Cmd::Turn(Action::Move(Direction::SouthEast)),
            KeyCode::Char('b') => Cmd::Turn(Action::Move(Direction::SouthWest)),
            KeyCode::Char('.') | KeyCode::Char('5') | KeyCode::Char(' ') => Cmd::Turn(Action::Wait),
            KeyCode::Char('>') => Cmd::Turn(Action::StairsDown),
            KeyCode::Char('<') => Cmd::Turn(Action::StairsUp),
            KeyCode::Char(',') | KeyCode::Char('g') => Cmd::Turn(Action::Pickup),
            KeyCode::Char('p') => Cmd::Turn(Action::PickLock),
            KeyCode::Char('?') => Cmd::Help,
            KeyCode::Char('i') => Cmd::Inventory,
            KeyCode::Char('t') => Cmd::Quests,
            KeyCode::Char('q') => Cmd::Verb(InventoryVerb::Quaff),
            KeyCode::Char('e') => Cmd::Verb(InventoryVerb::Eat),
            KeyCode::Char('r') => Cmd::Verb(InventoryVerb::Read),
            KeyCode::Char('w') => Cmd::Verb(InventoryVerb::Wield),
            KeyCode::Char('W') => Cmd::Verb(InventoryVerb::Wear),
            KeyCode::Char('o') => Cmd::Verb(InventoryVerb::RingOn),
            KeyCode::Char('O') => Cmd::Verb(InventoryVerb::RingOff),
            KeyCode::Char('d') => Cmd::Verb(InventoryVerb::Drop),
            KeyCode::Char('I') => Cmd::Verb(InventoryVerb::Identify),
            KeyCode::Char('S') => Cmd::SaveQuit,
            KeyCode::Char('Q') => Cmd::Abort,
            _ => Cmd::None,
        };

        match cmd {
            Cmd::Turn(action) => {
                if let Some(game) = self.game.as_mut() {
                    game.do_turn(action);
                }
                self.consume_events();
                self.check_end_state();
            }
            Cmd::SaveQuit => {
                if let Some(game) = self.game.as_mut() {
                    game.do_turn(Action::SaveQuit);
                }
                self.save_and_quit();
            }
            Cmd::Abort => {
                if let Some(game) = self.game.as_mut() {
                    game.do_turn(Action::Abort);
                }
                self.state = AppState::Quit;
                self.record_score = false;
            }
            Cmd::Help => self.state = AppState::Help,
            Cmd::Inventory => self.state = AppState::Inventory { selected: 0 },
            Cmd::Quests => self.state = AppState::Quests { selected: 0 },
            Cmd::Verb(verb) => {
                self.state = AppState::Inventory { selected: 0 };
                self.pending_verb = Some(verb);
            }
            Cmd::None => {}
        }
        Ok(())
    }

    /// Handle a key in the inventory menu.
    fn on_inventory_key(
        &mut self,
        key: KeyEvent,
        selected: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Read the inventory length without holding a borrow.
        let count = self
            .game
            .as_ref()
            .map(|g| g.player.inventory.len())
            .unwrap_or(0);
        match key.code {
            KeyCode::Up => {
                let new_sel = if selected == 0 {
                    count.saturating_sub(1)
                } else {
                    selected - 1
                };
                self.state = AppState::Inventory { selected: new_sel };
            }
            KeyCode::Down => {
                let new_sel = (selected + 1) % count.max(1);
                self.state = AppState::Inventory { selected: new_sel };
            }
            KeyCode::Esc => {
                self.state = AppState::Playing;
                self.pending_verb = None;
            }
            KeyCode::Enter => {
                // Act on the selected item with the pending verb (or a sensible default).
                let verb = self.pending_verb.take().unwrap_or_else(|| {
                    self.game
                        .as_ref()
                        .and_then(|g| g.player.inventory.get(selected))
                        .map(default_verb_for)
                        .unwrap_or(InventoryVerb::Drop)
                });
                let action = menu::inventory_action_for(selected, verb);
                self.state = AppState::Playing;
                if let Some(game) = self.game.as_mut() {
                    game.do_turn(action);
                }
                self.consume_events();
                self.check_end_state();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle a key in the shop menu.
    fn on_shop_key(
        &mut self,
        key: KeyEvent,
        selected: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let _ = (key, selected);
        self.state = AppState::Playing;
        Ok(())
    }

    /// Handle a key in the quest menu.
    fn on_quest_key(
        &mut self,
        key: KeyEvent,
        selected: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let count = self
            .game
            .as_ref()
            .map(|g| g.quests.quests.len())
            .unwrap_or(0);
        match key.code {
            KeyCode::Up => {
                let new_sel = if selected == 0 {
                    count.saturating_sub(1)
                } else {
                    selected - 1
                };
                self.state = AppState::Quests { selected: new_sel };
            }
            KeyCode::Down => {
                let new_sel = (selected + 1) % count.max(1);
                self.state = AppState::Quests { selected: new_sel };
            }
            KeyCode::Esc => {
                self.state = AppState::Playing;
            }
            KeyCode::Char('a') => {
                if let Some(game) = self.game.as_mut() {
                    game.do_turn(Action::AcceptQuest { index: selected });
                }
                self.consume_events();
                self.state = AppState::Playing;
            }
            KeyCode::Char('t') => {
                if let Some(game) = self.game.as_mut() {
                    game.do_turn(Action::TurnInQuest { index: selected });
                }
                self.consume_events();
                self.state = AppState::Playing;
            }
            _ => {}
        }
        Ok(())
    }

    /// Consume the game's events and play the corresponding audio.
    fn consume_events(&mut self) {
        if let Some(game) = self.game.as_mut() {
            for event in game.drain_events() {
                if let Some(sfx) = Sfx::from_event(&event) {
                    self.audio.play(sfx);
                }
            }
        }
    }

    /// Check whether the game has ended and update the state.
    fn check_end_state(&mut self) {
        if let Some(game) = self.game.as_ref()
            && game.is_over()
        {
            if game.victory {
                self.state = AppState::Victory;
            } else if game.save_quit {
                self.state = AppState::Quit;
            } else {
                self.state = AppState::Dead;
            }
        }
    }

    /// Save the game and mark for quit.
    fn save_and_quit(&mut self) {
        if let Some(game) = self.game.as_ref() {
            let data = game.to_save();
            if let Err(e) = save::save_game(&data) {
                eprintln!("failed to save: {}", e);
            }
        }
        self.state = AppState::Quit;
        self.record_score = false;
    }

    /// Record a high score for a finished run.
    fn record_high_score(&mut self) {
        let (score, name, race, class, depth, victory) = {
            let game = self.game.as_ref().expect("game must exist");
            (
                game.score(),
                format!("{} {}", game.player.race.name(), game.player.class.name()),
                game.player.race.name().to_string(),
                game.player.class.name().to_string(),
                game.depth,
                game.victory,
            )
        };
        let mut table = crate::hiscore::load();
        let entry = HighScoreEntry {
            name,
            race,
            class,
            score,
            depth,
            victory,
            date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default(),
        };
        table.insert(entry);
        if let Err(e) = crate::hiscore::save(&table) {
            eprintln!("failed to save high scores: {}", e);
        }
    }
}

/// Restore the terminal to its normal state.
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = crossterm::execute!(std::io::stdout(), LeaveAlternateScreen);
}

/// A centered rectangle of the given percentage size.
fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let inner = popup_layout[1];
    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(inner)[1]
}

/// The help screen lines.
fn help_lines() -> Vec<String> {
    vec![
        String::from("Movement: arrows or h/j/k/l (diagonals: u/y/b/n)"),
        String::from("Wait: . or space    Stairs: > (down) < (up)"),
        String::from("Pickup: , or g    Pick lock: p"),
        String::from("Inventory: i    Quaff: q    Eat: e    Read: r"),
        String::from("Wield: w    Wear: W    Ring on: o    Ring off: O"),
        String::from("Drop: d    Identify: I"),
        String::from("Save & quit: S    Abort: Q    Help: ?"),
        String::from(""),
        String::from("Bump into monsters to attack them."),
        String::from("Reach the Amulet of the Abyss on D25 to win."),
    ]
}

/// The default verb for an item based on its category.
fn default_verb_for(item: &crate::items::item::Item) -> InventoryVerb {
    if item.is_potion() {
        InventoryVerb::Quaff
    } else if item.is_food() {
        InventoryVerb::Eat
    } else if item.is_scroll() {
        InventoryVerb::Read
    } else if item.is_weapon() {
        InventoryVerb::Wield
    } else if item.is_armor() {
        InventoryVerb::Wear
    } else if item.is_ring() {
        InventoryVerb::RingOn
    } else {
        InventoryVerb::Drop
    }
}
