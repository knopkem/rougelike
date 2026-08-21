//! Races: 4 races with modifiers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaceId {
    Human,
    Elf,
    Dwarf,
    Halfling,
}

impl RaceId {
    pub const ALL: [RaceId; 4] = [RaceId::Human, RaceId::Elf, RaceId::Dwarf, RaceId::Halfling];

    pub fn name(&self) -> &'static str {
        match self {
            RaceId::Human => "Human",
            RaceId::Elf => "Elf",
            RaceId::Dwarf => "Dwarf",
            RaceId::Halfling => "Halfling",
        }
    }

    pub fn desc(&self) -> &'static str {
        match self {
            RaceId::Human => "Versatile: +1 all attributes.",
            RaceId::Elf => "+DEX +INT, +darkvision.",
            RaceId::Dwarf => "+CON, +AC, fire resistance.",
            RaceId::Halfling => "+DEX +CON, +stealth, +crit luck.",
        }
    }
}
