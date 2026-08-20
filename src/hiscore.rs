//! Persistent top-10 high scores.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::game::{Game, ScoreInfo};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hiscore {
    pub entries: Vec<ScoreInfo>,
}

pub fn hiscore_path() -> Option<PathBuf> {
    let base = dirs::data_dir()?;
    Some(base.join("deepdelve").join("hiscore.json"))
}

pub fn record(game: &Game) {
    let info = game.score_info();
    let mut hs = load();
    hs.entries.push(info);
    hs.entries.sort_by(|a, b| b.score.cmp(&a.score));
    hs.entries.truncate(10);
    save(&hs);
}

pub fn load() -> Hiscore {
    let path = match hiscore_path() {
        Some(p) => p,
        None => return Hiscore::default(),
    };
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(_) => return Hiscore::default(),
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save(hs: &Hiscore) {
    if let Some(path) = hiscore_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(hs) {
            let _ = std::fs::write(path, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::action::Action;
    use crate::tests_harness::new_game;
    use std::sync::Mutex;

    // Tests that observe the hiscore file must run one at a time because the
    // data dir is selected via the process-wide XDG_DATA_HOME variable.
    static LOCK: Mutex<()> = Mutex::new(());

    fn isolated(name: &str, f: impl FnOnce()) {
        let _guard = LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!(
            "deepdelve-test-{}-{}",
            std::process::id(),
            name
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("XDG_DATA_HOME", &dir);
        f();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn in_progress_run_records_nothing() {
        isolated("in_progress", || {
            let mut game = new_game(42);
            game.monsters.clear();
            game.do_turn(Action::Wait);
            assert!(game.alive && !game.won);
            assert_eq!(load().entries.len(), 0);
        });
    }

    #[test]
    fn death_records_exactly_one_entry() {
        isolated("death", || {
            let mut game = new_game(42);
            game.monsters.clear();
            game.player.hunger = 0;
            game.player.hp = 0;
            game.do_turn(Action::Wait);
            assert!(!game.alive);
            assert_eq!(load().entries.len(), 1);
        });
    }

    #[test]
    fn victory_records_exactly_one_entry() {
        isolated("victory", || {
            let mut game = new_game(42);
            game.monsters.clear();
            game.amulet_carried = true;
            game.current_level = 25;
            game.do_turn(Action::Wait);
            assert!(game.won);
            let hs = load();
            assert_eq!(hs.entries.len(), 1);
            assert!(hs.entries[0].won);
        });
    }

    #[test]
    fn death_and_victory_same_turn_record_once() {
        isolated("death_victory", || {
            let mut game = new_game(42);
            game.monsters.clear();
            game.player.hunger = 0;
            game.player.hp = 0;
            game.amulet_carried = true;
            game.current_level = 25;
            game.do_turn(Action::Wait);
            assert!(!game.alive);
            assert_eq!(load().entries.len(), 1);
        });
    }
}
