//! Integration tests: headless game simulation.
//!
//! Drives `Game::do_turn` with a deterministic action stream and asserts that
//! the core invariants hold for the entire run: the player stays in bounds,
//! the turn counter is monotonic, and the game never panics.

use deepdelve::core::action::{Action, Direction};
use deepdelve::core::game::Game;
use deepdelve::data::classes::{Class, Race};

/// Play a long, deterministic stream of actions and verify invariants.
#[test]
fn long_simulation_preserves_invariants() {
    let mut game = Game::new(1234, Race::Human, Class::Warrior);
    let dirs = Direction::ALL;
    let mut last_turn = game.turn;

    for step in 0..2000u32 {
        if game.is_over() {
            break;
        }
        let action = match step % 10 {
            0..=6 => Action::Move(dirs[(step % 8) as usize]),
            7 => Action::Wait,
            8 => Action::Pickup,
            _ => Action::StairsDown,
        };
        let turn_before = game.turn;
        game.do_turn(action);

        // Invariant: player position always in bounds.
        let p = game.player.pos();
        assert!(
            game.level.in_bounds(p),
            "step {step}: player out of bounds at {p:?}"
        );
        // Invariant: turn counter is monotonic non-decreasing.
        assert!(
            game.turn >= last_turn,
            "step {step}: turn went backwards ({last_turn} -> {})",
            game.turn
        );
        last_turn = game.turn;
        // A non-terminal action that didn't end the game must advance the turn.
        if !game.is_over() {
            assert!(game.turn > turn_before, "step {step}: turn did not advance");
        }
    }
}

/// Descending the stairs (when standing on them) increases depth.
#[test]
fn descending_increases_depth() {
    let mut game = Game::new(5, Race::Human, Class::Warrior);
    // Teleport the player onto the down-stairs and descend.
    if let Some(stairs) = game.level.stairs_down {
        game.player.entity.pos = stairs;
        let depth_before = game.depth;
        game.do_turn(Action::StairsDown);
        assert!(
            game.depth > depth_before,
            "depth should increase after descending ({depth_before} -> {})",
            game.depth
        );
    }
}

/// The game must terminate (death or victory) if the player is left to starve
/// and be attacked, given enough turns.
#[test]
fn idle_player_eventually_dies_or_game_ends() {
    let mut game = Game::new(99, Race::Human, Class::Warrior);
    // Wait a very long time; hunger will eventually kill the player.
    for _ in 0..5000 {
        if game.is_over() {
            break;
        }
        game.do_turn(Action::Wait);
    }
    assert!(
        game.is_over(),
        "player should have died of hunger or monsters within 5000 turns"
    );
}

/// SaveQuit ends the game and sets the save_quit flag.
#[test]
fn save_quit_ends_game() {
    let mut game = Game::new(1, Race::Human, Class::Warrior);
    game.do_turn(Action::SaveQuit);
    assert!(game.is_over());
    assert!(game.save_quit);
}
