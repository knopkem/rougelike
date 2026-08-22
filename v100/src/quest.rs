//! Quest runtime: tracking quest progress during a run.

use crate::data::quests::{QuestDef, QuestProgress, QuestState};
use serde::{Deserialize, Serialize};

/// The quest state for the entire run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestLog {
    /// Progress for each quest (indexed by quest def id).
    pub quests: Vec<QuestProgress>,
}

impl QuestLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the log with all quests in the Locked state.
    pub fn initialize(&mut self) {
        self.quests.clear();
        for def in QuestDef::ALL {
            self.quests.push(QuestProgress::new(def.id));
        }
    }

    /// Get a mutable reference to a quest by def id.
    pub fn get_mut(&mut self, def_id: u32) -> Option<&mut QuestProgress> {
        self.quests.iter_mut().find(|q| q.def_id == def_id)
    }

    /// Get an immutable reference to a quest by def id.
    pub fn get(&self, def_id: u32) -> Option<&QuestProgress> {
        self.quests.iter().find(|q| q.def_id == def_id)
    }

    /// The number of completed quests.
    pub fn completed_count(&self) -> u32 {
        self.quests
            .iter()
            .filter(|q| q.state == QuestState::Complete)
            .count() as u32
    }

    /// The number of active (in-progress) quests.
    pub fn active_count(&self) -> u32 {
        self.quests
            .iter()
            .filter(|q| q.state == QuestState::Active)
            .count() as u32
    }

    /// Accept a quest (if it's available).
    pub fn accept(&mut self, def_id: u32) -> bool {
        if let Some(q) = self.get_mut(def_id)
            && q.state == QuestState::Available
        {
            q.state = QuestState::Active;
            return true;
        }
        false
    }

    /// Record a kill toward a quest objective.
    pub fn record_kill(&mut self, species_id: u32) {
        for q in &mut self.quests {
            if q.state != QuestState::Active {
                continue;
            }
            let def = q.def();
            if let crate::data::quests::Objective::Kill { species, count } = def.objective
                && species == species_id
                && q.kills < count
            {
                q.kills += 1;
                if q.kills >= count {
                    q.state = QuestState::Ready;
                }
            }
        }
    }

    /// Mark a deliver/retrieve objective as met.
    pub fn mark_objective_met(&mut self, def_id: u32) {
        if let Some(q) = self.get_mut(def_id)
            && q.state == QuestState::Active
        {
            q.state = QuestState::Ready;
        }
    }

    /// Turn in a ready quest, returning the rewards.
    pub fn turn_in(&mut self, def_id: u32) -> Option<(u32, u32, Option<u32>)> {
        if let Some(q) = self.get_mut(def_id)
            && q.state == QuestState::Ready
        {
            let def = q.def();
            q.state = QuestState::Complete;
            return Some((def.xp_reward, def.gold_reward, def.item_reward));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_creates_all_quests() {
        let mut log = QuestLog::new();
        log.initialize();
        assert_eq!(log.quests.len(), QuestDef::ALL.len());
        assert!(log.quests.iter().all(|q| q.state == QuestState::Locked));
    }

    #[test]
    fn accept_available_quest() {
        let mut log = QuestLog::new();
        log.initialize();
        // Manually set a quest to Available.
        log.get_mut(0).unwrap().state = QuestState::Available;
        assert!(log.accept(0));
        assert_eq!(log.get(0).unwrap().state, QuestState::Active);
    }

    #[test]
    fn cannot_accept_locked_quest() {
        let mut log = QuestLog::new();
        log.initialize();
        assert!(!log.accept(0));
    }

    #[test]
    fn record_kill_progresses_quest() {
        let mut log = QuestLog::new();
        log.initialize();
        log.get_mut(1).unwrap().state = QuestState::Active;
        // Quest 1 is "Kill 4 cultists" (species 8).
        log.record_kill(8);
        log.record_kill(8);
        assert_eq!(log.get(1).unwrap().kills, 2);
        assert_eq!(log.get(1).unwrap().state, QuestState::Active);
        log.record_kill(8);
        log.record_kill(8);
        assert_eq!(log.get(1).unwrap().state, QuestState::Ready);
    }

    #[test]
    fn turn_in_ready_quest_gives_rewards() {
        let mut log = QuestLog::new();
        log.initialize();
        log.get_mut(0).unwrap().state = QuestState::Ready;
        let rewards = log.turn_in(0).unwrap();
        assert_eq!(rewards.0, 50); // xp
        assert_eq!(rewards.1, 100); // gold
        assert_eq!(log.get(0).unwrap().state, QuestState::Complete);
    }

    #[test]
    fn completed_count_tracks() {
        let mut log = QuestLog::new();
        log.initialize();
        assert_eq!(log.completed_count(), 0);
        log.get_mut(0).unwrap().state = QuestState::Complete;
        assert_eq!(log.completed_count(), 1);
    }
}
