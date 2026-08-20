//! Quest definitions, state, rewards.

use serde::{Deserialize, Serialize};

use crate::core::game::Game;
use crate::data::monsters::MonsterDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    Offered,
    Active,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: u8,
    pub name: String,
    pub description: String,
    pub status: QuestStatus,
    pub progress: u8,
    pub target: u8,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestLog {
    pub quests: Vec<Quest>,
    pub completed: Vec<u8>,
}

impl QuestLog {
    pub fn default() -> Self {
        Self {
            quests: vec![
                Quest {
                    id: 1,
                    name: "The Lost Signet".to_string(),
                    description: "An old man seeks his signet ring on D2.".to_string(),
                    status: QuestStatus::Offered,
                    progress: 0,
                    target: 1,
                },
                Quest {
                    id: 2,
                    name: "Blood on the Altar".to_string(),
                    description: "Kill 4 Cultists of the Abyss (D7-9).".to_string(),
                    status: QuestStatus::Offered,
                    progress: 0,
                    target: 4,
                },
                Quest {
                    id: 3,
                    name: "The Sealed Chamber".to_string(),
                    description: "Recover the Iron Key (D13-14).".to_string(),
                    status: QuestStatus::Offered,
                    progress: 0,
                    target: 1,
                },
            ],
            completed: Vec::new(),
        }
    }

    pub fn check_progress(&mut self, _game: &Game) {
        // Hook for quest progress checks.
    }

    pub fn on_kill(&mut self, game: &mut Game, def: &MonsterDef) {
        // Track kills for quest 2.
        if def.name.contains("Cultist") {
            if let Some(q) = self.quests.iter_mut().find(|q| q.id == 2) {
                if q.status == QuestStatus::Active {
                    q.progress += 1;
                    if q.progress >= q.target {
                        q.status = QuestStatus::Completed;
                        self.completed.push(q.id);
                        game.emit(crate::core::events::GameEvent::Quest {
                            kind: crate::core::events::QuestEvent::Completed,
                        });
                    }
                }
            }
        }
    }
}
