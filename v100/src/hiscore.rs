//! High scores: persistent leaderboard stored as JSON.

use crate::core::score::Score;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single high score entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighScoreEntry {
    /// The player's name.
    pub name: String,
    /// The race.
    pub race: String,
    /// The class.
    pub class: String,
    /// The final score.
    pub score: Score,
    /// The depth reached.
    pub depth: u32,
    /// Whether the run was a victory.
    pub victory: bool,
    /// ISO-8601 timestamp of the run.
    pub date: String,
}

/// The high score table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HighScoreTable {
    /// Schema version.
    pub version: u32,
    /// The entries, sorted by total score descending.
    pub entries: Vec<HighScoreEntry>,
}

/// Maximum number of entries retained.
pub const MAX_ENTRIES: usize = 20;

impl HighScoreTable {
    pub fn new() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
        }
    }

    /// Insert a new entry, keeping the table sorted and bounded.
    pub fn insert(&mut self, entry: HighScoreEntry) {
        self.entries.push(entry);
        self.entries
            .sort_by_key(|e| std::cmp::Reverse(e.score.total));
        if self.entries.len() > MAX_ENTRIES {
            self.entries.truncate(MAX_ENTRIES);
        }
    }

    /// Whether a score qualifies for the table.
    pub fn qualifies(&self, total: u64) -> bool {
        if self.entries.len() < MAX_ENTRIES {
            return true;
        }
        self.entries
            .last()
            .map(|e| e.score.total < total)
            .unwrap_or(true)
    }

    /// The number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// The path to the high score file.
pub fn hiscore_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("deepdelve").join("hiscore.json")
}

/// Load the high score table from disk.
pub fn load() -> HighScoreTable {
    let path = hiscore_path();
    if !path.exists() {
        return HighScoreTable::new();
    }
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_else(|_| HighScoreTable::new()),
        Err(_) => HighScoreTable::new(),
    }
}

/// Save the high score table to disk.
pub fn save(table: &HighScoreTable) -> Result<(), String> {
    let path = hiscore_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(table).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entry(name: &str, total: u64) -> HighScoreEntry {
        HighScoreEntry {
            name: name.to_string(),
            race: "Human".to_string(),
            class: "Warrior".to_string(),
            score: Score {
                gold: total,
                xp: 0,
                depth: 1,
                quests: 0,
                kills: 0,
                total,
            },
            depth: 1,
            victory: false,
            date: "2025-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn new_table_is_empty() {
        let table = HighScoreTable::new();
        assert!(table.is_empty());
    }

    #[test]
    fn insert_sorts_descending() {
        let mut table = HighScoreTable::new();
        table.insert(test_entry("A", 100));
        table.insert(test_entry("B", 300));
        table.insert(test_entry("C", 200));
        assert_eq!(table.entries[0].name, "B");
        assert_eq!(table.entries[1].name, "C");
        assert_eq!(table.entries[2].name, "A");
    }

    #[test]
    fn table_is_bounded() {
        let mut table = HighScoreTable::new();
        for i in 0..(MAX_ENTRIES + 5) {
            table.insert(test_entry(&format!("P{i}"), i as u64));
        }
        assert_eq!(table.len(), MAX_ENTRIES);
        // The lowest scores should have been dropped: we inserted 0..25 and
        // kept the top 20, so the lowest surviving score is 5.
        assert_eq!(table.entries.last().unwrap().score.total, 5);
    }

    #[test]
    fn qualifies_logic() {
        let mut table = HighScoreTable::new();
        assert!(table.qualifies(1));
        for i in 0..MAX_ENTRIES {
            table.insert(test_entry(&format!("P{i}"), (i + 10) as u64));
        }
        // A score below the lowest entry should not qualify.
        assert!(!table.qualifies(5));
        // A score above the highest should qualify.
        assert!(table.qualifies(1000));
    }

    #[test]
    fn hiscore_path_is_set() {
        let path = hiscore_path();
        assert!(!path.as_os_str().is_empty());
    }
}
