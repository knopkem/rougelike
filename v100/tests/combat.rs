//! Integration tests: combat resolution invariants.
//!
//! Exercises `deepdelve::combat::resolve_melee` through the public API.
//!
//! The to-hit chance is clamped to `[5, 95]`, so neither a guaranteed hit nor
//! a guaranteed miss is possible. These tests therefore assert *statistical*
//! properties (hit rates) and the exact damage bounds, which follow from the
//! documented formula:
//!
//! - to-hit = clamp(50 + 2*DEX + attack_bonus - AC, 5, 95); hit if d100 <= to-hit
//! - damage = die + (STR/2) + damage_bonus, doubled on a crit

use deepdelve::combat::resolve_melee;
use deepdelve::core::rng::Rng;

/// With overwhelming stats the to-hit clamps to 95, so the hit rate must be
/// high (>= 90%) and every hit deals at least 1 damage.
#[test]
fn high_to_hit_has_high_hit_rate() {
    let mut rng = Rng::new(7);
    let mut hits = 0;
    let trials = 2000;
    for _ in 0..trials {
        let result = resolve_melee(&mut rng, 30, 10, 5, 8, 0, 0, 5);
        if result.hit {
            hits += 1;
            assert!(result.damage >= 1, "damage must be at least 1 on a hit");
        }
    }
    let rate = hits as f64 / trials as f64;
    assert!(rate >= 0.90, "expected hit rate >= 0.90, got {rate}");
}

/// With terrible stats the to-hit clamps to 5, so the hit rate must be low
/// (<= 10%).
#[test]
fn low_to_hit_has_low_hit_rate() {
    let mut rng = Rng::new(7);
    let mut hits = 0;
    let trials = 2000;
    for _ in 0..trials {
        let result = resolve_melee(&mut rng, 0, 1, 0, 1, 0, 100, 0);
        if result.hit {
            hits += 1;
        }
    }
    let rate = hits as f64 / trials as f64;
    assert!(rate <= 0.10, "expected hit rate <= 0.10, got {rate}");
}

/// Damage must be bounded by the die plus the STR and weapon bonuses, doubled
/// on a crit.
#[test]
fn damage_is_bounded() {
    let mut rng = Rng::new(123);
    let die = 8u32;
    let str_stat = 10u32;
    let bonus = 3i32;
    let str_bonus = (str_stat / 2) as i32;
    let max_normal = (die as i32) + str_bonus + bonus;
    let max_crit = max_normal * 2;
    for _ in 0..2000 {
        let result = resolve_melee(&mut rng, 20, str_stat, 0, die, bonus, 10, 5);
        if result.hit {
            assert!(
                (1..=max_crit as u32).contains(&result.damage),
                "damage {} out of range [1, {max_crit}]",
                result.damage
            );
            if !result.crit {
                assert!(
                    result.damage <= max_normal as u32,
                    "non-crit damage {} exceeds max {}",
                    result.damage,
                    max_normal
                );
            }
        }
    }
}

/// A miss deals no damage.
#[test]
fn miss_deals_no_damage() {
    let mut rng = Rng::new(42);
    for _ in 0..2000 {
        let result = resolve_melee(&mut rng, 0, 1, 0, 1, 0, 100, 0);
        if !result.hit {
            assert_eq!(result.damage, 0, "a miss must deal 0 damage");
            assert!(!result.crit, "a miss cannot be a crit");
        }
    }
}

/// The same seed must produce the same combat sequence (determinism).
#[test]
fn combat_is_deterministic_for_a_seed() {
    let mut a = Rng::new(999);
    let mut b = Rng::new(999);
    for _ in 0..50 {
        let ra = resolve_melee(&mut a, 15, 12, 2, 6, 1, 12, 5);
        let rb = resolve_melee(&mut b, 15, 12, 2, 6, 1, 12, 5);
        assert_eq!(ra.hit, rb.hit);
        assert_eq!(ra.crit, rb.crit);
        assert_eq!(ra.damage, rb.damage);
    }
}
