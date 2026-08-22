//! The shop: a shopkeeper's stock and buy/sell logic.

use crate::core::rng::Rng;
use crate::items::item::Item;
use serde::{Deserialize, Serialize};

/// A single item in the shop's stock, with a price.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopItem {
    pub item: Item,
    /// The price to buy this item.
    pub buy_price: u32,
    /// The price the shop will pay to buy this item back.
    pub sell_price: u32,
}

/// A shop with a stock of items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shop {
    /// The shop's stock.
    pub stock: Vec<ShopItem>,
    /// The shop's name.
    pub name: String,
}

impl Shop {
    /// Generate a shop with a random stock appropriate for the depth.
    pub fn generate(rng: &mut Rng, depth: u32, name: &str) -> Self {
        let mut stock = Vec::new();
        // 6-10 items in the stock.
        let count = rng.range(6, 11) as usize;
        for _ in 0..count {
            let item = crate::items::loot::generate_item(rng, depth);
            let base = item.value();
            // Buy price: base * (1.5 to 2.5). Sell price: base * (0.3 to 0.6).
            let buy_mult = rng.range(150, 250);
            let sell_mult = rng.range(30, 60);
            let buy_price = base * buy_mult / 100;
            let sell_price = base * sell_mult / 100;
            stock.push(ShopItem {
                item,
                buy_price: buy_price.max(1),
                sell_price,
            });
        }
        Self {
            stock,
            name: name.to_string(),
        }
    }

    /// The number of items in stock.
    pub fn len(&self) -> usize {
        self.stock.len()
    }

    /// Whether the stock is empty.
    pub fn is_empty(&self) -> bool {
        self.stock.is_empty()
    }

    /// Buy an item from the shop. Returns the item if successful.
    ///
    /// `gold` is the player's gold; this function does not modify it (the caller
    /// handles the transaction). Returns `None` if the player can't afford it.
    pub fn can_buy(&self, index: usize, gold: u32) -> bool {
        self.stock
            .get(index)
            .map(|s| s.buy_price <= gold)
            .unwrap_or(false)
    }

    /// Remove an item from the stock (after a successful purchase).
    pub fn remove_purchased(&mut self, index: usize) -> Option<ShopItem> {
        if index < self.stock.len() {
            Some(self.stock.remove(index))
        } else {
            None
        }
    }

    /// Add an item to the stock (after a successful sale).
    pub fn add_sold(&mut self, item: Item) {
        let base = item.value();
        let buy_price = base * 200 / 100;
        let sell_price = base * 40 / 100;
        self.stock.push(ShopItem {
            item,
            buy_price: buy_price.max(1),
            sell_price,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_shop_has_stock() {
        let mut rng = Rng::new(42);
        let shop = Shop::generate(&mut rng, 5, "Test Shop");
        assert!(!shop.is_empty());
        assert!(shop.len() >= 6);
    }

    #[test]
    fn buy_price_exceeds_value() {
        let mut rng = Rng::new(42);
        let shop = Shop::generate(&mut rng, 5, "Test Shop");
        for s in &shop.stock {
            assert!(s.buy_price >= s.item.value());
        }
    }

    #[test]
    fn can_buy_checks_gold() {
        let mut rng = Rng::new(42);
        let shop = Shop::generate(&mut rng, 5, "Test Shop");
        let price = shop.stock[0].buy_price;
        assert!(shop.can_buy(0, price));
        assert!(!shop.can_buy(0, price - 1));
    }

    #[test]
    fn remove_purchased() {
        let mut rng = Rng::new(42);
        let mut shop = Shop::generate(&mut rng, 5, "Test Shop");
        let len = shop.len();
        let removed = shop.remove_purchased(0);
        assert!(removed.is_some());
        assert_eq!(shop.len(), len - 1);
    }

    #[test]
    fn add_sold() {
        let mut rng = Rng::new(42);
        let mut shop = Shop::generate(&mut rng, 5, "Test Shop");
        let len = shop.len();
        shop.add_sold(Item::new(0));
        assert_eq!(shop.len(), len + 1);
    }
}
