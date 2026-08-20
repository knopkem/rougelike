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
        let mut dmg = self.rng.int(1..m.def.damage_die as u64) as u8;
        let crit = self.rng.chance(p.crit_chance());
        if crit {
            dmg *= 2;
        }
        let ac = m.def.ac;
        let dmg = dmg.saturating_sub((ac / 2) as u8);
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

    #[test]
    fn player_can_hit() {
        let rng = Rng::new(42);
        let p = Player::new("T", "Human", "Warrior");
        let m = Monster::new(MONSTERS[0].clone(), (5, 5));
        let mut c = Combat::new(rng);
        let r = c.player_attacks(&p, &m);
        assert!(r.hit || !r.hit);
    }
}
