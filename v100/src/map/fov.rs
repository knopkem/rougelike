//! Field of view: per-tile line-of-sight using a DDA (digital differential
//! analyzer) line walk. For each tile within the radius we check whether a
//! straight line from the origin to that tile is blocked by an opaque tile
//! (wall, closed door). This is simple, correct, and fast enough for an
//! 80x25 grid (computed once per turn).

use crate::core::events::Pos;
use crate::map::level::Level;

/// Compute the field of view from `origin` with the given radius.
///
/// Returns a `Vec<bool>` of the same length as `level.tiles`, where `true`
/// means the tile is visible. The origin tile is always visible.
pub fn compute_fov(level: &Level, origin: Pos, radius: u32) -> Vec<bool> {
    let mut visible = vec![false; level.tiles.len()];
    if !level.in_bounds(origin) {
        return visible;
    }
    visible[level.idx(origin)] = true;

    let r = radius as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy > r * r {
                continue;
            }
            let p = Pos::new(
                (origin.x as i32 + dx).max(0) as u32,
                (origin.y as i32 + dy).max(0) as u32,
            );
            if !level.in_bounds(p) || p == origin {
                continue;
            }
            if line_of_sight(level, origin, p) {
                visible[level.idx(p)] = true;
            }
        }
    }
    visible
}

/// True if there is an unobstructed line of sight from `origin` to `target`.
///
/// Walks the line at 2 samples per cell (DDA). The origin and target tiles
/// themselves are never treated as blockers.
pub fn line_of_sight(level: &Level, origin: Pos, target: Pos) -> bool {
    if origin == target {
        return true;
    }
    let dx = target.x as f64 - origin.x as f64;
    let dy = target.y as f64 - origin.y as f64;
    let dist = (dx * dx + dy * dy).sqrt();
    let steps = (dist * 2.0).max(1.0) as usize;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = (origin.x as f64 + dx * t).round() as i32;
        let y = (origin.y as f64 + dy * t).round() as i32;
        let p = Pos::new(x as u32, y as u32);
        if p == target {
            return true;
        }
        if level.in_bounds(p) && !level.tiles[level.idx(p)].transparent() {
            return false;
        }
    }
    true
}

/// A brute-force line-of-sight reference that samples the line at a much finer
/// resolution (8 samples per cell). Used to cross-check `line_of_sight` on
/// unambiguous test cases.
pub fn los_brute_force(level: &Level, origin: Pos, target: Pos) -> bool {
    if origin == target {
        return true;
    }
    let dx = target.x as f64 - origin.x as f64;
    let dy = target.y as f64 - origin.y as f64;
    let dist = (dx * dx + dy * dy).sqrt();
    let steps = (dist * 8.0).max(1.0) as usize;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = (origin.x as f64 + dx * t).round() as i32;
        let y = (origin.y as f64 + dy * t).round() as i32;
        let p = Pos::new(x as u32, y as u32);
        if p == target {
            return true;
        }
        if level.in_bounds(p) && !level.tiles[level.idx(p)].transparent() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::level::Tile;

    fn open_room() -> Level {
        let mut lvl = Level::blank(1);
        for y in 2..20 {
            for x in 2..40 {
                lvl.carve(Pos::new(x, y));
            }
        }
        lvl
    }

    #[test]
    fn origin_always_visible() {
        let lvl = open_room();
        let origin = Pos::new(10, 10);
        let fov = compute_fov(&lvl, origin, 8);
        assert!(fov[lvl.idx(origin)]);
    }

    #[test]
    fn open_room_visible_within_radius() {
        let lvl = open_room();
        let origin = Pos::new(20, 10);
        let fov = compute_fov(&lvl, origin, 8);
        assert!(fov[lvl.idx(Pos::new(25, 10))]); // 5 away
        assert!(!fov[lvl.idx(Pos::new(32, 10))]); // 12 away, beyond radius
    }

    #[test]
    fn wall_blocks_sight() {
        let mut lvl = open_room();
        lvl.set_tile(Pos::new(15, 10), Tile::Wall);
        let origin = Pos::new(10, 10);
        let target = Pos::new(20, 10);
        let fov = compute_fov(&lvl, origin, 15);
        assert!(!fov[lvl.idx(target)]);
        // A tile in front of the wall is still visible.
        assert!(fov[lvl.idx(Pos::new(14, 10))]);
    }

    #[test]
    fn radius_zero_only_origin() {
        let lvl = open_room();
        let origin = Pos::new(10, 10);
        let fov = compute_fov(&lvl, origin, 0);
        assert!(fov[lvl.idx(origin)]);
        assert!(!fov[lvl.idx(Pos::new(11, 10))]);
    }

    #[test]
    fn fov_matches_brute_force_in_open_room() {
        let lvl = open_room();
        let origin = Pos::new(20, 10);
        // Radius 8 covers the full checked region (corners are ~7.07 away).
        let fov = compute_fov(&lvl, origin, 8);
        let mut checked = 0;
        for y in 5..15 {
            for x in 15..25 {
                let p = Pos::new(x, y);
                if !lvl.in_bounds(p) || !lvl.tiles[lvl.idx(p)].walkable() {
                    continue;
                }
                let expected = los_brute_force(&lvl, origin, p);
                let actual = fov[lvl.idx(p)];
                assert_eq!(actual, expected, "mismatch at {p:?}");
                checked += 1;
            }
        }
        assert!(checked > 50, "should check many tiles, got {checked}");
    }

    #[test]
    fn los_agrees_with_reference_on_clear_lines() {
        let lvl = open_room();
        let origin = Pos::new(10, 10);
        for y in 4..16 {
            for x in 4..30 {
                let p = Pos::new(x, y);
                if !lvl.in_bounds(p) {
                    continue;
                }
                assert_eq!(
                    line_of_sight(&lvl, origin, p),
                    los_brute_force(&lvl, origin, p),
                    "LOS mismatch at {p:?}"
                );
            }
        }
    }
}
