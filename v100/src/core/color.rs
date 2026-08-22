//! Core color type. The UI layer maps these to `ratatui::style::Color`.

use serde::{Deserialize, Serialize};

/// A palette-agnostic color used throughout the core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Gray,
    DarkGray,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Blue,
    DarkBlue,
    Yellow,
    DarkYellow,
    Cyan,
    DarkCyan,
    Magenta,
    DarkMagenta,
    Orange,
    Brown,
    Black,
}

impl Color {
    /// A dimmed variant for remembered (out-of-FOV) tiles.
    pub fn dim(self) -> Self {
        match self {
            Color::White => Color::Gray,
            Color::Gray => Color::DarkGray,
            Color::Red => Color::DarkRed,
            Color::Green => Color::DarkGreen,
            Color::Blue => Color::DarkBlue,
            Color::Yellow => Color::DarkYellow,
            Color::Cyan => Color::DarkCyan,
            Color::Magenta => Color::DarkMagenta,
            Color::Orange => Color::Brown,
            Color::Brown => Color::DarkGray,
            other => other,
        }
    }
}
