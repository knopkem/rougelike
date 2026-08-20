//! Shop: pricing, buy/sell.

use crate::core::game::Game;
use crate::items::item::Item;

/// Price for buying an item from the shop.
pub fn buy_price(item: &Item) -> u32 {
    let base = 50 + item.enchant.unsigned_abs() as u32 * 10 + item.defense as u32 * 5;
    base * 125 / 100
}

/// Price for selling an item to the shop.
pub fn sell_price(item: &Item) -> u32 {
    let base = 50 + item.enchant.unsigned_abs() as u32 * 10 + item.defense as u32 * 5;
    base * 80 / 100
}

/// Attempt a buy: deducts gold, adds item to inventory.
pub fn buy(game: &mut Game, item: &Item) -> bool {
    let price = buy_price(item);
    if game.player.gold < price {
        game.log(
            crate::core::message::MessageKind::Normal,
            "You can't afford that.",
        );
        return false;
    }
    game.player.gold -= price;
    game.player.inventory.push(item.clone());
    game.log(
        crate::core::message::MessageKind::Normal,
        format!("You buy the {} for {price} gold.", item.name()),
    );
    true
}

/// Attempt a sell: adds gold, removes item from inventory.
pub fn sell(game: &mut Game, slot: usize) -> bool {
    if slot >= game.player.inventory.len() {
        return false;
    }
    let item = game.player.inventory.get(slot).cloned();
    if let Some(item) = item {
        let price = sell_price(&item);
        game.player.gold += price;
        game.player.inventory.remove(slot);
        game.log(
            crate::core::message::MessageKind::Normal,
            format!("You sell the {} for {price} gold.", item.name()),
        );
        true
    } else {
        false
    }
}
