//! Wand/potion/scroll effects.

use crate::core::game::Game;
use crate::core::rng::Rng;
use crate::items::item::Item;
use crate::items::item::{PotionKind, ScrollKind, WandKind};

pub fn cast_wand(rng: &mut Rng, game: &mut Game, item: &Item, target: (u8, u8)) {
    match item.kind {
        crate::items::item::ItemKind::Wand(WandKind::FireBolt) => {
            // A firebolt can scorch the caster.
            if rng.chance(10) {
                game.statuses.burn = 3;
                game.log(
                    crate::core::message::MessageKind::Bad,
                    "The firebolt scorches your hand!",
                );
            }
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
                if let Some(idx) = game
                    .monsters
                    .iter()
                    .position(|m| m.pos == (x, y) && !m.dead)
                {
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
            if rng.chance(60) {
                game.statuses.sleep = 6;
                game.log(
                    crate::core::message::MessageKind::Bad,
                    "The drowsiness takes hold of you!",
                );
            }
        }
        crate::items::item::ItemKind::Wand(WandKind::Confusion) => {
            if rng.chance(80) {
                game.statuses.confusion = 8;
                game.log(
                    crate::core::message::MessageKind::Bad,
                    "Your thoughts swirl in knots!",
                );
            } else if game.statuses.is_blessed() && rng.chance(50) {
                game.log(
                    crate::core::message::MessageKind::Good,
                    "A blessing protects you.",
                );
            }
        }
        crate::items::item::ItemKind::Wand(WandKind::Paralyze) => {
            if rng.chance(60) {
                game.statuses.paralysis = 8;
                game.log(
                    crate::core::message::MessageKind::Bad,
                    "Your limbs go rigid!",
                );
            } else if game.statuses.is_blessed() && rng.chance(50) {
                game.log(
                    crate::core::message::MessageKind::Good,
                    "A blessing protects you.",
                );
            }
        }
        crate::items::item::ItemKind::Wand(WandKind::CurePoison) => {
            game.statuses.poison = 0;
            game.log(crate::core::message::MessageKind::Good, "The poison fades.");
        }
        crate::items::item::ItemKind::Wand(WandKind::Blink) => {
            // The player is briefly turned to stone as reality reassembles.
            if rng.chance(if game.statuses.is_blessed() { 10 } else { 30 }) {
                game.statuses.petrification = 3;
                game.log(
                    crate::core::message::MessageKind::Bad,
                    "You flash of stone as reality reassembles!",
                );
            }
        }
        crate::items::item::ItemKind::Wand(WandKind::Lightning) => {
            if let Some(idx) = game
                .monsters
                .iter()
                .position(|m| m.pos == target && !m.dead)
            {
                let dmg = rng.int(4..10) as u8;
                game.monsters[idx].hp = game.monsters[idx].hp.saturating_sub(dmg);
                if game.monsters[idx].hp == 0 {
                    game.log(
                        crate::core::message::MessageKind::Combat,
                        format!("The lightning strikes {}!", game.monsters[idx].name),
                    );
                }
            } else {
                game.log(
                    crate::core::message::MessageKind::Normal,
                    "The lightning fizzles out.",
                );
            }
            if rng.chance(if game.statuses.is_blessed() { 10 } else { 50 }) {
                game.statuses.burn = 4;
                game.log(
                    crate::core::message::MessageKind::Bad,
                    "The bolt backfires; your skin smolders!",
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

pub fn apply_potion(game: &mut Game, kind: PotionKind) {
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
            game.statuses.disease = 0;
            game.log(crate::core::message::MessageKind::Good, "The poison fades.");
        }
        PotionKind::Energy => {
            game.player.ep = game.player.max_ep;
            game.log(
                crate::core::message::MessageKind::Good,
                "You feel a surge of energy.",
            );
            if game.rng.chance(50) {
                game.statuses.blessed = 10;
                game.log(
                    crate::core::message::MessageKind::Good,
                    "You feel a blessing upon you.",
                );
            }
        }
        PotionKind::Restore => {
            game.statuses.poison = 0;
            game.statuses.disease = 0;
            game.statuses.sickness = 0;
            game.statuses.confusion = 0;
            game.statuses.paralysis = 0;
            game.statuses.petrification = 0;
            game.statuses.burn = 0;
            game.statuses.slow = 0;
            game.log(
                crate::core::message::MessageKind::Good,
                "You feel restored.",
            );
        }
        PotionKind::Identify => identify_items(game),
        PotionKind::Invisibility => {
            game.statuses.invisible = 10;
            game.log(
                crate::core::message::MessageKind::Good,
                "You become invisible!",
            );
        }
        PotionKind::Antidote => {
            game.log(
                crate::core::message::MessageKind::Normal,
                "It tastes bad. (It was just water.)",
            );
        }
        PotionKind::Mutation => {
            let step = |v: u8, up: bool| {
                if up {
                    v.saturating_add(1)
                } else {
                    v.saturating_sub(1)
                }
            };
            let idx = game.rng.int(0..6) as usize;
            let up = game.rng.chance(50);
            let label = match idx {
                0 => "STR",
                1 => "DEX",
                2 => "CON",
                3 => "INT",
                4 => "WIS",
                _ => "CHA",
            };
            match idx {
                0 => game.player.str = step(game.player.str, up),
                1 => game.player.dex = step(game.player.dex, up),
                2 => game.player.con = step(game.player.con, up),
                3 => game.player.int = step(game.player.int, up),
                4 => game.player.wis = step(game.player.wis, up),
                _ => game.player.cha = step(game.player.cha, up),
            };
            if up {
                game.log(
                    crate::core::message::MessageKind::Good,
                    format!("Your body shifts. ({label} +1)"),
                );
            } else {
                game.log(
                    crate::core::message::MessageKind::Bad,
                    format!("Your body shifts. ({label} -1)"),
                );
            }
            if game.rng.chance(25) {
                game.statuses.sickness = 5;
                game.log(
                    crate::core::message::MessageKind::Bad,
                    "Your body feels sick.",
                );
            }
        }
    }
}

pub fn apply_scroll(game: &mut Game, kind: ScrollKind) {
    match kind {
        ScrollKind::Identify => identify_items(game),
        ScrollKind::Teleport => {
            let before = game.player.pos;
            let level = game.current();
            let candidates: Vec<(u8, u8)> = level
                .floor_tiles()
                .into_iter()
                .filter(|p| *p != before)
                .filter(|p| !game.monsters.iter().any(|m| m.pos == *p && !m.dead))
                .collect();
            match game.rng.pick(&candidates) {
                Some(pos) => {
                    game.player.pos = pos;
                    game.emit(crate::core::events::GameEvent::Teleport);
                    game.log(
                        crate::core::message::MessageKind::Normal,
                        "You blink out of reality!",
                    );
                    if game
                        .rng
                        .chance(if game.statuses.is_blessed() { 8 } else { 20 })
                    {
                        game.statuses.petrification = 3;
                        game.log(
                            crate::core::message::MessageKind::Bad,
                            "You flash of stone for a moment!",
                        );
                    }
                }
                None => game.log(
                    crate::core::message::MessageKind::Normal,
                    "Nothing happens.",
                ),
            }
        }
        ScrollKind::EnchantWeapon => {
            let Some(item) = game.player.wielded.as_mut() else {
                game.log(
                    crate::core::message::MessageKind::Normal,
                    "You are not wielding a weapon.",
                );
                return;
            };
            item.enchant = item.enchant.saturating_add(1);
            game.log(
                crate::core::message::MessageKind::Good,
                "The weapon shimmers.",
            );
        }
        ScrollKind::EnchantArmor => {
            let Some(item) = game.player.armor.as_mut() else {
                game.log(
                    crate::core::message::MessageKind::Normal,
                    "You are not wearing any armor.",
                );
                return;
            };
            item.defense = item.defense.saturating_add(1);
            game.log(
                crate::core::message::MessageKind::Good,
                "The armor shimmers.",
            );
        }
        ScrollKind::RemoveCurse => {
            for item in game.player.inventory.iter_mut() {
                item.cursed = false;
            }
            if let Some(item) = game.player.wielded.as_mut() {
                item.cursed = false;
            }
            if let Some(item) = game.player.armor.as_mut() {
                item.cursed = false;
            }
            for item in game.player.rings.iter_mut() {
                item.cursed = false;
            }
            game.log(
                crate::core::message::MessageKind::Good,
                "A weight lifts from your items.",
            );
        }
        ScrollKind::Mapping => {
            let level = game.current_mut();
            for (i, t) in level.tiles.iter().enumerate() {
                if *t != crate::map::level::Tile::Wall {
                    level.explored[i] = true;
                }
            }
            game.log(
                crate::core::message::MessageKind::Good,
                "The level layout becomes clear to you.",
            );
        }
        ScrollKind::GodsMessage => {
            game.log(
                crate::core::message::MessageKind::System,
                "The god whispers: 'The Amulet rests in the deep dark.'",
            );
        }
        ScrollKind::Opening => {
            // Opens a closed door under or adjacent to the player; a locked
            // door is unlocked with an iron key (consumed).
            let pos = game.player.pos;
            let (px, py) = (pos.0 as i32, pos.1 as i32);
            let around: [(i32, i32); 5] = [
                (px, py),
                (px - 1, py),
                (px + 1, py),
                (px, py - 1),
                (px, py + 1),
            ];
            let in_bounds = |p: (i32, i32)| {
                p.0 >= 0
                    && p.0 < crate::map::level::MAP_W as i32
                    && p.1 >= 0
                    && p.1 < crate::map::level::MAP_H as i32
            };
            let doors: Vec<(u8, u8)> = around
                .iter()
                .filter(|p| in_bounds(**p))
                .map(|p| (p.0 as u8, p.1 as u8))
                .filter(|p| game.current().is_door(*p))
                .collect();
            let closed = doors
                .iter()
                .find(|p| game.current().tile_at(**p) == crate::map::level::Tile::DoorClosed)
                .copied();
            let locked = doors
                .iter()
                .find(|p| game.current().tile_at(**p) == crate::map::level::Tile::DoorLocked)
                .copied();
            match (closed, locked) {
                (Some(p), None) => {
                    game.current_mut()
                        .set_tile(p, crate::map::level::Tile::Floor);
                    game.emit(crate::core::events::GameEvent::Door {
                        opened: true,
                        locked: false,
                    });
                    game.log(
                        crate::core::message::MessageKind::Normal,
                        "The door swings open.",
                    );
                }
                (None, Some(p)) if game.player_has_key() => {
                    game.consume_key();
                    game.current_mut()
                        .set_tile(p, crate::map::level::Tile::Floor);
                    game.emit(crate::core::events::GameEvent::Door {
                        opened: true,
                        locked: true,
                    });
                    game.log(
                        crate::core::message::MessageKind::Normal,
                        "The iron key turns in the lock.",
                    );
                }
                (None, Some(_)) => {
                    game.log(
                        crate::core::message::MessageKind::Normal,
                        "The door is locked. You need a key.",
                    );
                }
                _ => {
                    game.log(
                        crate::core::message::MessageKind::Normal,
                        "Nothing happens.",
                    );
                }
            }
        }
        ScrollKind::Fear => {
            let player_pos = game.player.pos;
            for m in game.monsters.iter_mut() {
                if m.dead {
                    continue;
                }
                let dx = (m.pos.0 as i32 - player_pos.0 as i32).abs();
                let dy = (m.pos.1 as i32 - player_pos.1 as i32).abs();
                if dx + dy <= 10 {
                    m.hp = m.hp.saturating_sub(50);
                }
            }
            let fled: Vec<String> = game
                .monsters
                .iter()
                .filter(|m| !m.dead)
                .take(1)
                .map(|m| m.name.clone())
                .collect();
            if !fled.is_empty() {
                game.log(
                    crate::core::message::MessageKind::Good,
                    format!("{} flees in terror!", fled[0]),
                );
            } else {
                game.log(
                    crate::core::message::MessageKind::Normal,
                    "The monsters nearby cower in fear.",
                );
            }
            // The fear saps your own strength.
            if game.rng.chance(50) {
                if game.statuses.is_blessed() && game.rng.chance(50) {
                    game.log(
                        crate::core::message::MessageKind::Good,
                        "A blessing protects you.",
                    );
                } else {
                    game.statuses.slow = 6;
                    game.log(
                        crate::core::message::MessageKind::Bad,
                        "Your legs feel heavy.",
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::catalog::{make_potion, make_weapon};

    #[test]
    fn healing_potion_restores_hp() {
        let mut g = crate::tests_harness::new_game(42);
        g.player.hp = 5;
        apply_potion(&mut g, PotionKind::Healing(true));
        assert_eq!(g.player.hp, 15);
    }

    #[test]
    fn energy_potion_refills_ep() {
        let mut g = crate::tests_harness::new_game(42);
        g.player.ep = 0;
        apply_potion(&mut g, PotionKind::Energy);
        assert_eq!(g.player.ep, g.player.max_ep);
    }

    #[test]
    fn cure_poison_potion_clears_poison() {
        let mut g = crate::tests_harness::new_game(42);
        g.statuses.poison = 3;
        apply_potion(&mut g, PotionKind::CurePoison);
        assert_eq!(g.statuses.poison, 0);
    }

    #[test]
    fn restore_potion_clears_bad_statuses() {
        let mut g = crate::tests_harness::new_game(42);
        g.statuses.poison = 3;
        g.statuses.disease = 2;
        g.statuses.confusion = 4;
        g.statuses.paralysis = 1;
        apply_potion(&mut g, PotionKind::Restore);
        assert_eq!(g.statuses.poison, 0);
        assert_eq!(g.statuses.disease, 0);
        assert_eq!(g.statuses.confusion, 0);
        assert_eq!(g.statuses.paralysis, 0);
    }

    #[test]
    fn invisibility_potion_grants_invisibility() {
        let mut g = crate::tests_harness::new_game(42);
        apply_potion(&mut g, PotionKind::Invisibility);
        assert!(g.statuses.invisible > 0);
    }

    #[test]
    fn identify_potion_identifies_unidentified_item() {
        let mut g = crate::tests_harness::new_game(42);
        g.player.inventory.push(make_potion(PotionKind::Energy));
        assert!(!g.player.inventory[0].identified);
        apply_potion(&mut g, PotionKind::Identify);
        assert!(g.player.inventory[0].identified);
    }

    #[test]
    fn teleport_scroll_moves_player_to_walkable_tile() {
        let mut g = crate::tests_harness::new_game(42);
        g.monsters.clear();
        let before = g.player.pos;
        apply_scroll(&mut g, ScrollKind::Teleport);
        assert_ne!(g.player.pos, before);
        assert!(g.current().is_walkable(g.player.pos));
        assert!(g
            .drain_events()
            .iter()
            .any(|e| matches!(e, crate::core::events::GameEvent::Teleport)));
    }

    #[test]
    fn teleport_scroll_avoids_occupied_tiles() {
        let mut g = crate::tests_harness::new_game(7);
        let level = g.current().clone();
        g.monsters.clear();
        for p in level.floor_tiles() {
            if p != g.player.pos {
                let mut m = crate::entities::monster::Monster::new(
                    crate::data::monsters::MONSTERS[0].clone(),
                    p,
                );
                m.pos = p;
                g.monsters.push(m);
            }
        }
        assert!(!g.monsters.is_empty(), "setup sanity: monsters exist");
        // Every walkable tile except the player's is occupied: teleport must
        // not move the player and must not place it on a monster.
        apply_scroll(&mut g, ScrollKind::Teleport);
        assert!(
            !g.monsters.iter().any(|m| m.pos == g.player.pos && !m.dead),
            "player must not land on a monster"
        );
        assert!(g.current().is_walkable(g.player.pos));
    }

    #[test]
    fn identify_scroll_identifies_inventory_item() {
        let mut g = crate::tests_harness::new_game(42);
        g.player.inventory.push(make_potion(PotionKind::Energy));
        assert!(!g.player.inventory[0].identified);
        apply_scroll(&mut g, ScrollKind::Identify);
        assert!(g.player.inventory[0].identified);
    }

    #[test]
    fn identify_scroll_with_nothing_unidentified_logs_message() {
        let mut g = crate::tests_harness::new_game(42);
        let item = make_potion(PotionKind::Energy);
        g.player.inventory.push(item);
        apply_scroll(&mut g, ScrollKind::Identify);
        assert!(g.player.inventory[0].identified);
        apply_scroll(&mut g, ScrollKind::Identify);
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text == "You have nothing to identify."));
    }

    #[test]
    fn enchant_weapon_scroll_enchants_wielded_weapon() {
        let mut g = crate::tests_harness::new_game(42);
        g.player.wielded = Some(make_weapon(crate::items::item::WeaponKind::Dagger, 0, false));
        apply_scroll(&mut g, ScrollKind::EnchantWeapon);
        assert_eq!(g.player.wielded.as_ref().unwrap().enchant, 1);
    }

    #[test]
    fn enchant_weapon_scroll_without_weapon_logs_message() {
        let mut g = crate::tests_harness::new_game(42);
        apply_scroll(&mut g, ScrollKind::EnchantWeapon);
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text == "You are not wielding a weapon."));
    }

    #[test]
    fn remove_curse_scroll_uncurses_items() {
        let mut g = crate::tests_harness::new_game(42);
        let mut item = make_potion(PotionKind::Energy);
        item.cursed = true;
        g.player.inventory.push(item);
        apply_scroll(&mut g, ScrollKind::RemoveCurse);
        assert!(!g.player.inventory[0].cursed);
    }

    #[test]
    fn mapping_scroll_explores_current_level() {
        let mut g = crate::tests_harness::new_game(42);
        apply_scroll(&mut g, ScrollKind::Mapping);
        let level = g.current();
        for (i, t) in level.tiles.iter().enumerate() {
            if *t != crate::map::level::Tile::Wall {
                assert!(level.explored[i]);
            }
        }
    }

    #[test]
    fn opening_scroll_opens_adjacent_door() {
        let mut g = crate::tests_harness::new_game(42);
        let pos = g.player.pos;
        g.current_mut()
            .set_tile(pos, crate::map::level::Tile::DoorClosed);
        apply_scroll(&mut g, ScrollKind::Opening);
        assert_eq!(g.current().tile_at(pos), crate::map::level::Tile::Floor);
    }
}

/// Reveal the first unidentified item the player carries, if any.
fn identify_items(game: &mut Game) {
    if let Some(slot) = game.player.inventory.iter().position(|i| !i.identified) {
        game.identify_item(slot);
        let name = game.player.inventory[slot].name();
        game.log(
            crate::core::message::MessageKind::Good,
            format!("You now know it is a {name}."),
        );
        return;
    }
    let mut name = None;
    if let Some(item) = game.player.wielded.as_mut() {
        if !item.identified {
            item.identified = true;
            name = Some(item.name());
        }
    }
    if name.is_none() {
        if let Some(item) = game.player.armor.as_mut() {
            if !item.identified {
                item.identified = true;
                name = Some(item.name());
            }
        }
    }
    if name.is_none() {
        for item in game.player.rings.iter_mut() {
            if !item.identified {
                item.identified = true;
                name = Some(item.name());
                break;
            }
        }
    }
    match name {
        Some(name) => game.log(
            crate::core::message::MessageKind::Good,
            format!("You now know it is a {name}."),
        ),
        None => game.log(
            crate::core::message::MessageKind::Normal,
            "You have nothing to identify.",
        ),
    }
}
