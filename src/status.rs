//! Status effects + hunger states.

use serde::{Deserialize, Serialize};
use crate::entities::player::Player;

/// Hunger ladder: well-fed → hungry → starving (HP drain) → weak (no regen,
/// -to-hit) → dying (escalated HP drain). The bottom three stages all sit at
/// 0 hunger and escalate with the number of consecutive turns starved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HungerState {
    #[default]
    WellFed,
    Hungry,
    Starving,
    Weak,
    Dying,
}

/// Consecutive turns at 0 hunger before the ladder escalates.
pub const STARVING_WEAK_AFTER: u8 = 30;
pub const STARVING_DYING_AFTER: u8 = 60;

/// To-hit penalty while weak or dying.
pub const HUNGER_TO_HIT_PENALTY: u64 = 10;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Statuses {
    pub poison: u8,
    pub disease: u8,
    pub sleep: u8,
    pub confusion: u8,
    pub paralysis: u8,
    pub blessed: u8,
    pub invisible: u8,
    #[serde(default)]
    pub sickness: u8,
    #[serde(default)]
    pub petrification: u8,
    #[serde(default)]
    pub burn: u8,
    #[serde(default)]
    pub slow: u8,
    pub poisoned_turn: bool,
    /// Consecutive turns the player has spent at 0 hunger; drives the
    /// starving → weak → dying ladder.
    #[serde(default)]
    pub starving_turns: u8,
}

impl Statuses {
    pub fn is_paralyzed(&self) -> bool {
        self.paralysis > 0
    }

    pub fn is_confused(&self) -> bool {
        self.confusion > 0
    }

    pub fn is_asleep(&self) -> bool {
        self.sleep > 0
    }

    pub fn is_petrified(&self) -> bool {
        self.petrification > 0
    }

    pub fn is_invisible(&self) -> bool {
        self.invisible > 0
    }

    pub fn is_slowed(&self) -> bool {
        self.slow > 0
    }

    pub fn is_blessed(&self) -> bool {
        self.blessed > 0
    }

    /// The hunger ladder stage for the given hunger value and the number of
    /// consecutive turns spent at 0 hunger. Only the zero-hunger stages
    /// (Starving/Weak/Dying) drain HP.
    pub fn hunger_state(hunger: u16, starving_turns: u8) -> HungerState {
        if hunger > 400 {
            HungerState::WellFed
        } else if hunger > 0 {
            HungerState::Hungry
        } else if starving_turns < STARVING_WEAK_AFTER {
            HungerState::Starving
        } else if starving_turns < STARVING_DYING_AFTER {
            HungerState::Weak
        } else {
            HungerState::Dying
        }
    }

    /// To-hit penalty from the hunger ladder (weak and dying).
    pub fn hunger_to_hit_penalty(&self, hunger: u16) -> u64 {
        let stage = Self::hunger_state(hunger, self.starving_turns);
        match stage {
            HungerState::Weak | HungerState::Dying => HUNGER_TO_HIT_PENALTY,
            _ => 0,
        }
    }

    /// One turn of status processing. Applies damage (poison, disease,
    /// sickness, burn, petrification, hunger ladder), lets poison fester
    /// into disease, drains durations, and tracks the starving ladder.
    /// Returns the cause of death if this turn's damage killed the player.
    pub fn tick(&mut self, player: &mut Player) -> Option<crate::core::game::DeathCause> {
        // Capture durations before draining: a 1-turn status still acts.
        let (poison, disease, sickness, burn, petrification) = (
            self.poison,
            self.disease,
            self.sickness,
            self.burn,
            self.petrification,
        );

        // Duration drain.
        self.poison = self.poison.saturating_sub(1);
        self.disease = self.disease.saturating_sub(1);
        self.sickness = self.sickness.saturating_sub(1);
        self.petrification = self.petrification.saturating_sub(1);
        self.burn = self.burn.saturating_sub(1);
        self.sleep = self.sleep.saturating_sub(1);
        self.confusion = self.confusion.saturating_sub(1);
        self.paralysis = self.paralysis.saturating_sub(1);
        self.blessed = self.blessed.saturating_sub(1);
        self.invisible = self.invisible.saturating_sub(1);
        self.slow = self.slow.saturating_sub(1);

        // Poison festers into disease as it wears off.
        if poison == 1 {
            self.disease = self.disease.saturating_add(1);
        }

        // Tick damage, in order: poison, disease, sickness, burn,
        // petrification, starvation. The last damage dealt is the cause of
        // death if HP hits 0.
        let mut cause: Option<crate::core::game::DeathCause> = None;
        let mut deal = |player: &mut Player, amount: u8, death: crate::core::game::DeathCause| {
            player.hp = player.hp.saturating_sub(amount);
            if player.hp == 0 {
                cause = Some(death);
            }
        };
        if poison > 0 {
            deal(player, 1, crate::core::game::DeathCause::Poisoned);
        }
        if disease > 0 {
            deal(player, 1, crate::core::game::DeathCause::Other);
        }
        if sickness > 0 {
            deal(player, 1, crate::core::game::DeathCause::Other);
        }
        if burn > 0 {
            deal(player, 1, crate::core::game::DeathCause::Burned);
        }
        if petrification > 0 {
            deal(player, 1, crate::core::game::DeathCause::Petrified);
        }

        // Hunger ladder.
        if player.hunger == 0 {
            self.starving_turns = self.starving_turns.saturating_add(1);
        } else {
            self.starving_turns = 0;
        }
        let stage = Self::hunger_state(player.hunger, self.starving_turns);
        match stage {
            HungerState::WellFed => {
                // Regen when well-fed.
                if player.hp < player.max_hp {
                    player.hp = (player.hp + 1).min(player.max_hp);
                }
            }
            HungerState::Starving => deal(player, 1, crate::core::game::DeathCause::Starved),
            HungerState::Dying => deal(player, 2, crate::core::game::DeathCause::Starved),
            _ => {}
        }

        cause
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game::DeathCause;
    use crate::entities::player::Player;

    fn player() -> Player {
        Player::new("T", "Human", "Warrior")
    }

    #[test]
    fn durations_drain_by_one_per_tick() {
        let mut s = Statuses::default();
        s.poison = 2;
        s.disease = 2;
        s.sickness = 2;
        s.petrification = 2;
        s.burn = 2;
        s.sleep = 2;
        s.confusion = 2;
        s.paralysis = 2;
        s.blessed = 2;
        s.invisible = 2;
        s.slow = 2;
        let mut p = player();
        p.hp = 100;
        p.hunger = 300; // Hungry: no regen, no ladder damage
        s.tick(&mut p);
        assert_eq!(s.poison, 1);
        assert_eq!(s.disease, 1); // drained; no fester yet (poison was 2)
        assert_eq!(s.sickness, 1);
        assert_eq!(s.petrification, 1);
        assert_eq!(s.burn, 1);
        assert_eq!(s.sleep, 1);
        assert_eq!(s.confusion, 1);
        assert_eq!(s.paralysis, 1);
        assert_eq!(s.blessed, 1);
        assert_eq!(s.invisible, 1);
        assert_eq!(s.slow, 1);
    }

    #[test]
    fn poison_ticks_damage_and_festors_into_disease() {
        let mut s = Statuses::default();
        s.poison = 1;
        let mut p = player();
        p.hp = 1;
        p.hunger = 300; // Hungry: no regen to offset the tick damage
        let cause = s.tick(&mut p);
        assert_eq!(p.hp, 0);
        assert_eq!(cause, Some(DeathCause::Poisoned));
        assert_eq!(s.poison, 0);
        assert_eq!(s.disease, 1, "poison should fester into disease");
    }

    #[test]
    fn curing_poison_stops_ticks() {
        let mut s = Statuses::default();
        s.poison = 5;
        s.poison = 0; // e.g. cured by a potion
        let mut p = player();
        let hp = p.hp;
        let cause = s.tick(&mut p);
        assert_eq!(p.hp, hp);
        assert_eq!(cause, None);
        assert_eq!(s.disease, 0, "no fester without an active poison tick");
    }

    #[test]
    fn disease_and_sickness_tick_damage() {
        let mut s = Statuses::default();
        s.disease = 2;
        s.sickness = 2;
        let mut p = player();
        p.hp = 2;
        p.hunger = 300; // Hungry: no regen to offset the tick damage
        let cause = s.tick(&mut p);
        assert_eq!(p.hp, 0);
        assert_eq!(cause, Some(DeathCause::Other));
    }

    #[test]
    fn burn_and_petrification_tick_damage_with_their_causes() {
        let mut s = Statuses::default();
        s.burn = 1;
        s.petrification = 1;
        let mut p = player();
        p.hp = 2;
        p.hunger = 300; // Hungry: no regen to offset the tick damage
        let cause = s.tick(&mut p);
        assert_eq!(p.hp, 0);
        // Petrification deals damage after burn, so its cause wins.
        assert_eq!(cause, Some(DeathCause::Petrified));
    }

    #[test]
    fn burn_ticks_damage_with_its_own_cause() {
        let mut s = Statuses::default();
        s.burn = 1;
        let mut p = player();
        p.hp = 1;
        p.hunger = 300;
        let cause = s.tick(&mut p);
        assert_eq!(p.hp, 0);
        assert_eq!(cause, Some(DeathCause::Burned));
    }

    #[test]
    fn hunger_ladder_starts_well_fed() {
        assert_eq!(
            Statuses::hunger_state(1000, 0),
            HungerState::WellFed
        );
        assert_eq!(Statuses::hunger_state(401, 0), HungerState::WellFed);
        assert_eq!(Statuses::hunger_state(400, 0), HungerState::Hungry);
        assert_eq!(Statuses::hunger_state(100, 0), HungerState::Hungry);
        assert_eq!(Statuses::hunger_state(1, 0), HungerState::Hungry);
        assert_eq!(Statuses::hunger_state(0, 0), HungerState::Starving);
        assert_eq!(Statuses::hunger_state(0, 10), HungerState::Starving);
        assert_eq!(Statuses::hunger_state(0, STARVING_WEAK_AFTER), HungerState::Weak);
        assert_eq!(
            Statuses::hunger_state(0, STARVING_DYING_AFTER),
            HungerState::Dying
        );
        assert_eq!(
            Statuses::hunger_state(0, u8::MAX),
            HungerState::Dying
        );
    }

    #[test]
    fn starving_counter_resets_when_fed() {
        let mut s = Statuses::default();
        s.starving_turns = 20;
        let mut p = player();
        p.hunger = 1000;
        s.tick(&mut p);
        assert_eq!(s.starving_turns, 0);
    }

    #[test]
    fn starving_escalates_with_escalating_damage() {
        let mut s = Statuses::default();
        let mut p = player();
        p.hunger = 0;
        let hp = p.hp;
        // Starving stage: 1 HP.
        s.tick(&mut p);
        assert_eq!(p.hp, hp - 1);
        assert_eq!(s.starving_turns, 1);
        // Force weak stage: no extra HP loss, but to-hit is penalized.
        s.starving_turns = STARVING_WEAK_AFTER;
        s.tick(&mut p);
        assert_eq!(p.hp, hp - 1, "weak stage drains no HP");
        assert_eq!(s.hunger_to_hit_penalty(p.hunger), HUNGER_TO_HIT_PENALTY);
        // Force dying stage: 2 HP.
        s.starving_turns = STARVING_DYING_AFTER;
        s.tick(&mut p);
        assert_eq!(p.hp, hp - 3, "dying stage drains 2 HP");
        assert_eq!(s.hunger_to_hit_penalty(p.hunger), HUNGER_TO_HIT_PENALTY);
        s.tick(&mut p);
        assert_eq!(p.hp, hp - 5, "dying stage keeps draining 2 HP");
    }

    #[test]
    fn well_fed_player_regen() {
        let mut s = Statuses::default();
        let mut p = player();
        p.hunger = 1000;
        p.hp = p.max_hp - 3;
        s.tick(&mut p);
        assert_eq!(p.hp, p.max_hp - 2);
        assert_eq!(s.tick(&mut p), None);
        assert_eq!(p.hp, p.max_hp - 1);
        assert_eq!(s.tick(&mut p), None);
        assert_eq!(p.hp, p.max_hp);
        // Capped at max.
        s.tick(&mut p);
        assert_eq!(p.hp, p.max_hp);
    }

    #[test]
    fn old_save_without_new_fields_deserializes() {
        // Pre-issue-18 saves lack sickness/petrification/burn/slow/starving_turns.
        let s: Statuses = serde_json::from_str(
            r#"{"poison":2,"disease":1,"sleep":0,"confusion":0,"paralysis":0,"blessed":0,"invisible":0,"poisoned_turn":false}"#,
        )
        .unwrap();
        assert_eq!(s.poison, 2);
        assert_eq!(s.disease, 1);
        assert_eq!(s.sickness, 0);
        assert_eq!(s.petrification, 0);
        assert_eq!(s.burn, 0);
        assert_eq!(s.slow, 0);
        assert_eq!(s.starving_turns, 0);
    }

    #[test]
    fn death_cause_is_the_last_tick_damage() {
        let mut s = Statuses::default();
        s.poison = 1;
        s.disease = 1;
        let mut p = player();
        p.hp = 1;
        p.hunger = 300; // Hungry: no regen to revive a fatal tick
        let cause = s.tick(&mut p);
        assert_eq!(p.hp, 0);
        assert_eq!(cause, Some(DeathCause::Other), "disease deals damage after poison");
    }
}
