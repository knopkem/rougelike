//! Status effects: timed conditions that modify entity behavior.

use serde::{Deserialize, Serialize};

/// A status effect that can be applied to an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Poisoned: takes damage each turn.
    Poison,
    /// Confused: moves randomly.
    Confused,
    /// Sleeping: won't act until hit.
    Sleeping,
    /// Paralyzed: cannot move or attack.
    Paralyzed,
    /// Petrified: cannot act at all (stronger than paralyzed).
    Petrified,
    /// Berserk: +attack, -AC.
    Berserk,
    /// Regenerating: heals each turn.
    Regenerating,
    /// Infravision: see in the dark.
    Infravision,
    /// Blind: cannot see.
    Blind,
    /// Haste: acts twice (simplified: +move).
    Haste,
    /// Slow: acts less often.
    Slow,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Poison => "poisoned",
            Status::Confused => "confused",
            Status::Sleeping => "sleeping",
            Status::Paralyzed => "paralyzed",
            Status::Petrified => "petrified",
            Status::Berserk => "berserk",
            Status::Regenerating => "regenerating",
            Status::Infravision => "infravision",
            Status::Blind => "blind",
            Status::Haste => "hasted",
            Status::Slow => "slowed",
        }
    }

    /// Whether this status is harmful.
    pub fn is_harmful(self) -> bool {
        matches!(
            self,
            Status::Poison
                | Status::Confused
                | Status::Sleeping
                | Status::Paralyzed
                | Status::Petrified
                | Status::Blind
                | Status::Slow
        )
    }
}

/// A status effect with a remaining duration (in turns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedStatus {
    pub status: Status,
    /// Remaining turns. 0 means expired.
    pub turns: u32,
}

impl TimedStatus {
    pub fn new(status: Status, turns: u32) -> Self {
        Self { status, turns }
    }

    /// Decrement the duration. Returns true if the status just expired.
    pub fn tick(&mut self) -> bool {
        if self.turns > 0 {
            self.turns -= 1;
        }
        self.turns == 0
    }
}

/// The collection of active status effects on an entity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusSet {
    statuses: Vec<TimedStatus>,
}

impl StatusSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a status, replacing any existing instance of the same kind.
    pub fn apply(&mut self, status: Status, turns: u32) {
        if let Some(existing) = self.statuses.iter_mut().find(|s| s.status == status) {
            existing.turns = existing.turns.max(turns);
        } else {
            self.statuses.push(TimedStatus::new(status, turns));
        }
    }

    /// Remove a status.
    pub fn remove(&mut self, status: Status) {
        self.statuses.retain(|s| s.status != status);
    }

    /// Whether the entity has the given status.
    pub fn has(&self, status: Status) -> bool {
        self.statuses.iter().any(|s| s.status == status)
    }

    /// The remaining turns for a status, if active.
    pub fn turns(&self, status: Status) -> Option<u32> {
        self.statuses
            .iter()
            .find(|s| s.status == status)
            .map(|s| s.turns)
    }

    /// Advance all statuses by one turn, removing expired ones.
    /// Returns the list of statuses that just expired.
    pub fn tick(&mut self) -> Vec<Status> {
        let mut expired = Vec::new();
        for s in &mut self.statuses {
            if s.tick() {
                expired.push(s.status);
            }
        }
        self.statuses.retain(|s| s.turns > 0);
        expired
    }

    /// All active statuses.
    pub fn all(&self) -> &[TimedStatus] {
        &self.statuses
    }

    /// Whether any status is active.
    pub fn is_empty(&self) -> bool {
        self.statuses.is_empty()
    }

    /// Clear all statuses.
    pub fn clear(&mut self) {
        self.statuses.clear();
    }

    /// Remove all harmful statuses (e.g. from a cure potion).
    pub fn clear_harmful(&mut self) {
        self.statuses.retain(|s| !s.status.is_harmful());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_and_has() {
        let mut s = StatusSet::new();
        s.apply(Status::Poison, 5);
        assert!(s.has(Status::Poison));
        assert!(!s.has(Status::Blind));
    }

    #[test]
    fn apply_extends_duration() {
        let mut s = StatusSet::new();
        s.apply(Status::Poison, 5);
        s.apply(Status::Poison, 10);
        assert_eq!(s.turns(Status::Poison), Some(10));
        assert_eq!(s.all().len(), 1);
    }

    #[test]
    fn tick_expires_statuses() {
        let mut s = StatusSet::new();
        s.apply(Status::Poison, 2);
        s.apply(Status::Blind, 1);
        let expired = s.tick();
        assert!(expired.contains(&Status::Blind));
        assert!(!expired.contains(&Status::Poison));
        assert!(s.has(Status::Poison));
        let expired = s.tick();
        assert!(expired.contains(&Status::Poison));
        assert!(s.is_empty());
    }

    #[test]
    fn clear_harmful_keeps_beneficial() {
        let mut s = StatusSet::new();
        s.apply(Status::Poison, 5);
        s.apply(Status::Regenerating, 5);
        s.clear_harmful();
        assert!(!s.has(Status::Poison));
        assert!(s.has(Status::Regenerating));
    }

    #[test]
    fn remove_specific() {
        let mut s = StatusSet::new();
        s.apply(Status::Poison, 5);
        s.remove(Status::Poison);
        assert!(!s.has(Status::Poison));
    }
}
