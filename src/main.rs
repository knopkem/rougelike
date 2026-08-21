//! Deepdelve — a terminal roguelike in Rust.

use std::io;
use std::time::Duration;

use crossterm::event::{self, KeyCode};

use deepdelve::audio::sfx::SfxEngine;
use deepdelve::core::game::Game;
use deepdelve::core::score;
use deepdelve::save;
use deepdelve::ui::app::{App, Screen};
use deepdelve::ui::menu;
use deepdelve::ui::panels;
use deepdelve::ui::render;

const USAGE: &str = "Usage: deepdelve [OPTIONS]

Options:
  --seed <u64>   Use an explicit run seed (default: generated)
  --headless     Run the game loop without the curses UI
  --no-audio     Start with audio disabled
  -h, --help     Show this help
";

fn main() {
    let args = match deepdelve::cli::parse(std::env::args().skip(1).collect::<Vec<String>>()) {
        Ok(Some(args)) => args,
        Ok(None) => {
            print!("{USAGE}");
            return;
        }
        Err(e) => {
            eprintln!("deepdelve: {e}");
            eprint!("{USAGE}");
            std::process::exit(2);
        }
    };
    let seed = args.seed.unwrap_or_else(deepdelve::cli::generated_seed);
    eprintln!("deepdelve: seed {seed}");
    if args.headless {
        run_headless(seed, args.no_audio);
        return;
    }
    if let Err(e) = run_ui(seed, args.no_audio) {
        eprintln!("deepdelve: {e}");
        std::process::exit(1);
    }
}

/// Interactive session: curses UI driving the app state machine.
fn run_ui(seed: u64, no_audio: bool) -> io::Result<()> {
    deepdelve::terminal::install_panic_hook();
    deepdelve::terminal::Guard::enter()?;
    let mut terminal = ratatui::init();
    let mut app = if no_audio {
        App::with_sfx(SfxEngine::disabled())
    } else {
        App::new()
    };

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
                    if let Some(picker) = &app.picker {
                        panels::render_picker(f, picker, f.area());
                    }
                }
            }
            Screen::Death | Screen::Victory => {
                if let Some(game) = &app.game {
                    let s = score::compute(game);
                    if app.screen == Screen::Death {
                        menu::render_death(f, s, game.current_level, game.player.level);
                    } else {
                        menu::render_victory(f, s, game.turn, !game.endless);
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
                    KeyCode::Enter => app.start_game(seed),
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
                            } else if game.won && !game.endless {
                                app.screen = Screen::Victory;
                            }
                            for ev in game.drain_events() {
                                app.sfx.play_event(&ev);
                            }
                            save::autosave(game);
                        }
                    }
                }
                Screen::Death | Screen::Victory => match key.code {
                    KeyCode::Char('c') | KeyCode::Char('C')
                        if app.screen == Screen::Victory
                            && app.game.as_ref().is_some_and(|g| g.won && !g.endless) =>
                    {
                        app.continue_endless();
                    }
                    KeyCode::Enter => {
                        app.game = None;
                        app.screen = Screen::Title;
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.quit_requested = true;
                    }
                    _ => {}
                },
            }
        }
    }

    // The terminal guard (Drop) and the panic hook restore the terminal.
    Ok(())
}

/// Headless session: no curses, no raw mode — the simulation driver
/// (`core::sim`) feeds the same `do_turn`/event path until the run ends.
fn run_headless(seed: u64, no_audio: bool) {
    let mut sfx = if no_audio {
        SfxEngine::disabled()
    } else {
        SfxEngine::new()
    };
    let mut game = Game::new(seed, "Deepdelver", "Human", "Warrior");
    let end = deepdelve::core::sim::run_turns(
        &mut game,
        10_000,
        |_turn, _game| Some(deepdelve::core::action::Action::Wait),
        |ev| sfx.play_event(ev),
    );
    let score = score::compute(&game);
    let end = match end {
        deepdelve::core::sim::RunEnd::Death => "death",
        deepdelve::core::sim::RunEnd::Victory => "victory",
        deepdelve::core::sim::RunEnd::Exhausted => "exhausted",
        deepdelve::core::sim::RunEnd::TurnLimit => "turn limit",
    };
    println!(
        "seed {} — {} after {} turns (D{}, {} kills, score {})",
        seed, end, game.turn, game.current_level, game.player.kills, score,
    );
}
