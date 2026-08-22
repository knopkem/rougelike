//! Dungeon zone themes. 25 levels grouped into 5 themed zones of 5 levels.

use crate::core::color::Color;
use serde::{Deserialize, Serialize};

/// A themed zone of the dungeon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    BarrowHalls,
    FungalGrottos,
    DrownedVaults,
    EmberWorks,
    Abyss,
}

impl Theme {
    /// The theme for a given 1-based depth.
    pub fn for_depth(depth: u32) -> Self {
        match depth {
            1..=5 => Theme::BarrowHalls,
            6..=10 => Theme::FungalGrottos,
            11..=15 => Theme::DrownedVaults,
            16..=20 => Theme::EmberWorks,
            _ => Theme::Abyss,
        }
    }

    /// The display name of the zone.
    pub fn name(self) -> &'static str {
        match self {
            Theme::BarrowHalls => "The Barrow Halls",
            Theme::FungalGrottos => "The Fungal Grottos",
            Theme::DrownedVaults => "The Drowned Vaults",
            Theme::EmberWorks => "The Ember Works",
            Theme::Abyss => "The Abyss",
        }
    }

    /// The base floor color for this theme.
    pub fn floor_color(self) -> Color {
        match self {
            Theme::BarrowHalls => Color::DarkYellow,
            Theme::FungalGrottos => Color::DarkGreen,
            Theme::DrownedVaults => Color::DarkBlue,
            Theme::EmberWorks => Color::DarkRed,
            Theme::Abyss => Color::DarkMagenta,
        }
    }

    /// The wall color for this theme.
    pub fn wall_color(self) -> Color {
        match self {
            Theme::BarrowHalls => Color::Gray,
            Theme::FungalGrottos => Color::DarkGray,
            Theme::DrownedVaults => Color::DarkCyan,
            Theme::EmberWorks => Color::Brown,
            Theme::Abyss => Color::DarkGray,
        }
    }

    /// The field-of-view radius for this theme (reduced in the Abyss).
    pub fn fov_radius(self) -> u32 {
        match self {
            Theme::Abyss => 6,
            _ => 8,
        }
    }

    /// Whether this theme has water pools.
    pub fn has_water(self) -> bool {
        self == Theme::DrownedVaults
    }

    /// Whether this theme has lava pools.
    pub fn has_lava(self) -> bool {
        self == Theme::EmberWorks
    }

    /// Whether this theme has spore-gas patches.
    pub fn has_spore_gas(self) -> bool {
        self == Theme::FungalGrottos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_by_depth() {
        assert_eq!(Theme::for_depth(1), Theme::BarrowHalls);
        assert_eq!(Theme::for_depth(5), Theme::BarrowHalls);
        assert_eq!(Theme::for_depth(6), Theme::FungalGrottos);
        assert_eq!(Theme::for_depth(10), Theme::FungalGrottos);
        assert_eq!(Theme::for_depth(11), Theme::DrownedVaults);
        assert_eq!(Theme::for_depth(15), Theme::DrownedVaults);
        assert_eq!(Theme::for_depth(16), Theme::EmberWorks);
        assert_eq!(Theme::for_depth(20), Theme::EmberWorks);
        assert_eq!(Theme::for_depth(21), Theme::Abyss);
        assert_eq!(Theme::for_depth(25), Theme::Abyss);
        assert_eq!(Theme::for_depth(30), Theme::Abyss);
    }

    #[test]
    fn abyss_has_reduced_fov() {
        assert!(Theme::Abyss.fov_radius() < Theme::BarrowHalls.fov_radius());
    }
}
