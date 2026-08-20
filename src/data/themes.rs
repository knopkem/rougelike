//! Dungeon zone themes.

use crate::map::level::LevelTheme;

pub fn theme_for_depth(depth: u8) -> LevelTheme {
    LevelTheme::for_depth(depth)
}

pub fn zone_name(depth: u8) -> &'static str {
    LevelTheme::for_depth(depth).name()
}
