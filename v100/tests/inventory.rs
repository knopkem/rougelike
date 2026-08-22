//! Integration tests: inventory, equipment, and pickup/drop through the public API.

use deepdelve::core::action::{Action, Direction};
use deepdelve::core::game::Game;
use deepdelve::data::classes::{Class, Race};
use deepdelve::items::equip::Slot;
use deepdelve::items::item::Item;

/// Adding items grows the inventory; removing shrinks it.
#[test]
fn add_and_remove_items() {
    let mut game = Game::new(1, Race::Human, Class::Warrior);
    let before = game.player.inventory.len();
    game.player.add_item(Item::new(600)); // trail rations
    assert_eq!(game.player.inventory.len(), before + 1);
    let removed = game.player.remove_item(before);
    assert!(removed.is_some());
    assert_eq!(game.player.inventory.len(), before);
}

/// Equipping a weapon and armor reflects in the derived combat stats.
#[test]
fn equipment_reflects_in_stats() {
    let mut game = Game::new(1, Race::Human, Class::Warrior);
    let inv = &mut game.player.inventory;
    let weapon_idx = inv.len();
    inv.push(Item::new(1)); // longsword: d8, +1, attack +3
    let armor_idx = inv.len();
    inv.push(Item::new(101)); // chain mail: AC +6

    game.player.equipment.set(Slot::Weapon, Some(weapon_idx));
    game.player.equipment.set(Slot::Armor, Some(armor_idx));

    let (die, bonus) = game.player.equipment.weapon_damage(&game.player.inventory);
    assert_eq!(die, 8);
    assert_eq!(bonus, 1);
    assert_eq!(
        game.player.equipment.attack_bonus(&game.player.inventory),
        3
    );
    assert_eq!(game.player.equipment.ac_bonus(&game.player.inventory), 6);
}

/// Unequipping everything reverts to unarmed defaults.
#[test]
fn unequip_reverts_to_unarmed() {
    let mut game = Game::new(1, Race::Human, Class::Warrior);
    game.player.equipment.set(Slot::Weapon, None);
    game.player.equipment.set(Slot::Armor, None);
    let (die, bonus) = game.player.equipment.weapon_damage(&game.player.inventory);
    assert_eq!((die, bonus), (4, 0));
    assert_eq!(game.player.equipment.ac_bonus(&game.player.inventory), 0);
}

/// Picking up an item on the player's tile adds it to the inventory.
#[test]
fn pickup_adds_item_to_inventory() {
    let mut game = Game::new(1, Race::Human, Class::Warrior);
    let pos = game.player.pos();
    let idx = game.level.idx(pos);
    game.level.items[idx] = Some(600); // trail rations
    let before = game.player.inventory.len();
    game.do_turn(Action::Pickup);
    assert_eq!(game.player.inventory.len(), before + 1);
    // The floor tile is now empty.
    assert!(game.level.items[idx].is_none());
}

/// Dropping an item places it on the floor and removes it from the inventory.
#[test]
fn drop_places_item_on_floor() {
    let mut game = Game::new(1, Race::Human, Class::Warrior);
    let pos = game.player.pos();
    let idx = game.level.idx(pos);
    let before = game.player.inventory.len();
    game.do_turn(Action::Drop { index: 0 });
    assert_eq!(game.player.inventory.len(), before - 1);
    assert!(
        game.level.items[idx].is_some(),
        "dropped item should be on the floor"
    );
}

/// Waiting advances the turn counter.
#[test]
fn wait_advances_turn() {
    let mut game = Game::new(1, Race::Human, Class::Warrior);
    let t0 = game.turn;
    game.do_turn(Action::Wait);
    assert_eq!(game.turn, t0 + 1);
}

/// Moving into a wall does not change the player's position.
#[test]
fn move_into_wall_is_blocked() {
    // Find a floor tile that is adjacent to a wall, stand the player there, and
    // verify that moving into the wall does not change position.
    let mut game = Game::new(1, Race::Human, Class::Warrior);
    let level = &game.level;
    let mut found = None;
    for y in 0..deepdelve::map::level::HEIGHT {
        for x in 0..deepdelve::map::level::WIDTH {
            let p = deepdelve::core::events::Pos::new(x, y);
            if !level.tiles[level.idx(p)].walkable() {
                continue;
            }
            for dir in Direction::ALL {
                let (dx, dy) = dir.delta();
                let np = deepdelve::core::events::Pos::new(
                    (x as i32 + dx).max(0) as u32,
                    (y as i32 + dy).max(0) as u32,
                );
                if level.in_bounds(np) && !level.tiles[level.idx(np)].walkable() {
                    found = Some((p, dir));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        if found.is_some() {
            break;
        }
    }
    let (floor, dir) = found.expect("a floor tile adjacent to a wall must exist");
    game.player.entity.pos = floor;
    game.do_turn(Action::Move(dir));
    assert_eq!(
        game.player.pos(),
        floor,
        "moving into a wall must not move the player"
    );
}
