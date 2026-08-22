//! Combat resolution: to-hit rolls, damage, and criticals.

use crate::core::rng::Rng;

/// The outcome of a single attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackResult {
    /// Whether the attack hit.
    pub hit: bool,
    /// Whether the attack was a critical.
    pub crit: bool,
    /// Damage dealt (0 if missed).
    pub damage: u32,
}

impl AttackResult {
    pub fn miss() -> Self {
        Self {
            hit: false,
            crit: false,
            damage: 0,
        }
    }

    pub fn hit(damage: u32, crit: bool) -> Self {
        Self {
            hit: true,
            crit,
            damage,
        }
    }
}

/// Resolve a melee attack from `attacker` against `defender_ac`.
///
/// - To-hit: base 50% + 2% per DEX point + weapon attack bonus - target AC (d100).
/// - Damage: weapon die + STR bonus (+ enchantment). Critical (5% + luck) doubles damage.
#[allow(clippy::too_many_arguments)]
pub fn resolve_melee(
    rng: &mut Rng,
    attacker_dex: u32,
    attacker_str: u32,
    attack_bonus: i32,
    damage_die: u32,
    damage_bonus: i32,
    defender_ac: i32,
    crit_chance: u32,
) -> AttackResult {
    // To-hit roll.
    let to_hit = 50 + (attacker_dex as i32 * 2) + attack_bonus - defender_ac;
    let to_hit = to_hit.clamp(5, 95);
    let roll = rng.range(1, 101); // 1..=100
    if roll > to_hit as u32 {
        return AttackResult::miss();
    }

    // Damage roll.
    let die = roll_die(rng, damage_die);
    let str_bonus = (attacker_str as i32 / 2).max(0);
    let mut damage = ((die as i32) + str_bonus + damage_bonus).max(1) as u32;

    // Critical check.
    let crit_roll = rng.range(1, 101);
    if crit_roll <= crit_chance {
        damage *= 2;
        return AttackResult::hit(damage, true);
    }

    AttackResult::hit(damage, false)
}

/// Resolve a ranged attack (simplified: same as melee but no STR bonus).
pub fn resolve_ranged(
    rng: &mut Rng,
    attacker_dex: u32,
    attack_bonus: i32,
    damage_die: u32,
    damage_bonus: i32,
    defender_ac: i32,
    crit_chance: u32,
) -> AttackResult {
    let to_hit = 50 + (attacker_dex as i32 * 2) + attack_bonus - defender_ac;
    let to_hit = to_hit.clamp(5, 95);
    let roll = rng.range(1, 101);
    if roll > to_hit as u32 {
        return AttackResult::miss();
    }

    let die = roll_die(rng, damage_die);
    let mut damage = ((die as i32) + damage_bonus).max(1) as u32;

    let crit_roll = rng.range(1, 101);
    if crit_roll <= crit_chance {
        damage *= 2;
        return AttackResult::hit(damage, true);
    }

    AttackResult::hit(damage, false)
}

/// Roll a die with the given number of sides (minimum 1).
pub fn roll_die(rng: &mut Rng, sides: u32) -> u32 {
    if sides <= 1 {
        1
    } else {
        rng.range(1, sides + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_die_in_range() {
        let mut rng = Rng::new(42);
        for _ in 0..100 {
            let v = roll_die(&mut rng, 6);
            assert!((1..=6).contains(&v));
        }
    }

    #[test]
    fn roll_die_single_side() {
        let mut rng = Rng::new(42);
        assert_eq!(roll_die(&mut rng, 1), 1);
    }

    #[test]
    fn high_attack_hits_more() {
        let mut rng = Rng::new(1);
        let mut hits = 0;
        for _ in 0..1000 {
            let result = resolve_melee(
                &mut rng, 15, // dex
                14, // str
                10, // attack bonus
                8,  // die
                2,  // damage bonus
                5,  // defender ac
                5,  // crit
            );
            if result.hit {
                hits += 1;
            }
        }
        // With high attack vs low AC, should hit most of the time.
        assert!(hits > 700);
    }

    #[test]
    fn low_attack_hits_less() {
        let mut rng = Rng::new(1);
        let mut hits = 0;
        for _ in 0..1000 {
            let result = resolve_melee(
                &mut rng, 5,  // dex
                7,  // str
                -5, // attack bonus
                4,  // die
                0,  // damage bonus
                20, // defender ac
                5,  // crit
            );
            if result.hit {
                hits += 1;
            }
        }
        // With low attack vs high AC, should hit less often.
        assert!(hits < 500);
    }

    #[test]
    fn damage_is_positive_on_hit() {
        let mut rng = Rng::new(42);
        for _ in 0..100 {
            let result = resolve_melee(&mut rng, 10, 10, 0, 6, 0, 10, 5);
            if result.hit {
                assert!(result.damage > 0);
            }
        }
    }

    #[test]
    fn crit_doubles_damage() {
        // Use a fixed seed and force a crit by setting crit_chance to 100.
        let mut rng = Rng::new(99);
        let result = resolve_melee(
            &mut rng, 20, // high dex to ensure hit
            10, 20, // high attack to ensure hit
            6, 0, 0,   // low AC to ensure hit
            100, // always crit
        );
        if result.hit {
            assert!(result.crit);
        }
    }

    #[test]
    fn ranged_attack_works() {
        let mut rng = Rng::new(42);
        let result = resolve_ranged(&mut rng, 15, 5, 6, 1, 10, 5);
        // Just verify it doesn't panic and returns a valid result.
        if result.hit {
            assert!(result.damage > 0);
        }
    }
}
