//! Score calculation: gold + XP + depth + quests.

use serde::{Deserialize, Serialize};

/// A breakdown of the final score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub gold: u64,
    pub xp: u64,
    pub depth: u64,
    pub quests: u64,
    pub kills: u64,
    pub total: u64,
}

impl Score {
    /// Compute a score from its components.
    ///
    /// - Gold counts 1:1.
    /// - XP counts 1:1.
    /// - Depth is worth 100 per level reached.
    /// - Completed quests are worth 500 each.
    /// - Kills are worth 10 each.
    pub fn compute(gold: u32, xp: u32, depth: u32, quests: u32, kills: u32) -> Self {
        let gold = gold as u64;
        let xp = xp as u64;
        let depth = (depth as u64) * 100;
        let quests = (quests as u64) * 500;
        let kills = (kills as u64) * 10;
        let total = gold + xp + depth + quests + kills;
        Self {
            gold,
            xp,
            depth,
            quests,
            kills,
            total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_components_sum() {
        let s = Score::compute(100, 200, 5, 2, 10);
        assert_eq!(s.gold, 100);
        assert_eq!(s.xp, 200);
        assert_eq!(s.depth, 500);
        assert_eq!(s.quests, 1000);
        assert_eq!(s.kills, 100);
        assert_eq!(s.total, 1900);
    }
}
