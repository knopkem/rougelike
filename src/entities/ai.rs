//! Monster AI state machine: wander / chase / attack / wait.

use crate::core::rng::Rng;
use crate::entities::monster::Monster;
use crate::map::path;

#[derive(Debug)]
pub enum AiDecision {
    MoveTo((u8, u8)),
    AttackPlayer,
    Wait,
}

pub struct AiGame<'a> {
    pub level: &'a crate::map::level::Level,
    pub player_pos: (u8, u8),
    pub monsters: &'a [Monster],
    /// The player is under an invisibility effect. Monsters can then only
    /// "see" the player on direct line of sight (i.e. when adjacent).
    pub player_invisible: bool,
}

impl<'a> AiGame<'a> {
    pub fn new(
        level: &'a crate::map::level::Level,
        player_pos: (u8, u8),
        monsters: &'a [Monster],
        player_invisible: bool,
    ) -> Self {
        Self {
            level,
            player_pos,
            monsters,
            player_invisible,
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

    // Can the monster see the player? An invisible player is only seen on
    // direct line of sight (adjacency counts); issue 19 replaces the
    // flat sight roll with FOV-based sight.
    let sees = if game.player_invisible
        && !crate::map::fov::line_of_sight(
            game.level,
            m_pos.0 as i32,
            m_pos.1 as i32,
            player_pos.0 as i32,
            player_pos.1 as i32,
        ) {
        false
    } else {
        rng.chance(40)
    };
    if sees {
        // Chase via A*.
        if let Some(path) = path::astar(game.level, m_pos, player_pos) {
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
            && !game.monsters.iter().any(|o| o.pos == (nx, ny) && !o.dead)
        {
            return Some(AiDecision::MoveTo((nx, ny)));
        }
    }
    Some(AiDecision::Wait)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::level::Tile;

    /// A one-tile-wide corridor; the player sits beyond a wall gap the
    /// monster cannot reach, so a chase (if it happened) would have to
    /// detour. The direct corridor itself is clear.
    fn corridor_with_detour() -> crate::map::level::Level {
        let mut lvl = crate::map::level::Level::new(1);
        for x in 5..=30 {
            lvl.set_tile((x, 5), Tile::Floor);
        }
        for x in 5..=30 {
            lvl.set_tile((x, 8), Tile::Floor);
        }
        for y in 5..=8 {
            lvl.set_tile((5, y), Tile::Floor);
        }
        lvl
    }

    fn monster_at(pos: (u8, u8)) -> Monster {
        Monster::new(crate::data::monsters::MONSTERS[0].clone(), pos)
    }

    #[test]
    fn invisible_player_without_line_of_sight_is_never_targeted() {
        let lvl = corridor_with_detour();
        let m = monster_at((10, 5));
        let monsters = vec![m.clone()];
        // Player on the far corridor: the wall between the rows blocks any
        // direct line of sight, and the only route is the x=5 detour.
        let player = (20, 8);
        for seed in 0..32u64 {
            let mut rng = Rng::new(seed);
            let mut ai = AiGame::new(&lvl, player, &monsters, true);
            let d = act(&mut rng, &mut ai, &monsters[0]);
            assert!(
                !matches!(d, Some(AiDecision::AttackPlayer)),
                "seed {seed}: an unseen invisible player must never be attacked"
            );
            assert!(
                !matches!(d, Some(AiDecision::MoveTo(p)) if p == (9, 5)),
                "seed {seed}: an unseen invisible player must not be chased; got {d:?}"
            );
        }
    }

    #[test]
    fn invisible_player_on_direct_line_of_sight_can_still_be_found() {
        let lvl = corridor_with_detour();
        let m = monster_at((10, 5));
        let monsters = vec![m.clone()];
        let player = (20, 5); // same row, unobstructed line of sight
        let mut found = false;
        for seed in 0..64u64 {
            let mut rng = Rng::new(seed);
            let mut ai = AiGame::new(&lvl, player, &monsters, true);
            if matches!(
                act(&mut rng, &mut ai, &monsters[0]),
                Some(AiDecision::MoveTo(p)) if p == (11, 5)
            ) {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "an invisible player in direct line of sight must still be seeable (40% sight)"
        );
    }

    #[test]
    fn visible_player_can_be_chased() {
        let lvl = corridor_with_detour();
        let m = monster_at((10, 5));
        let monsters = vec![m.clone()];
        let player = (20, 5);
        let mut chased = false;
        for seed in 0..64u64 {
            let mut rng = Rng::new(seed);
            let mut ai = AiGame::new(&lvl, player, &monsters, false);
            if matches!(
                act(&mut rng, &mut ai, &monsters[0]),
                Some(AiDecision::MoveTo(p)) if p == (11, 5)
            ) {
                chased = true;
                break;
            }
        }
        assert!(
            chased,
            "a visible player in line of sight must sometimes be chased"
        );
    }
}
