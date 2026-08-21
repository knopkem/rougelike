//! Combat: hit calc, damage rolls, crits.

use crate::core::rng::Rng;
use crate::entities::monster::Monster;
use crate::entities::player::Player;

pub struct HitResult {
    pub hit: bool,
    pub damage: u8,
    pub crit: bool,
}

pub struct Combat {
    rng: Rng,
}

impl Combat {
    pub fn new(rng: Rng) -> Self {
        Self { rng }
    }

    pub fn player_attacks(&mut self, p: &Player, m: &Monster) -> HitResult {
        let to_hit = p.to_hit();
        let roll = self.rng.int(0..100);
        let hit = roll < to_hit;
        if !hit {
            return HitResult {
                hit: false,
                damage: 0,
                crit: false,
            };
        }
        // Weapon damage die, in faces (a d4 rolls 1..=4); unarmed fights
        // with a 1d4. The monster side keeps its legacy roll convention.
        let die = p
            .wielded
            .as_ref()
            .map(|w| w.defense)
            .unwrap_or(4);
        let mut dmg = self.rng.int_inclusive(1..=die as u64) as u8;
        let crit = self.rng.chance(p.crit_chance());
        if crit {
            dmg *= 2;
        }
        let ac = m.def.ac;
        let dmg = dmg.saturating_sub((ac / 2) as u8);
        // Weapon enchant is a flat damage bonus, applied after AC reduction.
        let enchant = p
            .wielded
            .as_ref()
            .map(|w| w.enchant as i32)
            .unwrap_or(0);
        let dmg = (dmg as i32 + enchant).max(0) as u8;
        HitResult {
            hit: true,
            damage: dmg,
            crit,
        }
    }

    pub fn monster_attacks(&mut self, m: &Monster, p: &Player) -> HitResult {
        let to_hit = 50 + m.def.attack as u64;
        let roll = self.rng.int(0..100);
        let hit = roll < to_hit;
        if !hit {
            return HitResult {
                hit: false,
                damage: 0,
                crit: false,
            };
        }
        let dmg = self.rng.int(1..m.def.damage_die as u64) as u8;
        let ac = p.ac();
        let mut dmg = dmg.saturating_sub((ac / 2) as u8);
        let crit = self.rng.chance(5);
        if crit {
            dmg *= 2;
        }
        HitResult {
            hit: true,
            damage: dmg,
            crit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::monsters::MONSTERS;
    use crate::entities::monster::Monster;
    use crate::entities::player::Player;
    use crate::items::catalog::make_weapon;
    use crate::items::item::WeaponKind;

    #[test]
    fn player_can_hit() {
        let rng = Rng::new(42);
        let p = Player::new("T", "Human", "Warrior");
        let m = Monster::new(MONSTERS[0].clone(), (5, 5));
        let mut c = Combat::new(rng);
        let r = c.player_attacks(&p, &m);
        assert!(r.hit || !r.hit);
    }

    fn first_hit(c: &mut Combat, p: &Player, m: &Monster) -> HitResult {
        for _ in 0..1000 {
            let r = c.player_attacks(p, m);
            if r.hit {
                return r;
            }
        }
        panic!("attack never hit in 1000 tries");
    }

    #[test]
    fn unarmed_player_rolls_1d4_damage() {
        let mut c = Combat::new(Rng::new(7));
        let p = Player::new("T", "Human", "Warrior");
        assert!(p.wielded.is_none());
        let m = Monster::new(MONSTERS[0].clone(), (5, 5)); // ac 1 -> subtracts 0
        let r = first_hit(&mut c, &p, &m);
        let expected = if r.crit { 2..=8 } else { 1..=4 };
        assert!(
            expected.contains(&r.damage),
            "unarmed damage {} not 1d4 (crit: {})",
            r.damage,
            r.crit
        );
    }

    #[test]
    fn wielded_weapon_rolls_its_die_plus_enchant_bonus() {
        let mut c = Combat::new(Rng::new(7));
        let mut p = Player::new("T", "Human", "Warrior");
        let mut rng = Rng::new(9);
        let flail = make_weapon(WeaponKind::WarFlail, 10, false, &mut rng);
        assert_eq!(flail.defense, 5, "war flail is a d5");
        p.wielded = Some(flail);
        let m = Monster::new(MONSTERS[0].clone(), (5, 5)); // ac 1 -> subtracts 0
        let mut saw_max_die = false;
        for _ in 0..500 {
            let r = c.player_attacks(&p, &m);
            if !r.hit {
                continue;
            }
            let die_roll = r.damage as i32 - 10;
            let expected = if r.crit { 2..=10 } else { 1..=5 };
            assert!(
                expected.contains(&(die_roll as u8)),
                "die roll {} from a +10 d5 weapon not 1d5 (crit: {})",
                die_roll,
                r.crit
            );
            if !r.crit && die_roll == 5 {
                saw_max_die = true;
            }
        }
        assert!(
            saw_max_die,
            "a d5 weapon must be able to roll 5 (seeded, deterministic)"
        );
    }
}
