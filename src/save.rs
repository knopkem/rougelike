//! Save/load: serde JSON for full game state.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::game::Game;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,
    pub game: Game,
}

const SAVE_VERSION: u32 = 1;

pub fn save_dir() -> Option<PathBuf> {
    let base = dirs::data_dir()?;
    Some(base.join("deepdelve").join("saves"))
}

pub fn autosave(game: &Game) {
    if !game.alive || game.won {
        return;
    }
    let dir = match save_dir() {
        Some(d) => d,
        None => return,
    };
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("autosave.json");
    let file = SaveFile {
        version: SAVE_VERSION,
        game: game.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&file) {
        let _ = std::fs::write(path, json);
    }
}

pub fn load_autosave() -> Option<Game> {
    let dir = save_dir()?;
    let path = dir.join("autosave.json");
    let data = std::fs::read_to_string(path).ok()?;
    let file: SaveFile = serde_json::from_str(&data).ok()?;
    if file.version != SAVE_VERSION {
        return None;
    }
    if !file.game.alive || file.game.won {
        return None;
    }
    Some(file.game)
}

pub fn delete_save(_game: &Game) {
    let dir = match save_dir() {
        Some(d) => d,
        None => return,
    };
    let path = dir.join("autosave.json");
    let _ = std::fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests_harness::{new_game, with_isolated_data_dir};

    fn autosave_file_exists() -> bool {
        save_dir()
            .map(|d| d.join("autosave.json").exists())
            .unwrap_or(false)
    }

    fn write_autosave_file(game: &Game) {
        let dir = save_dir().unwrap();
        std::fs::create_dir_all(&dir).unwrap();
        let file = SaveFile {
            version: SAVE_VERSION,
            game: game.clone(),
        };
        std::fs::write(
            dir.join("autosave.json"),
            serde_json::to_string_pretty(&file).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn mid_run_autosave_writes_loadable_save() {
        with_isolated_data_dir("mid_run", || {
            let game = new_game(1);
            assert!(game.alive && !game.won);
            autosave(&game);
            assert!(autosave_file_exists());
            let loaded = load_autosave().expect("live autosave should load");
            assert!(loaded.alive && !loaded.won);
        });
    }

    #[test]
    fn dead_game_autosave_writes_nothing() {
        with_isolated_data_dir("dead_autosave", || {
            let mut game = new_game(2);
            game.alive = false;
            autosave(&game);
            assert!(!autosave_file_exists(), "dead game must not be autosaved");
            assert!(load_autosave().is_none());
        });
    }

    #[test]
    fn won_game_autosave_writes_nothing() {
        with_isolated_data_dir("won_autosave", || {
            let mut game = new_game(3);
            game.won = true;
            autosave(&game);
            assert!(!autosave_file_exists(), "won game must not be autosaved");
            assert!(load_autosave().is_none());
        });
    }

    #[test]
    fn load_rejects_dead_autosave_on_disk() {
        with_isolated_data_dir("load_dead", || {
            let mut game = new_game(4);
            game.alive = false;
            // Simulate a stale autosave left by an older build.
            write_autosave_file(&game);
            assert!(
                load_autosave().is_none(),
                "dead autosave must not be offered as Continue"
            );
        });
    }

    #[test]
    fn load_rejects_won_autosave_on_disk() {
        with_isolated_data_dir("load_won", || {
            let mut game = new_game(5);
            game.won = true;
            write_autosave_file(&game);
            assert!(
                load_autosave().is_none(),
                "won autosave must not be offered as Continue"
            );
        });
    }

    #[test]
    fn save_load_preserves_monsters() {
        with_isolated_data_dir("monster_roundtrip", || {
            let game = new_game(9);
            assert!(!game.monsters.is_empty(), "fresh game must have monsters");
            let before: Vec<_> = game
                .monsters
                .iter()
                .map(|m| {
                    (
                        m.def.clone(),
                        m.name.clone(),
                        m.pos,
                        m.hp,
                        m.max_hp,
                        m.xp,
                        m.dead,
                        m.is_unique,
                        m.is_boss,
                        m.ability_cooldown,
                    )
                })
                .collect();
            autosave(&game);
            let loaded = load_autosave().expect("autosave should load");
            let after: Vec<_> = loaded
                .monsters
                .iter()
                .map(|m| {
                    (
                        m.def.clone(),
                        m.name.clone(),
                        m.pos,
                        m.hp,
                        m.max_hp,
                        m.xp,
                        m.dead,
                        m.is_unique,
                        m.is_boss,
                        m.ability_cooldown,
                    )
                })
                .collect();
            assert_eq!(before, after, "save/load must preserve the monster list");
        });
    }

    #[test]
    fn death_turn_leaves_no_autosave() {
        with_isolated_data_dir("death_turn", || {
            let mut game = new_game(6);
            game.monsters.clear();
            game.player.hunger = 0;
            game.player.hp = 0;
            game.do_turn(crate::core::action::Action::Wait);
            // Mirrors the main loop: per-turn autosave after do_turn.
            autosave(&game);
            assert!(!game.alive);
            assert!(
                !autosave_file_exists(),
                "no autosave may remain after the death turn"
            );
        });
    }

    #[test]
    fn victory_turn_leaves_no_autosave() {
        with_isolated_data_dir("victory_turn", || {
            let mut game = new_game(7);
            game.monsters.clear();
            game.amulet_carried = true;
            game.current_level = 25;
            game.do_turn(crate::core::action::Action::Wait);
            autosave(&game);
            assert!(game.won);
            assert!(
                !autosave_file_exists(),
                "no autosave may remain after the victory turn"
            );
        });
    }
}
