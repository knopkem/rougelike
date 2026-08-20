//! Theme-aware color palette for tiles and glyphs.

use ratatui::style::Color;

use crate::map::level::LevelTheme;

pub fn wall(theme: LevelTheme) -> Color {
    match theme {
        LevelTheme::BarrowHalls => Color::DarkGray,
        LevelTheme::FungalGrottos => Color::Green,
        LevelTheme::DrownedVaults => Color::Blue,
        LevelTheme::EmberWorks => Color::DarkGray,
        LevelTheme::Abyss => Color::Magenta,
    }
}

pub fn floor(theme: LevelTheme) -> Color {
    match theme {
        LevelTheme::BarrowHalls => Color::Gray,
        LevelTheme::FungalGrottos => Color::Green,
        LevelTheme::DrownedVaults => Color::Blue,
        LevelTheme::EmberWorks => Color::Yellow,
        LevelTheme::Abyss => Color::Magenta,
    }
}

pub fn stairs() -> Color {
    Color::Yellow
}
