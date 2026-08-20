//! Score calculation: gold + XP + depth + quest bonuses.

use super::game::Game;

pub fn compute(game: &Game) -> u64 {
    let mut score = 0u64;
    score += game.player.gold as u64;
    score += game.player.xp as u64;
    score += (game.current_level as u64) * 100;
    score += game.player.kills as u64 * 10;
    for _ in &game.quests.completed {
        score += 500;
    }
    if game.won {
        score += 10_000;
    }
    score
}
