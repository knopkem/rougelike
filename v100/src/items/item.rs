//! Items: instance type, effect enums, and the static catalog.

use crate::core::color::Color;
use serde::{Deserialize, Serialize};

/// The broad category of an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Ring,
    Potion,
    Scroll,
    Wand,
    Food,
    Amulet,
    Misc,
}

impl ItemCategory {
    /// The generic (unidentified) name for this category.
    pub fn generic_name(self) -> &'static str {
        match self {
            ItemCategory::Weapon => "weapon",
            ItemCategory::Armor => "armor",
            ItemCategory::Ring => "ring",
            ItemCategory::Potion => "potion",
            ItemCategory::Scroll => "scroll",
            ItemCategory::Wand => "wand",
            ItemCategory::Food => "food",
            ItemCategory::Amulet => "amulet",
            ItemCategory::Misc => "object",
        }
    }
}

/// A ring's magical effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RingEffect {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
    Regeneration,
    FireResist,
    Stealth,
    Luck,
    Infravision,
    Protection,
}

impl RingEffect {
    pub fn name(self) -> &'static str {
        match self {
            RingEffect::Strength => "strength",
            RingEffect::Dexterity => "dexterity",
            RingEffect::Constitution => "constitution",
            RingEffect::Intelligence => "intelligence",
            RingEffect::Wisdom => "wisdom",
            RingEffect::Charisma => "charisma",
            RingEffect::Regeneration => "regeneration",
            RingEffect::FireResist => "fire resistance",
            RingEffect::Stealth => "stealth",
            RingEffect::Luck => "luck",
            RingEffect::Infravision => "infravision",
            RingEffect::Protection => "protection",
        }
    }
}

/// A potion's effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PotionEffect {
    Healing,
    FullHealing,
    CurePoison,
    RestoreEp,
    Infravision,
    Energy,
    Experience,
    Berserk,
    Teleport,
    Blindness,
    Confusion,
    Antidote,
}

impl PotionEffect {
    pub fn name(self) -> &'static str {
        match self {
            PotionEffect::Healing => "healing",
            PotionEffect::FullHealing => "full healing",
            PotionEffect::CurePoison => "cure poison",
            PotionEffect::RestoreEp => "restore energy",
            PotionEffect::Infravision => "infravision",
            PotionEffect::Energy => "energy",
            PotionEffect::Experience => "experience",
            PotionEffect::Berserk => "berserking",
            PotionEffect::Teleport => "teleportation",
            PotionEffect::Blindness => "blindness",
            PotionEffect::Confusion => "confusion",
            PotionEffect::Antidote => "antidote",
        }
    }
}

/// A scroll's effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollEffect {
    Identify,
    Mapping,
    EnchantWeapon,
    EnchantArmor,
    Teleport,
    Blink,
    Creation,
    WordOfRecall,
    Earthquake,
    MonsterLightning,
}

impl ScrollEffect {
    pub fn name(self) -> &'static str {
        match self {
            ScrollEffect::Identify => "identify",
            ScrollEffect::Mapping => "mapping",
            ScrollEffect::EnchantWeapon => "enchant weapon",
            ScrollEffect::EnchantArmor => "enchant armor",
            ScrollEffect::Teleport => "teleportation",
            ScrollEffect::Blink => "blinking",
            ScrollEffect::Creation => "creation",
            ScrollEffect::WordOfRecall => "word of recall",
            ScrollEffect::Earthquake => "earthquake",
            ScrollEffect::MonsterLightning => "monster lightning",
        }
    }
}

/// A wand's effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WandEffect {
    FireBolt,
    Cold,
    Paralysis,
    Lightning,
    Teleport,
    Healing,
    Sleep,
    Disintegration,
    MonsterLightning,
}

impl WandEffect {
    pub fn name(self) -> &'static str {
        match self {
            WandEffect::FireBolt => "fire bolt",
            WandEffect::Cold => "cold",
            WandEffect::Paralysis => "paralysis",
            WandEffect::Lightning => "lightning",
            WandEffect::Teleport => "teleportation",
            WandEffect::Healing => "healing",
            WandEffect::Sleep => "sleep",
            WandEffect::Disintegration => "disintegration",
            WandEffect::MonsterLightning => "monster lightning",
        }
    }
}

/// A static item template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: u32,
    pub name: &'static str,
    pub glyph: char,
    pub color: Color,
    pub category: ItemCategory,
    /// Weapon damage die (0 for non-weapons).
    pub damage_die: u32,
    /// Weapon damage bonus.
    pub damage_bonus: i32,
    /// Weapon attack (to-hit) bonus.
    pub attack_bonus: i32,
    /// Armor AC bonus.
    pub ac_bonus: i32,
    /// Ring effect (rings only).
    pub ring_effect: Option<RingEffect>,
    /// Potion effect (potions only).
    pub potion_effect: Option<PotionEffect>,
    /// Scroll effect (scrolls only).
    pub scroll_effect: Option<ScrollEffect>,
    /// Wand effect (wands only).
    pub wand_effect: Option<WandEffect>,
    /// Maximum wand charges.
    pub max_charges: u32,
    /// Nutrition value (food only).
    pub nutrition: u32,
    /// Base gold value.
    pub base_value: u32,
    /// Weight (for carrying capacity).
    pub weight: u32,
    /// Loot tier (1..=5).
    pub tier: u8,
    /// Minimum depth for loot generation.
    pub min_depth: u32,
    /// Maximum depth for loot generation.
    pub max_depth: u32,
}

impl ItemDef {
    /// Create a base `ItemDef` with all defaults, for use in the `item!` macro.
    pub const fn base(
        id: u32,
        name: &'static str,
        glyph: char,
        color: Color,
        category: ItemCategory,
    ) -> Self {
        Self {
            id,
            name,
            glyph,
            color,
            category,
            damage_die: 0,
            damage_bonus: 0,
            attack_bonus: 0,
            ac_bonus: 0,
            ring_effect: None,
            potion_effect: None,
            scroll_effect: None,
            wand_effect: None,
            max_charges: 0,
            nutrition: 0,
            base_value: 10,
            weight: 1,
            tier: 1,
            min_depth: 1,
            max_depth: 25,
        }
    }

    pub const fn damage_die(mut self, v: u32) -> Self {
        self.damage_die = v;
        self
    }
    pub const fn damage_bonus(mut self, v: i32) -> Self {
        self.damage_bonus = v;
        self
    }
    pub const fn attack_bonus(mut self, v: i32) -> Self {
        self.attack_bonus = v;
        self
    }
    pub const fn ac_bonus(mut self, v: i32) -> Self {
        self.ac_bonus = v;
        self
    }
    pub const fn ring_effect(mut self, v: Option<RingEffect>) -> Self {
        self.ring_effect = v;
        self
    }
    pub const fn potion_effect(mut self, v: Option<PotionEffect>) -> Self {
        self.potion_effect = v;
        self
    }
    pub const fn scroll_effect(mut self, v: Option<ScrollEffect>) -> Self {
        self.scroll_effect = v;
        self
    }
    pub const fn wand_effect(mut self, v: Option<WandEffect>) -> Self {
        self.wand_effect = v;
        self
    }
    pub const fn max_charges(mut self, v: u32) -> Self {
        self.max_charges = v;
        self
    }
    pub const fn nutrition(mut self, v: u32) -> Self {
        self.nutrition = v;
        self
    }
    pub const fn base_value(mut self, v: u32) -> Self {
        self.base_value = v;
        self
    }
    pub const fn weight(mut self, v: u32) -> Self {
        self.weight = v;
        self
    }
    pub const fn tier(mut self, v: u8) -> Self {
        self.tier = v;
        self
    }
    pub const fn min_depth(mut self, v: u32) -> Self {
        self.min_depth = v;
        self
    }
    pub const fn max_depth(mut self, v: u32) -> Self {
        self.max_depth = v;
        self
    }

    /// Look up a definition by id.
    pub fn by_id(id: u32) -> Option<&'static ItemDef> {
        super::catalog::ALL.iter().find(|d| d.id == id)
    }
}

/// A concrete item instance in the world or inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub def_id: u32,
    /// Stack quantity (for stackable items).
    pub quantity: u32,
    /// Enchantment (weapons/armor).
    pub enchantment: i32,
    /// Remaining wand charges.
    pub charges: u32,
    /// Whether the player has identified this item.
    pub identified: bool,
}

impl Item {
    /// Create a new item instance from a definition.
    pub fn new(def_id: u32) -> Self {
        let def = ItemDef::by_id(def_id).expect("valid item id");
        Self {
            def_id,
            quantity: 1,
            enchantment: 0,
            charges: def.max_charges,
            identified: matches!(
                def.category,
                ItemCategory::Weapon
                    | ItemCategory::Armor
                    | ItemCategory::Food
                    | ItemCategory::Amulet
                    | ItemCategory::Misc
            ),
        }
    }

    /// The static definition for this item.
    pub fn def(&self) -> &'static ItemDef {
        ItemDef::by_id(self.def_id).expect("valid item id")
    }

    /// The display name, accounting for identification and enchantment.
    pub fn name(&self) -> String {
        let def = self.def();
        if !self.identified {
            return format!("a {}", def.category.generic_name());
        }
        match def.category {
            ItemCategory::Weapon | ItemCategory::Armor => {
                if self.enchantment > 0 {
                    format!("a +{} {}", self.enchantment, def.name)
                } else if self.enchantment < 0 {
                    format!("a {} {}", self.enchantment, def.name)
                } else {
                    format!("a {}", def.name)
                }
            }
            ItemCategory::Wand => {
                if self.charges == 0 {
                    format!("a (empty) {}", def.name)
                } else {
                    format!("a {}", def.name)
                }
            }
            _ => format!("a {}", def.name),
        }
    }

    /// Whether this item can be wielded as a weapon.
    pub fn is_weapon(&self) -> bool {
        self.def().category == ItemCategory::Weapon
    }

    /// Whether this item can be worn as armor.
    pub fn is_armor(&self) -> bool {
        self.def().category == ItemCategory::Armor
    }

    /// Whether this item is a ring.
    pub fn is_ring(&self) -> bool {
        self.def().category == ItemCategory::Ring
    }

    /// Whether this item is a potion.
    pub fn is_potion(&self) -> bool {
        self.def().category == ItemCategory::Potion
    }

    /// Whether this item is a scroll.
    pub fn is_scroll(&self) -> bool {
        self.def().category == ItemCategory::Scroll
    }

    /// Whether this item is a wand.
    pub fn is_wand(&self) -> bool {
        self.def().category == ItemCategory::Wand
    }

    /// Whether this item is food.
    pub fn is_food(&self) -> bool {
        self.def().category == ItemCategory::Food
    }

    /// The total weight of this stack.
    pub fn total_weight(&self) -> u32 {
        self.def().weight.saturating_mul(self.quantity)
    }

    /// The gold value of this stack.
    pub fn value(&self) -> u32 {
        self.def().base_value.saturating_mul(self.quantity)
    }
}
