//! Wand/potion/scroll effects.

use crate::core::game::Game;
use crate::core::rng::Rng;
use crate::items::item::Item;
use crate::items::item::{PotionKind, ScrollKind, WandKind};

pub fn cast_wand(rng: &mut Rng, game: &mut Game, item: &Item, target: (u8, u8)) {
    match item.kind {
        crate::items::item::ItemKind::Wand(WandKind::FireBolt) => {
            // Damage all monsters in a line from player to target.
            let p = game.player.pos;
            let mut x = p.0;
            let mut y = p.1;
            let dx = if target.0 >= p.0 { 1i8 } else { -1i8 };
            let dy = if target.1 >= p.1 { 1i8 } else { -1i8 };
            let mut steps = 0;
            while steps < 20 {
                x = (x as i8 + dx) as u8;
                y = (y as i8 + dy) as u8;
                steps += 1;
                if x == target.0 && y == target.1 {
                    break;
                }
                if let Some(idx) = game.monsters.iter().position(|m| m.pos == (x, y) && !m.dead) {
                    let dmg = rng.int(3..8) as u8;
                    game.monsters[idx].hp = game.monsters[idx].hp.saturating_sub(dmg);
                    if game.monsters[idx].hp == 0 {
                        game.log(
                            crate::core::message::MessageKind::Combat,
                            format!("The firebolt burns {}!", game.monsters[idx].name),
                        );
                    }
                }
            }
            game.log(
                crate::core::message::MessageKind::Normal,
                "You hurl a bolt of fire!",
            );
        }
        crate::items::item::ItemKind::Wand(WandKind::Healing) => {
            let heal = rng.int(5..15) as u8;
            game.player.hp = (game.player.hp + heal).min(game.player.max_hp);
            game.log(
                crate::core::message::MessageKind::Good,
                format!("The wand restores {heal} HP."),
            );
        }
        crate::items::item::ItemKind::Wand(WandKind::Sleep) => {
            if let Some(idx) = game
                .monsters
                .iter()
                .position(|m| m.pos == target && !m.dead)
            {
                game.log(
                    crate::core::message::MessageKind::Normal,
                    format!("{} falls asleep.", game.monsters[idx].name),
                );
            }
        }
        _ => {
            game.log(
                crate::core::message::MessageKind::Normal,
                format!("You use the {}.", item.name()),
            );
        }
    }
}

pub fn apply_potion_full(game: &mut Game, kind: PotionKind) {
    match kind {
        PotionKind::Healing(small) => {
            let heal = if small { 10u8 } else { 30 };
            game.player.hp = (game.player.hp + heal).min(game.player.max_hp);
            game.log(
                crate::core::message::MessageKind::Good,
                format!("You feel better. (+{heal} HP)"),
            );
        }
        PotionKind::CurePoison => {
            game.statuses.poison = 0;
            game.log(
                crate::core::message::MessageKind::Good,
                "The poison fades.",
            );
        }
        PotionKind::Energy => {
            game.player.ep = game.player.max_ep;
            game.log(
                crate::core::message::MessageKind::Good,
                "You feel a surge of energy.",
            );
        }
        _ => {
            game.log(
                crate::core::message::MessageKind::Normal,
                "You drink the potion.",
            );
        }
    }
}

pub fn apply_scroll_full(game: &mut Game, kind: ScrollKind) {
    match kind {
        ScrollKind::Teleport => {
            game.emit(crate::core::events::GameEvent::Teleport);
            game.log(
                crate::core::message::MessageKind::Normal,
                "You blink out of reality!",
            );
        }
        _ => {
            game.log(
                crate::core::message::MessageKind::Normal,
                "You read the scroll.",
            );
        }
    }
}
