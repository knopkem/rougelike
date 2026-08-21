//! Item catalog: all item definitions and factory.

use crate::core::rng::Rng;
use crate::entities::monster::Rarity;
use crate::items::item::*;

pub fn make_weapon(kind: WeaponKind, enchant: i8, cursed: bool, rng: &mut Rng) -> Item {
    let (name, defense) = match kind {
        WeaponKind::Dagger => ("dagger", 1),
        WeaponKind::Shortsword => ("shortsword", 2),
        WeaponKind::Longsword => ("longsword", 3),
        WeaponKind::Greatsword => ("greatsword", 4),
        WeaponKind::BattleAxe => ("battle axe", 3),
        WeaponKind::WarHammer => ("war hammer", 4),
        WeaponKind::Mace => ("mace", 3),
        WeaponKind::Flail => ("flail", 3),
        WeaponKind::Spear => ("spear", 2),
        WeaponKind::Trident => ("trident", 4),
        WeaponKind::MorningStar => ("morning star", 4),
        WeaponKind::WarFlail => ("war flail", 5),
    };
    let _ = rng;
    Item {
        kind: ItemKind::Weapon(kind),
        name_str: name.to_string(),
        enchant,
        defense: defense as u8,
        cursed,
        identified: false,
        rarity: Rarity::Common,
    }
}

pub fn make_armor(kind: ArmorKind, enchant: i8, cursed: bool) -> Item {
    let (name, defense) = match kind {
        ArmorKind::Chainmail => ("chain mail", 3),
        ArmorKind::Plate => ("plate armor", 5),
        ArmorKind::LeatherHelm => ("leather helm", 1),
        ArmorKind::IronHelm => ("iron helm", 2),
        ArmorKind::LeatherGloves => ("leather gloves", 1),
        ArmorKind::PlateGloves => ("plate gloves", 2),
        ArmorKind::LeatherBoots => ("leather boots", 1),
        ArmorKind::IronBoots => ("iron boots", 2),
    };
    Item {
        kind: ItemKind::Armor(kind),
        name_str: name.to_string(),
        enchant,
        defense,
        cursed,
        identified: false,
        rarity: Rarity::Common,
    }
}

pub fn make_shield(kind: ShieldKind, enchant: i8, cursed: bool) -> Item {
    let (name, defense) = match kind {
        ShieldKind::Small => ("small shield", 1),
        ShieldKind::Large => ("large shield", 2),
        ShieldKind::Tower => ("tower shield", 3),
    };
    Item {
        kind: ItemKind::Shield(kind),
        name_str: name.to_string(),
        enchant,
        defense,
        cursed,
        identified: false,
        rarity: Rarity::Common,
    }
}

pub fn make_wand(kind: WandKind, enchant: i8) -> Item {
    let name = match kind {
        WandKind::FireBolt => "wand of fire bolt",
        WandKind::Lightning => "wand of lightning",
        WandKind::Healing => "wand of healing",
        WandKind::CurePoison => "wand of cure poison",
        WandKind::Sleep => "wand of sleep",
        WandKind::Confusion => "wand of confusion",
        WandKind::Paralyze => "wand of paralyze",
        WandKind::Blink => "wand of blink",
        WandKind::TeleportControl => "wand of teleport control",
        WandKind::MonsterRemoval => "wand of monster removal",
        WandKind::MagicMapping => "wand of magic mapping",
    };
    Item {
        kind: ItemKind::Wand(kind),
        name_str: name.to_string(),
        enchant,
        defense: 0,
        cursed: false,
        identified: false,
        rarity: Rarity::Uncommon,
    }
}

pub fn make_potion(kind: PotionKind) -> Item {
    let name = match kind {
        PotionKind::Healing(small) => {
            if small {
                "potion of healing"
            } else {
                "potion of super healing"
            }
        }
        PotionKind::CurePoison => "potion of cure poison",
        PotionKind::Restore => "potion of restore",
        PotionKind::Identify => "potion of identify",
        PotionKind::Invisibility => "potion of invisibility",
        PotionKind::Energy => "potion of energy",
        PotionKind::Antidote => "potion of antidote",
        PotionKind::Mutation => "potion of mutation",
    };
    Item {
        kind: ItemKind::Potion(kind),
        name_str: name.to_string(),
        enchant: 0,
        defense: 0,
        cursed: false,
        identified: false,
        rarity: Rarity::Common,
    }
}

pub fn make_scroll(kind: ScrollKind) -> Item {
    let name = match kind {
        ScrollKind::Identify => "scroll of identify",
        ScrollKind::Teleport => "scroll of teleport",
        ScrollKind::EnchantWeapon => "scroll of enchant weapon",
        ScrollKind::EnchantArmor => "scroll of enchant armor",
        ScrollKind::RemoveCurse => "scroll of remove curse",
        ScrollKind::Mapping => "scroll of mapping",
        ScrollKind::GodsMessage => "scroll of god's message",
        ScrollKind::Opening => "scroll of opening",
        ScrollKind::Fear => "scroll of fear",
    };
    Item {
        kind: ItemKind::Scroll(kind),
        name_str: name.to_string(),
        enchant: 0,
        defense: 0,
        cursed: false,
        identified: false,
        rarity: Rarity::Common,
    }
}

pub fn make_ring(kind: RingKind) -> Item {
    let name = match kind {
        RingKind::Protection => "ring of protection",
        RingKind::Energy => "ring of energy",
        RingKind::Stealth => "ring of stealth",
        RingKind::Infravision => "ring of infravision",
        RingKind::Sustenance => "ring of sustenance",
        RingKind::PoisonResistance => "ring of poison resistance",
    };
    Item {
        kind: ItemKind::Ring(kind),
        name_str: name.to_string(),
        enchant: 0,
        defense: 0,
        cursed: false,
        identified: false,
        rarity: Rarity::Rare,
    }
}

pub fn make_food(kind: FoodKind) -> Item {
    let name = match kind {
        FoodKind::TrailRations => "trail rations",
        FoodKind::Apple => "apple",
        FoodKind::Mushroom => "mushroom",
        FoodKind::Steak => "steak",
        FoodKind::EnergyDrink => "energy drink",
        FoodKind::Candy => "potion-flavored candy",
    };
    Item {
        kind: ItemKind::Food(kind),
        name_str: name.to_string(),
        enchant: 0,
        defense: 0,
        cursed: false,
        identified: true,
        rarity: Rarity::Common,
    }
}

pub fn make_gold() -> Item {
    Item {
        kind: ItemKind::Gold,
        name_str: "gold".to_string(),
        enchant: 0,
        defense: 0,
        cursed: false,
        identified: true,
        rarity: Rarity::Common,
    }
}

pub fn make_amulet() -> Item {
    Item {
        kind: ItemKind::Amulet,
        name_str: "Amulet of the Abyss".to_string(),
        enchant: 0,
        defense: 0,
        cursed: false,
        identified: true,
        rarity: Rarity::Legendary,
    }
}

pub fn make_key() -> Item {
    Item {
        kind: ItemKind::Key,
        name_str: "iron key".to_string(),
        enchant: 0,
        defense: 0,
        cursed: false,
        identified: true,
        rarity: Rarity::Rare,
    }
}

#[allow(dead_code)]
pub fn random_weapon(rng: &mut Rng, depth: u8) -> Item {
    let kinds = [
        WeaponKind::Dagger,
        WeaponKind::Shortsword,
        WeaponKind::Longsword,
        WeaponKind::Greatsword,
        WeaponKind::BattleAxe,
        WeaponKind::WarHammer,
        WeaponKind::Mace,
        WeaponKind::Flail,
        WeaponKind::Spear,
        WeaponKind::Trident,
        WeaponKind::MorningStar,
        WeaponKind::WarFlail,
    ];
    let k = rng.pick(&kinds).unwrap_or(WeaponKind::Dagger);
    let enchant = (rng.int_inclusive(0..=3) as i8) + (depth / 8) as i8;
    let cursed = rng.chance(10);
    make_weapon(k, enchant, cursed, rng)
}

#[allow(dead_code)]
pub fn random_armor(rng: &mut Rng, depth: u8) -> Item {
    let kinds = [
        ArmorKind::Chainmail,
        ArmorKind::Plate,
        ArmorKind::LeatherHelm,
        ArmorKind::IronHelm,
        ArmorKind::LeatherGloves,
        ArmorKind::PlateGloves,
        ArmorKind::LeatherBoots,
        ArmorKind::IronBoots,
    ];
    let k = rng.pick(&kinds).unwrap_or(ArmorKind::Chainmail);
    let enchant = (rng.int_inclusive(0..=2) as i8) + (depth / 10) as i8;
    let cursed = rng.chance(8);
    make_armor(k, enchant, cursed)
}

#[allow(dead_code)]
pub fn random_shield(rng: &mut Rng) -> Item {
    let kinds = [ShieldKind::Small, ShieldKind::Large, ShieldKind::Tower];
    let k = rng.pick(&kinds).unwrap_or(ShieldKind::Small);
    make_shield(k, 0, rng.chance(8))
}

#[allow(dead_code)]
pub fn random_wand(rng: &mut Rng) -> Item {
    let kinds = [
        WandKind::FireBolt,
        WandKind::Lightning,
        WandKind::Healing,
        WandKind::CurePoison,
        WandKind::Sleep,
        WandKind::Confusion,
        WandKind::Paralyze,
        WandKind::Blink,
        WandKind::TeleportControl,
        WandKind::MonsterRemoval,
        WandKind::MagicMapping,
    ];
    let k = rng.pick(&kinds).unwrap_or(WandKind::FireBolt);
    make_wand(k, 0)
}

#[allow(dead_code)]
pub fn random_potion(rng: &mut Rng) -> Item {
    let kinds = [
        PotionKind::Healing(true),
        PotionKind::Healing(false),
        PotionKind::CurePoison,
        PotionKind::Restore,
        PotionKind::Identify,
        PotionKind::Invisibility,
        PotionKind::Energy,
        PotionKind::Antidote,
        PotionKind::Mutation,
    ];
    let k = rng.pick(&kinds).unwrap_or(PotionKind::Healing(true));
    make_potion(k)
}

#[allow(dead_code)]
pub fn random_scroll(rng: &mut Rng) -> Item {
    let kinds = [
        ScrollKind::Identify,
        ScrollKind::Teleport,
        ScrollKind::EnchantWeapon,
        ScrollKind::EnchantArmor,
        ScrollKind::RemoveCurse,
        ScrollKind::Mapping,
        ScrollKind::GodsMessage,
        ScrollKind::Opening,
        ScrollKind::Fear,
    ];
    let k = rng.pick(&kinds).unwrap_or(ScrollKind::Identify);
    make_scroll(k)
}

#[allow(dead_code)]
pub fn random_ring(rng: &mut Rng) -> Item {
    let kinds = [
        RingKind::Protection,
        RingKind::Energy,
        RingKind::Stealth,
        RingKind::Infravision,
        RingKind::Sustenance,
        RingKind::PoisonResistance,
    ];
    let k = rng.pick(&kinds).unwrap_or(RingKind::Protection);
    make_ring(k)
}

#[allow(dead_code)]
pub fn random_food(rng: &mut Rng) -> Item {
    let kinds = [
        FoodKind::TrailRations,
        FoodKind::Apple,
        FoodKind::Mushroom,
        FoodKind::Steak,
        FoodKind::EnergyDrink,
        FoodKind::Candy,
    ];
    let k = rng.pick(&kinds).unwrap_or(FoodKind::TrailRations);
    make_food(k)
}
