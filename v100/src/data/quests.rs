//! Quest definitions, state, and rewards.

use serde::{Deserialize, Serialize};

/// The lifecycle state of a quest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestState {
    /// Not yet available.
    Locked,
    /// Available to accept (quest giver present).
    Available,
    /// Accepted, in progress.
    Active,
    /// Ready to turn in (objective met).
    Ready,
    /// Completed and rewarded.
    Complete,
}

impl QuestState {
    /// A short display name for this state.
    pub fn name(self) -> &'static str {
        match self {
            QuestState::Locked => "locked",
            QuestState::Available => "available",
            QuestState::Active => "active",
            QuestState::Ready => "ready",
            QuestState::Complete => "complete",
        }
    }
}

/// A quest objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Objective {
    /// Kill N monsters of the given species id.
    Kill { species: u32, count: u32 },
    /// Deliver an item (the signet ring) to the quest giver.
    DeliverItem,
    /// Retrieve an item (the iron key) from a location.
    RetrieveItem,
}

/// A static quest definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestDef {
    pub id: u32,
    pub name: &'static str,
    pub description: &'static str,
    /// Depth range where the quest giver can be found.
    pub giver_depth_min: u32,
    pub giver_depth_max: u32,
    pub objective: Objective,
    /// XP reward.
    pub xp_reward: u32,
    /// Gold reward.
    pub gold_reward: u32,
    /// Item id reward (None for none).
    pub item_reward: Option<u32>,
}

impl QuestDef {
    /// The three quests.
    pub const ALL: &'static [QuestDef] = &[
        QuestDef {
            id: 0,
            name: "The Lost Signet",
            description: "An old man seeks his signet ring, lost with a guard on D2.",
            giver_depth_min: 1,
            giver_depth_max: 1,
            objective: Objective::DeliverItem,
            xp_reward: 50,
            gold_reward: 100,
            item_reward: Some(200), // small healing potion
        },
        QuestDef {
            id: 1,
            name: "Blood on the Altar",
            description: "Kill 4 Cultists of the Abyss.",
            giver_depth_min: 7,
            giver_depth_max: 9,
            objective: Objective::Kill {
                species: 8, // cultist
                count: 4,
            },
            xp_reward: 150,
            gold_reward: 200,
            item_reward: Some(300), // wand of fire bolt
        },
        QuestDef {
            id: 2,
            name: "The Sealed Chamber",
            description: "Recover the Iron Key and open a sealed vault.",
            giver_depth_min: 13,
            giver_depth_max: 14,
            objective: Objective::RetrieveItem,
            xp_reward: 300,
            gold_reward: 500,
            item_reward: None, // legendary item granted directly
        },
    ];

    pub fn by_id(id: u32) -> Option<&'static QuestDef> {
        Self::ALL.iter().find(|q| q.id == id)
    }
}

/// The mutable state of a single quest during a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestProgress {
    pub def_id: u32,
    pub state: QuestState,
    /// Progress toward a kill objective.
    pub kills: u32,
}

impl QuestProgress {
    pub fn new(def_id: u32) -> Self {
        Self {
            def_id,
            state: QuestState::Locked,
            kills: 0,
        }
    }

    pub fn def(&self) -> &'static QuestDef {
        QuestDef::by_id(self.def_id).expect("valid quest id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_quests_defined() {
        assert_eq!(QuestDef::ALL.len(), 3);
    }

    #[test]
    fn quest_ids_are_distinct() {
        use std::collections::HashSet;
        let ids: HashSet<u32> = QuestDef::ALL.iter().map(|q| q.id).collect();
        assert_eq!(ids.len(), QuestDef::ALL.len());
    }

    #[test]
    fn new_quest_is_locked() {
        let q = QuestProgress::new(0);
        assert_eq!(q.state, QuestState::Locked);
        assert_eq!(q.kills, 0);
    }

    #[test]
    fn quest_def_lookup() {
        assert_eq!(QuestDef::by_id(0).unwrap().name, "The Lost Signet");
        assert!(QuestDef::by_id(99).is_none());
    }
}
