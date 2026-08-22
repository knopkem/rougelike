//! Monster AI: decides what a monster does each turn.

use crate::core::events::Pos;
use crate::core::rng::Rng;
use crate::entities::monster::Monster;
use crate::map::fov::line_of_sight;
use crate::map::level::Level;
use crate::map::path::{PathResult, astar};

/// The action a monster wants to take this turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiAction {
    /// Do nothing (stunned, sleeping, etc.).
    Idle,
    /// Move to an adjacent tile (toward the player or wandering).
    Move { dx: i32, dy: i32 },
    /// Melee attack the player (must be adjacent).
    MeleeAttack,
    /// Ranged attack the player (must have LOS).
    RangedAttack,
    /// Use a special ability.
    Ability(crate::data::monsters::Ability),
    /// Confused: move in a random direction.
    ConfusedMove { dx: i32, dy: i32 },
}

/// The result of running the AI for a monster.
pub struct AiDecision {
    pub action: AiAction,
    /// Whether the monster is now aware of the player.
    pub aware: bool,
}

/// Run the AI for a single monster.
///
/// This is a pure decision function: it inspects the monster, the level, and the
/// player position, and returns what the monster wants to do. The game loop is
/// responsible for actually executing the action (checking collisions, resolving
/// combat, etc.).
pub fn decide(monster: &Monster, level: &Level, player_pos: Pos, rng: &mut Rng) -> AiDecision {
    let mpos = monster.pos();

    // Stunned monsters don't act.
    if monster.is_stunned() {
        return AiDecision {
            action: AiAction::Idle,
            aware: false,
        };
    }

    // Confused monsters move randomly.
    if monster.statuses.has(crate::status::Status::Confused) {
        let (dx, dy) = random_direction(rng);
        return AiDecision {
            action: AiAction::ConfusedMove { dx, dy },
            aware: false,
        };
    }

    let can_see = line_of_sight(level, mpos, player_pos);
    let distance = mpos.manhattan(player_pos);

    // If the monster can see the player and is adjacent, melee attack.
    if can_see && distance == 1 {
        // Check for special abilities first (chance-based).
        if let Some(ability) = maybe_use_ability(monster, rng) {
            return AiDecision {
                action: AiAction::Ability(ability),
                aware: true,
            };
        }
        return AiDecision {
            action: AiAction::MeleeAttack,
            aware: true,
        };
    }

    // Ranged monsters attack if they have LOS and are in range.
    if monster.is_ranged() && can_see && distance <= 8 {
        if let Some(ability) = maybe_use_ability(monster, rng) {
            return AiDecision {
                action: AiAction::Ability(ability),
                aware: true,
            };
        }
        return AiDecision {
            action: AiAction::RangedAttack,
            aware: true,
        };
    }

    // If the monster can see the player but isn't adjacent, move toward them.
    if can_see {
        if let Some((dx, dy)) = step_toward(level, mpos, player_pos) {
            return AiDecision {
                action: AiAction::Move { dx, dy },
                aware: true,
            };
        }
        // Can't find a path; idle.
        return AiDecision {
            action: AiAction::Idle,
            aware: true,
        };
    }

    // The monster can't see the player. Wander randomly (with some bias).
    let (dx, dy) = random_direction(rng);
    AiDecision {
        action: AiAction::Move { dx, dy },
        aware: false,
    }
}

/// Maybe use a special ability, based on the monster's abilities and a chance roll.
fn maybe_use_ability(monster: &Monster, rng: &mut Rng) -> Option<crate::data::monsters::Ability> {
    let def = monster.def();
    if def.abilities.is_empty() {
        return None;
    }
    // Abilities have a 30% chance to be used when in range.
    if !rng.chance(30) {
        return None;
    }
    // Pick a random ability from the monster's list.
    rng.pick(def.abilities)
}

/// Compute a single step from `from` toward `goal` using A*.
/// Returns the (dx, dy) of the first step, or None if unreachable.
fn step_toward(level: &Level, from: Pos, goal: Pos) -> Option<(i32, i32)> {
    let result = astar(level, from, goal);
    match result {
        PathResult::Found(path) => {
            if path.len() >= 2 {
                let next = path[1];
                Some((next.x as i32 - from.x as i32, next.y as i32 - from.y as i32))
            } else {
                None
            }
        }
        PathResult::Unreachable => None,
    }
}

/// Pick a random 8-directional delta.
fn random_direction(rng: &mut Rng) -> (i32, i32) {
    let dirs: [(i32, i32); 8] = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];
    let idx = rng.range(0, 8);
    dirs[idx as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::level::{HEIGHT, Level, Tile, WIDTH};

    fn open_level() -> Level {
        let mut level = Level::blank(1);
        // Carve an open floor.
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                level.set_tile(Pos::new(x, y), Tile::Floor);
            }
        }
        level
    }

    #[test]
    fn stunned_monster_idles() {
        let level = open_level();
        let def = crate::data::monsters::MonsterDef::by_id(3).unwrap();
        let mut m = Monster::from_def(def, Pos::new(5, 5));
        m.statuses.apply(crate::status::Status::Paralyzed, 3);
        let mut rng = Rng::new(1);
        let decision = decide(&m, &level, Pos::new(6, 5), &mut rng);
        assert_eq!(decision.action, AiAction::Idle);
    }

    #[test]
    fn adjacent_monster_attacks() {
        let level = open_level();
        let def = crate::data::monsters::MonsterDef::by_id(3).unwrap();
        let m = Monster::from_def(def, Pos::new(5, 5));
        let mut rng = Rng::new(1);
        let decision = decide(&m, &level, Pos::new(6, 5), &mut rng);
        // Should melee attack (or use an ability, but goblin has none).
        assert_eq!(decision.action, AiAction::MeleeAttack);
        assert!(decision.aware);
    }

    #[test]
    fn distant_monster_moves_toward_player() {
        let level = open_level();
        let def = crate::data::monsters::MonsterDef::by_id(3).unwrap();
        let m = Monster::from_def(def, Pos::new(5, 5));
        let mut rng = Rng::new(1);
        let decision = decide(&m, &level, Pos::new(20, 5), &mut rng);
        match decision.action {
            AiAction::Move { dx, dy } => {
                // Should move toward the player (positive x).
                assert!(dx >= 0);
                assert_eq!(dy, 0);
            }
            other => panic!("expected Move, got {:?}", other),
        }
    }

    #[test]
    fn ranged_monster_attacks_at_range() {
        let level = open_level();
        let def = crate::data::monsters::MonsterDef::by_id(20).unwrap(); // harpy
        let m = Monster::from_def(def, Pos::new(5, 5));
        let mut rng = Rng::new(1);
        let decision = decide(&m, &level, Pos::new(10, 5), &mut rng);
        assert!(matches!(
            decision.action,
            AiAction::RangedAttack | AiAction::Ability(_)
        ));
    }

    #[test]
    fn step_toward_returns_valid_step() {
        let level = open_level();
        let from = Pos::new(5, 5);
        let goal = Pos::new(10, 5);
        let step = step_toward(&level, from, goal).unwrap();
        assert_eq!(step, (1, 0));
    }
}
