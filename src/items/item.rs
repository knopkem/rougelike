//! Item model: kinds, enchant, identification, stacks.

use serde::{Deserialize, Serialize};

use crate::entities::monster::Rarity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponKind {
    Dagger,
    Shortsword,
    Longsword,
    Greatsword,
    BattleAxe,
    WarHammer,
    Mace,
    Flail,
    Spear,
    Trident,
    MorningStar,
    WarFlail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmorKind {
    Chainmail,
    Plate,
    LeatherHelm,
    IronHelm,
    LeatherGloves,
    PlateGloves,
    LeatherBoots,
    IronBoots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShieldKind {
    Small,
    Large,
    Tower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WandKind {
    FireBolt,
    Lightning,
    Healing,
    CurePoison,
    Sleep,
    Confusion,
    Paralyze,
    Blink,
    TeleportControl,
    MonsterRemoval,
    MagicMapping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PotionKind {
    Healing(bool), // small
    CurePoison,
    Restore,
    Identify,
    Invisibility,
    Energy,
    Antidote,
    Mutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollKind {
    Identify,
    Teleport,
    EnchantWeapon,
    EnchantArmor,
    RemoveCurse,
    Mapping,
    GodsMessage,
    Opening,
    Fear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RingKind {
    Protection,
    Energy,
    Stealth,
    Infravision,
    Sustenance,
    PoisonResistance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoodKind {
    TrailRations,
    Apple,
    Mushroom,
    Steak,
    EnergyDrink,
    Candy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    Weapon(WeaponKind),
    Armor(ArmorKind),
    Shield(ShieldKind),
    Wand(WandKind),
    Potion(PotionKind),
    Scroll(ScrollKind),
    Ring(RingKind),
    Food(FoodKind),
    Gold,
    Amulet,
    Key,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub name_str: String,
    pub enchant: i8,
    pub defense: u8,
    pub cursed: bool,
    pub identified: bool,
    pub rarity: Rarity,
}

impl Item {
    pub fn name(&self) -> String {
        if self.identified {
            self.name_str.clone()
        } else {
            format!(
                "unidentified {}",
                self.name_str.split(' ').next().unwrap_or("item")
            )
        }
    }

    /// Floor glyph.
    pub fn glyph(&self) -> char {
        match self.kind {
            ItemKind::Weapon(_) => '/',
            ItemKind::Armor(_) => '[',
            ItemKind::Shield(_) => '[',
            ItemKind::Wand(_) => '=',
            ItemKind::Potion(_) => '!',
            ItemKind::Scroll(_) => '?',
            ItemKind::Ring(_) => ')',
            ItemKind::Food(_) => '%',
            ItemKind::Gold => '$',
            ItemKind::Amulet => '"',
            ItemKind::Key => '~',
        }
    }

    pub fn food_value(&self) -> u8 {
        match self.kind {
            ItemKind::Food(_) => 200,
            _ => 0,
        }
    }

    pub fn ep_cost(&self) -> u8 {
        match self.kind {
            ItemKind::Wand(_) => 2,
            _ => 0,
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name_str == other.name_str
    }
}

impl Eq for Item {}
