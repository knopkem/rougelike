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
