//! Quest data: 3 quests.

use crate::quest::{Quest, QuestStatus};

pub fn initial_quests() -> Vec<Quest> {
    vec![
        Quest {
            id: 1,
            name: "The Lost Signet".to_string(),
            description: "An old man seeks his signet ring, lost with a named guard on D2.".to_string(),
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
            description: "Recover the Iron Key (D13-14) and open a sealed vault.".to_string(),
            status: QuestStatus::Offered,
            progress: 0,
            target: 1,
        },
    ]
}
