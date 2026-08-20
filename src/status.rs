//! Status effects + hunger states.

use serde::{Deserialize, Serialize};
use crate::core::game::Game;
use crate::entities::player::Player;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Statuses {
    pub poison: u8,
    pub disease: u8,
    pub sleep: u8,
    pub confusion: u8,
    pub paralysis: u8,
    pub blessed: u8,
    pub invisible: u8,
    pub poisoned_turn: bool,
}

impl Statuses {
     pub fn tick(&mut self, player: &mut Player) {
        if self.poison > 0 {
            player.hp = player.hp.saturating_sub(1);
            self.poison = self.poison.saturating_sub(1);
        }
        if self.disease > 0 {
            player.hp = player.hp.saturating_sub(1);
            self.disease = self.disease.saturating_sub(1);
        }
        if self.confusion > 0 {
            self.confusion = self.confusion.saturating_sub(1);
        }
        if self.paralysis > 0 {
            self.paralysis = self.paralysis.saturating_sub(1);
        }
        if self.blessed > 0 {
            self.blessed = self.blessed.saturating_sub(1);
        }
        if self.invisible > 0 {
            self.invisible = self.invisible.saturating_sub(1);
        }
        // Hunger states
        if player.hunger == 0 {
            player.hp = player.hp.saturating_sub(1);
        }
        // Regen when well-fed.
        if player.hunger > 800 && player.hp < player.max_hp {
            player.hp = (player.hp + 1).min(player.max_hp);
        }
    }
}
