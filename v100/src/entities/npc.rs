//! NPCs: quest givers and shopkeepers.

use crate::core::events::Pos;
use serde::{Deserialize, Serialize};

/// The role an NPC plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcRole {
    /// Gives quests.
    QuestGiver,
    /// Sells items (shopkeeper).
    Shopkeeper,
}

impl NpcRole {
    pub fn name(self) -> &'static str {
        match self {
            NpcRole::QuestGiver => "quest giver",
            NpcRole::Shopkeeper => "shopkeeper",
        }
    }
}

/// A non-player character on the level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Npc {
    /// The NPC's role.
    pub role: NpcRole,
    /// Position on the level.
    pub pos: Pos,
    /// Display name.
    pub name: String,
    /// Glyph.
    pub glyph: char,
    /// For quest givers: the quest def id they offer.
    pub quest_id: Option<u32>,
    /// For shopkeepers: the shop id (index into the shop's stock).
    pub shop_id: Option<u32>,
    /// Whether the player has talked to this NPC this turn.
    pub talked: bool,
}

impl Npc {
    /// Create a quest giver NPC.
    pub fn quest_giver(pos: Pos, name: &str, quest_id: u32) -> Self {
        Self {
            role: NpcRole::QuestGiver,
            pos,
            name: name.to_string(),
            glyph: '!',
            quest_id: Some(quest_id),
            shop_id: None,
            talked: false,
        }
    }

    /// Create a shopkeeper NPC.
    pub fn shopkeeper(pos: Pos, name: &str, shop_id: u32) -> Self {
        Self {
            role: NpcRole::Shopkeeper,
            pos,
            name: name.to_string(),
            glyph: '$',
            quest_id: None,
            shop_id: Some(shop_id),
            talked: false,
        }
    }

    /// Reset the per-turn talked flag.
    pub fn reset_turn(&mut self) {
        self.talked = false;
    }

    /// The NPC's position.
    pub fn pos(&self) -> Pos {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_giver_has_quest() {
        let npc = Npc::quest_giver(Pos::new(5, 5), "Old Man", 0);
        assert_eq!(npc.role, NpcRole::QuestGiver);
        assert_eq!(npc.quest_id, Some(0));
        assert!(npc.shop_id.is_none());
    }

    #[test]
    fn shopkeeper_has_shop() {
        let npc = Npc::shopkeeper(Pos::new(5, 5), "Merchant", 3);
        assert_eq!(npc.role, NpcRole::Shopkeeper);
        assert_eq!(npc.shop_id, Some(3));
        assert!(npc.quest_id.is_none());
    }

    #[test]
    fn reset_turn_clears_talked() {
        let mut npc = Npc::quest_giver(Pos::new(5, 5), "Old Man", 0);
        npc.talked = true;
        npc.reset_turn();
        assert!(!npc.talked);
    }
}
