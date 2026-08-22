//! Maps the core, palette-agnostic `Color` to `ratatui::style::Color`.

use crate::core::color::Color as CoreColor;
use ratatui::style::Color;

/// Convert a core color to a ratatui color.
pub fn to_ratatui(c: CoreColor) -> Color {
    match c {
        CoreColor::White => Color::White,
        CoreColor::Gray => Color::Gray,
        CoreColor::DarkGray => Color::DarkGray,
        CoreColor::Red => Color::Red,
        CoreColor::DarkRed => Color::Rgb(128, 0, 0),
        CoreColor::Green => Color::Green,
        CoreColor::DarkGreen => Color::Rgb(0, 128, 0),
        CoreColor::Blue => Color::Blue,
        CoreColor::DarkBlue => Color::Rgb(0, 0, 128),
        CoreColor::Yellow => Color::Yellow,
        CoreColor::DarkYellow => Color::Rgb(128, 128, 0),
        CoreColor::Cyan => Color::Cyan,
        CoreColor::DarkCyan => Color::Rgb(0, 128, 128),
        CoreColor::Magenta => Color::Magenta,
        CoreColor::DarkMagenta => Color::Rgb(128, 0, 128),
        CoreColor::Orange => Color::Rgb(255, 165, 0),
        CoreColor::Brown => Color::Rgb(139, 69, 19),
        CoreColor::Black => Color::Black,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_all_core_colors() {
        // Just ensure the function is total (no missing arms).
        let _ = to_ratatui(CoreColor::White);
        let _ = to_ratatui(CoreColor::Orange);
        let _ = to_ratatui(CoreColor::Black);
    }
}
