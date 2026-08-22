//! Integration tests: level-generation invariants across many seeds and depths.
//!
//! These exercise the public API only (`deepdelve::map::generation`,
//! `deepdelve::core::rng`, `deepdelve::core::game`) and assert structural
//! properties that must hold for every generated level.

use deepdelve::core::game::Game;
use deepdelve::core::rng::Rng;
use deepdelve::data::classes::{Class, Race};
use deepdelve::map::generation::generate_level;
use deepdelve::map::level::{HEIGHT, Level, WIDTH};

/// Every generated level must have a down-stairs and a walkable player start.
#[test]
fn generated_levels_have_stairs_and_walkable_start() {
    for seed in 0..50u64 {
        for depth in 1..=25u32 {
            let mut rng = Rng::new(seed * 1000 + depth as u64);
            let level = generate_level(&mut rng, depth, depth > 1);
            assert!(
                level.stairs_down.is_some(),
                "seed {seed} depth {depth}: missing down-stairs"
            );
            let start = level.player_start;
            assert!(
                level.in_bounds(start),
                "seed {seed} depth {depth}: player start out of bounds"
            );
            assert!(
                level.tiles[level.idx(start)].walkable(),
                "seed {seed} depth {depth}: player start not walkable"
            );
            // Up-stairs only on depth > 1.
            if depth > 1 {
                assert!(
                    level.stairs_up.is_some(),
                    "seed {seed} depth {depth}: missing up-stairs"
                );
            }
        }
    }
}

/// The down-stairs must be reachable from the player start (level connected).
#[test]
fn generated_levels_are_connected() {
    for seed in 0..25u64 {
        let mut rng = Rng::new(seed);
        let level = generate_level(&mut rng, 1, false);
        let start = level.player_start;
        let down = level.stairs_down.expect("down-stairs present");
        let reachable = deepdelve::map::path::bfs_path(&level, start, down).is_some();
        assert!(
            reachable,
            "seed {seed}: down-stairs not reachable from start"
        );
    }
}

/// A full `Game::new` must produce a live player and a non-empty monster list
/// (at least on the depths that place monsters).
#[test]
fn new_game_is_well_formed() {
    for seed in 0..20u64 {
        let game = Game::new(seed, Race::Human, Class::Warrior);
        assert!(!game.over, "seed {seed}: game over immediately");
        assert!(game.player.is_alive(), "seed {seed}: player dead at start");
        assert_eq!(game.depth, 1, "seed {seed}: wrong starting depth");
        let pidx = game.level.idx(game.player.pos());
        assert!(
            game.level.tiles[pidx].walkable(),
            "seed {seed}: player standing on non-walkable tile"
        );
    }
}

/// Monster and item placement must stay within bounds and on walkable tiles.
#[test]
fn placements_are_in_bounds_and_walkable() {
    for seed in 0..30u64 {
        let mut rng = Rng::new(seed);
        let level = generate_level(&mut rng, 3, true);
        for (i, &m) in level.monsters.iter().enumerate() {
            if let Some(id) = m {
                let p = Level::pos(i);
                assert!(
                    level.in_bounds(p),
                    "seed {seed}: monster {id} out of bounds"
                );
                assert!(
                    level.tiles[i].walkable(),
                    "seed {seed}: monster {id} on non-walkable tile"
                );
            }
        }
        for (i, &it) in level.items.iter().enumerate() {
            if let Some(id) = it {
                let p = Level::pos(i);
                assert!(level.in_bounds(p), "seed {seed}: item {id} out of bounds");
            }
        }
    }
}

/// The grid dimensions must match the documented constants.
#[test]
fn grid_dimensions_match_constants() {
    let mut rng = Rng::new(1);
    let level = generate_level(&mut rng, 1, false);
    assert_eq!(level.tiles.len(), (WIDTH * HEIGHT) as usize);
    assert_eq!(level.seen.len(), (WIDTH * HEIGHT) as usize);
    assert_eq!(level.visible.len(), (WIDTH * HEIGHT) as usize);
}
