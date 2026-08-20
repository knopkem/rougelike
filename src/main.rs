//! Deepdelve — a terminal roguelike in Rust.

use crossterm::event::{self, KeyCode};
use std::io;
use std::time::Duration;

use deepdelve::core::score;
use deepdelve::save;
use deepdelve::ui::app::{App, Screen};
use deepdelve::ui::menu;
use deepdelve::ui::panels;
use deepdelve::ui::render;

fn main() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = ratatui::init();
    let mut app: App = App::new();

    loop {
        terminal.draw(|f| match app.screen {
            Screen::Title => menu::render_title(f, save::load_autosave().is_some()),
            Screen::Creation => menu::render_creation(f, &app.creation),
            Screen::Play => {
                if let Some(game) = &app.game {
                    render::render(f, game);
                    if let Some(panel) = &app.panel {
                        panels::render(f, game, panel);
                    }
                }
            }
            Screen::Death | Screen::Victory => {
                if let Some(game) = &app.game {
                    let s = score::compute(game);
                    if app.screen == Screen::Death {
                        menu::render_death(f, s, game.current_level, game.player.level);
                    } else {
                        menu::render_victory(f, s, game.turn);
                    }
                }
            }
        })?;

        if app.quit_requested {
            break;
        }

        let timeout = Duration::from_millis(80);
        if event::poll(timeout)? {
            let key = match event::read()? {
                crossterm::event::Event::Key(k) => k,
                _ => continue,
            };
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            match app.screen {
                Screen::Title => match key.code {
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.screen = Screen::Creation;
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        if let Some(game) = save::load_autosave() {
                            app.load_game(game);
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.quit_requested = true;
                    }
                    _ => {}
                },
                Screen::Creation => match key.code {
                    KeyCode::Enter => {
                        let seed = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as u64)
                            .unwrap_or(0);
                        app.start_game(seed);
                    }
                    KeyCode::Esc => {
                        app.screen = Screen::Title;
                    }
                    _ => {}
                },
                Screen::Play => {
                    if let Some(action) = app.handle_play_key(key) {
                        if let Some(game) = app.game.as_mut() {
                            game.do_turn(action);
                            if !game.alive {
                                app.screen = Screen::Death;
                            } else if game.won {
                                app.screen = Screen::Victory;
                            }
                            save::autosave(game);
                        }
                    }
                }
                Screen::Death | Screen::Victory => {
                    match key.code {
                        KeyCode::Enter => {
                            app.game = None;
                            app.screen = Screen::Title;
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            app.quit_requested = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
