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
