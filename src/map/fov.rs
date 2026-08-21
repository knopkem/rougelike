//! 3x3 shadowcasting field of view.

use crate::map::level::Level;

/// Compute field of view from `origin` with radius `radius`, writing to
/// `level.seen` (tiles visible this turn) and `level.explored`.
pub fn compute_fov(level: &mut Level, origin: (u8, u8), radius: u8) {
    let w = crate::map::level::MAP_W as i32;
    let h = crate::map::level::MAP_H as i32;
    let (ox, oy) = (origin.0 as i32, origin.1 as i32);

    // Clear seen; keep explored.
    level.seen.fill(false);
    level.seen[Level::pos_idx(origin)] = true;
    level.explored[Level::pos_idx(origin)] = true;

    // 8 shadowcaster directions (rings).
    for (dx, dy) in SHADOW_DIRECTIONS {
        let mut x = ox + dx;
        let mut y = oy + dy;
        let mut l = 1.0f32;
        while l <= (radius as f32) * (2.0f32.sqrt()) {
            if !level.is_in_bounds(x as u8, y as u8) {
                break;
            }
            // LOS from origin to this tile.
            if line_of_sight(level, ox, oy, x, y) {
                set_seen(level, x, y);
                if !level.is_transparent((x as u8, y as u8)) {
                    break;
                }
            } else {
                // Wall blocks the ring from here on out.
                break;
            }
            // Advance
            x += dx;
            y += dy;
            l = ((dx * dx + dy * dy) as f32).sqrt() + l;
        }
    }
    let _ = (w, h);
}

const SHADOW_DIRECTIONS: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

fn set_seen(level: &mut Level, x: i32, y: i32) {
    if x < 0
        || y < 0
        || x >= crate::map::level::MAP_W as i32
        || y >= crate::map::level::MAP_H as i32
    {
        return;
    }
    let i = Level::idx(x as u8, y as u8);
    level.seen[i] = true;
    level.explored[i] = true;
}

/// Bresenham line of sight between two points (inclusive of endpoints).
/// An invisible player can still be seen on direct line of sight, so the
/// monster AI consults this as the invisibility seam.
pub fn line_of_sight(level: &Level, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        if x == x1 && y == y1 {
            return true;
        }
        if !(x == x0 && y == y0) && !level.is_transparent((x as u8, y as u8)) {
            return false;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::level::Tile;

    #[test]
    fn sees_own_tile() {
        let mut lvl = Level::new(1);
        lvl.set_tile((10, 5), Tile::Floor);
        compute_fov(&mut lvl, (10, 5), 8);
        assert!(lvl.seen[Level::pos_idx((10, 5))]);
        assert!(lvl.explored[Level::pos_idx((10, 5))]);
    }

    #[test]
    fn wall_blocks() {
        let mut lvl = Level::new(1);
        for x in 8..=20 {
            lvl.set_tile((x, 5), Tile::Floor);
        }
        lvl.set_tile((12, 5), Tile::Wall);
        compute_fov(&mut lvl, (10, 5), 8);
        // Tile behind wall not seen
        assert!(!lvl.seen[Level::pos_idx((14, 5))]);
        // Tile in front seen
        assert!(lvl.seen[Level::pos_idx((11, 5))]);
    }
}
