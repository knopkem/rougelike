//! Classes: 5 classes with distinct stats and kits.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassId {
    Warrior,
    Thief,
    Ranger,
    Mage,
    Cleric,
}

impl ClassId {
    pub const ALL: [ClassId; 5] = [
        ClassId::Warrior,
        ClassId::Thief,
        ClassId::Ranger,
        ClassId::Mage,
        ClassId::Cleric,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ClassId::Warrior => "Warrior",
            ClassId::Thief => "Thief",
            ClassId::Ranger => "Ranger",
            ClassId::Mage => "Mage",
            ClassId::Cleric => "Cleric",
        }
    }

    pub fn desc(&self) -> &'static str {
        match self {
            ClassId::Warrior => "Heavy hitter, high HP.",
            ClassId::Thief => "Stealth, lockpicking, crits.",
            ClassId::Ranger => "Ranged combat, balanced.",
            ClassId::Mage => "High EP, magic power.",
            ClassId::Cleric => "Healing, wisdom, solid HP.",
        }
    }
}
