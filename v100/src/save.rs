//! Save/load: serialize and deserialize the full game state.
//!
//! Saves are stored as JSON in the platform data directory
//! (`~/.local/share/deepdelve/saves/` on Linux, equivalent on other OSes).
//! A versioned schema field guards against loading incompatible saves.

use crate::core::rng::Rng;
use crate::entities::player::Player;
use crate::map::level::Level;
use crate::quest::QuestLog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// The save file schema version. Bump this when the save format changes.
pub const SAVE_VERSION: u32 = 1;

/// The full serializable game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// Schema version.
    pub version: u32,
    /// The seed used for this run (for reproducibility).
    pub seed: u64,
    /// The current RNG state.
    pub rng: Rng,
    /// The current depth.
    pub depth: u32,
    /// The current level.
    pub level: Level,
    /// The player.
    pub player: Player,
    /// The quest log.
    pub quests: QuestLog,
    /// The current turn number.
    pub turn: u64,
    /// The message log (recent messages).
    pub messages: crate::core::message::MessageLog,
    /// Whether the game is in endless mode (past D25).
    pub endless: bool,
}

impl SaveData {
    /// Create a new save from the current game state.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seed: u64,
        rng: Rng,
        depth: u32,
        level: Level,
        player: Player,
        quests: QuestLog,
        turn: u64,
        messages: crate::core::message::MessageLog,
        endless: bool,
    ) -> Self {
        Self {
            version: SAVE_VERSION,
            seed,
            rng,
            depth,
            level,
            player,
            quests,
            turn,
            messages,
            endless,
        }
    }

    /// Validate the save version.
    pub fn is_compatible(&self) -> bool {
        self.version == SAVE_VERSION
    }
}

/// The directory where saves are stored.
pub fn save_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("deepdelve").join("saves")
}

/// Save the game state to a file. Returns the path on success.
pub fn save_game(data: &SaveData) -> Result<PathBuf, SaveError> {
    let dir = save_dir();
    fs::create_dir_all(&dir).map_err(|e| SaveError::Io(e.to_string()))?;
    let path = dir.join(format!("save_{}.json", data.depth));
    let json =
        serde_json::to_string_pretty(data).map_err(|e| SaveError::Serialize(e.to_string()))?;
    fs::write(&path, json).map_err(|e| SaveError::Io(e.to_string()))?;
    Ok(path)
}

/// Load the game state from a file.
pub fn load_game(path: &PathBuf) -> Result<SaveData, SaveError> {
    let json = fs::read_to_string(path).map_err(|e| SaveError::Io(e.to_string()))?;
    let data: SaveData =
        serde_json::from_str(&json).map_err(|e| SaveError::Deserialize(e.to_string()))?;
    if !data.is_compatible() {
        return Err(SaveError::IncompatibleVersion {
            found: data.version,
            expected: SAVE_VERSION,
        });
    }
    Ok(data)
}

/// Delete a save file (on death, for permadeath).
pub fn delete_save(path: &PathBuf) -> Result<(), SaveError> {
    if path.exists() {
        fs::remove_file(path).map_err(|e| SaveError::Io(e.to_string()))?;
    }
    Ok(())
}

/// List all save files in the save directory.
pub fn list_saves() -> Vec<PathBuf> {
    let dir = save_dir();
    if !dir.exists() {
        return Vec::new();
    }
    fs::read_dir(&dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|ext| ext == "json").unwrap_or(false))
                .collect()
        })
        .unwrap_or_default()
}

/// An error that can occur during save/load.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialize(String),
    #[error("deserialization error: {0}")]
    Deserialize(String),
    #[error("incompatible save version: found {found}, expected {expected}")]
    IncompatibleVersion { found: u32, expected: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::Pos;
    use crate::data::classes::{Class, Race};

    fn test_save() -> SaveData {
        let mut rng = Rng::new(42);
        let player = Player::new(Pos::new(10, 10), Race::Human, Class::Warrior);
        let level = crate::map::generation::generate_level(&mut rng, 1, false);
        let quests = QuestLog::new();
        let messages = crate::core::message::MessageLog::new();
        SaveData::new(42, rng, 1, level, player, quests, 0, messages, false)
    }

    #[test]
    fn save_and_load_roundtrip() {
        let data = test_save();
        let json = serde_json::to_string(&data).unwrap();
        let loaded: SaveData = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.version, SAVE_VERSION);
        assert_eq!(loaded.depth, 1);
        assert_eq!(loaded.player.level, 1);
        assert!(loaded.is_compatible());
    }

    #[test]
    fn save_data_is_compatible() {
        let data = test_save();
        assert!(data.is_compatible());
    }

    #[test]
    fn save_dir_exists() {
        let dir = save_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn list_saves_does_not_panic() {
        let saves = list_saves();
        // Just verify it doesn't panic; the count depends on the environment.
        let _ = saves;
    }
}
