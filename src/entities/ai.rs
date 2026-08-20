//! Monster AI state machine: wander / chase / attack / wait.

use crate::core::rng::Rng;
use crate::entities::monster::Monster;
use crate::map::path;

pub enum AiDecision {
    MoveTo((u8, u8)),
    AttackPlayer,
    Wait,
}

pub struct AiGame<'a> {
    pub level: &'a crate::map::level::Level,
    pub player_pos: (u8, u8),
    pub monsters: &'a [Monster],
}

impl<'a> AiGame<'a> {
    pub fn new(
        level: &'a crate::map::level::Level,
        player_pos: (u8, u8),
        monsters: &'a [Monster],
    ) -> Self {
        Self {
            level,
            player_pos,
            monsters,
        }
    }
}

pub fn act<'a>(rng: &mut Rng, game: &mut AiGame<'a>, monster: &Monster) -> Option<AiDecision> {
    let player_pos = game.player_pos;
    let m_pos = monster.pos;

    // Adjacent? Attack.
    let dx = (m_pos.0 as i32 - player_pos.0 as i32).abs();
    let dy = (m_pos.1 as i32 - player_pos.1 as i32).abs();
    if dx + dy <= 1 {
        return Some(AiDecision::AttackPlayer);
    }

    // Can the monster see the player?
    let sees = rng.chance(40);
    if sees {
        // Chase via A*.
        if let Some(path) = path::astar(
            game.level,
            m_pos,
            player_pos,
        ) {
            if let Some(next) = path.first().copied() {
                return Some(AiDecision::MoveTo(next));
            }
        }
    }

    // Wander.
    let dirs = [(1u8, 0), (0, 1), (-1i8 as u8, 0), (0, 255u8)];
    for (dx, dy) in dirs {
        let nx = (m_pos.0 as i32 + dx as i32) as u8;
        let ny = (m_pos.1 as i32 + dy as i32) as u8;
        if game.level.is_walkable((nx, ny))
            && game.player_pos != (nx, ny)
            && !game
                .monsters
                .iter()
                .any(|o| o.pos == (nx, ny) && !o.dead)
        {
            return Some(AiDecision::MoveTo((nx, ny)));
        }
    }
    Some(AiDecision::Wait)
}
