//! Integration tests: save/load round-trip fidelity through the public API.
//!
//! Uses `Game::to_save` / `Game::from_save` and the JSON (de)serialization of
//! `SaveData`. File I/O is exercised via `deepdelve::save` against a temp dir
//! is avoided (it writes to the user data dir); instead we round-trip through
//! `serde_json` directly, which is the same path `save_game`/`load_game` use.

use deepdelve::core::game::Game;
use deepdelve::data::classes::{Class, Race};
use deepdelve::save::SaveData;

/// A game serialized to JSON and back must preserve the key state.
#[test]
fn save_roundtrip_preserves_state() {
    let mut game = Game::new(42, Race::Elf, Class::Mage);
    // Advance a few turns so the RNG state and turn counter change.
    for _ in 0..5 {
        game.do_turn(deepdelve::core::action::Action::Wait);
    }

    let data = game.to_save();
    let json = serde_json::to_string(&data).expect("serialize");
    let loaded: SaveData = serde_json::from_str(&json).expect("deserialize");
    assert!(loaded.is_compatible(), "save version must be compatible");

    let restored = Game::from_save(loaded);
    assert_eq!(restored.seed, game.seed);
    assert_eq!(restored.depth, game.depth);
    assert_eq!(restored.turn, game.turn);
    assert_eq!(restored.player.pos(), game.player.pos());
    assert_eq!(restored.player.hp(), game.player.hp());
    assert_eq!(restored.player.gold, game.player.gold);
    assert_eq!(restored.player.inventory.len(), game.player.inventory.len());
    assert_eq!(restored.endless, game.endless);
}

/// The RNG state must round-trip so that future randomness is reproducible.
#[test]
fn save_roundtrip_preserves_rng() {
    let mut game = Game::new(7, Race::Human, Class::Warrior);
    let data = game.to_save();
    let json = serde_json::to_string(&data).expect("serialize");
    let loaded: SaveData = serde_json::from_str(&json).expect("deserialize");
    let mut restored = Game::from_save(loaded);

    // Both the original and restored games should now produce identical
    // sequences from their (equal) RNG states.
    let a = game.rng.range(0, 1_000_000);
    let b = restored.rng.range(0, 1_000_000);
    assert_eq!(a, b, "RNG state must be preserved across save/load");
}

/// A save with a bumped version must be rejected as incompatible.
#[test]
fn incompatible_version_is_rejected() {
    let game = Game::new(1, Race::Human, Class::Warrior);
    let mut data = game.to_save();
    data.version = 999;
    let json = serde_json::to_string(&data).expect("serialize");
    let loaded: SaveData = serde_json::from_str(&json).expect("deserialize");
    assert!(
        !loaded.is_compatible(),
        "bumped version must be incompatible"
    );
}
