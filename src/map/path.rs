//! A* pathfinding on the level grid.

use crate::map::level::Level;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Copy)]
struct Node {
    x: u8,
    y: u8,
    g: f64,
    f: f64,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f && self.g == other.g
    }
}
impl Eq for Node {}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Node {
    // BinaryHeap pops the maximum, so invert the comparison: smallest f
    // first, ties broken toward the larger g (closer to the goal).
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .partial_cmp(&self.f)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.g.partial_cmp(&other.g).unwrap_or(Ordering::Equal))
    }
}

fn heuristic(a: (u8, u8), b: (u8, u8)) -> f64 {
    let dx = (a.0 as i32 - b.0 as i32).abs() as f64;
    let dy = (a.1 as i32 - b.1 as i32).abs() as f64;
    dx + dy
}

/// A* from `start` to `goal` on walkable tiles. Returns path (excluding
/// start, including goal) or `None` if unreachable.
pub fn astar(level: &Level, start: (u8, u8), goal: (u8, u8)) -> Option<Vec<(u8, u8)>> {
    if !level.is_walkable(goal) {
        // Allow goal on stairs which are walkable.
        if !matches!(
            level.tile_at(goal),
            crate::map::level::Tile::Floor
                | crate::map::level::Tile::StairsUp
                | crate::map::level::Tile::StairsDown
                | crate::map::level::Tile::Water
                | crate::map::level::Tile::Lava
                | crate::map::level::Tile::SporeGas
                | crate::map::level::Tile::Grate
                | crate::map::level::Tile::Trap(_)
        ) {
            return None;
        }
    }
    if start == goal {
        return Some(vec![]);
    }
    let w = crate::map::level::MAP_W as usize;
    let h = crate::map::level::MAP_H as usize;
    let n = w * h;
    let mut g = vec![f64::INFINITY; n];
    let mut came: Vec<Option<(u8, u8)>> = vec![None; n];
    let mut closed = vec![false; n];

    let start_i = start.1 as usize * w + start.0 as usize;
    let goal_i = goal.1 as usize * w + goal.0 as usize;
    g[start_i] = 0.0;

    let mut open = BinaryHeap::new();
    open.push(Node {
        x: start.0,
        y: start.1,
        g: 0.0,
        f: heuristic(start, goal),
    });

    while let Some(node) = open.pop() {
        let ci = node.y as usize * w + node.x as usize;
        if closed[ci] {
            continue;
        }
        closed[ci] = true;
        if ci == goal_i {
            // Reconstruct: include goal, then walk parents until (but excluding) start.
            let mut path = Vec::new();
            let mut cur = goal_i;
            if cur != start_i {
                path.push(((cur % w) as u8, (cur / w) as u8));
            }
            while cur != start_i {
                if let Some(parent) = came[cur] {
                    cur = parent.1 as usize * w + parent.0 as usize;
                } else {
                    break;
                }
                if cur != start_i {
                    path.push(((cur % w) as u8, (cur / w) as u8));
                }
            }
            path.reverse();
            return Some(path);
        }
        let (cx, cy) = (node.x, node.y);
        for (dx, dy) in [(1i8, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = (cx as i8 + dx) as u8;
            let ny = (cy as i8 + dy) as u8;
            if !level.is_in_bounds(nx, ny) {
                continue;
            }
            if !level.is_walkable((nx, ny)) {
                continue;
            }
            let ni = ny as usize * w + nx as usize;
            if closed[ni] {
                continue;
            }
            let tentative = g[ci] + 1.0;
            if tentative < g[ni] {
                g[ni] = tentative;
                came[ni] = Some((cx, cy));
                open.push(Node {
                    x: nx,
                    y: ny,
                    g: tentative,
                    f: tentative + heuristic((nx, ny), goal),
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::level::Tile;

    #[test]
    fn straight_path() {
        let mut lvl = Level::new(1);
        for x in 5..=15 {
            lvl.set_tile((x, 10), Tile::Floor);
        }
        let path = astar(&lvl, (5, 10), (10, 10)).unwrap();
        assert_eq!(path.len(), 5);
        assert_eq!(path.last().unwrap(), &(10, 10));
    }

    #[test]
    fn unreachable() {
        let mut lvl = Level::new(1);
        lvl.set_tile((5, 10), Tile::Floor);
        let path = astar(&lvl, (5, 10), (10, 10));
        assert!(path.is_none());
    }

    #[test]
    fn detour_finds_optimal_path() {
        // Direct row y=10 is blocked at x=9..=11. Short route goes over the
        // y=9 gap (12 steps); a corridor below offers a 30-step detour.
        // With the inverted (max-f) heap the goal was popped via the detour.
        let mut lvl = Level::new(1);
        for x in 5..=15 {
            if (9..=11).contains(&x) {
                continue;
            }
            lvl.set_tile((x, 10), Tile::Floor);
        }
        for x in 8..=12 {
            lvl.set_tile((x, 9), Tile::Floor);
        }
        for y in 11..=20 {
            lvl.set_tile((5, y), Tile::Floor);
            lvl.set_tile((15, y), Tile::Floor);
        }
        for x in 6..=14 {
            lvl.set_tile((x, 20), Tile::Floor);
        }
        let path = astar(&lvl, (5, 10), (15, 10)).unwrap();
        assert_eq!(path.len(), 12);
        assert_eq!(path.first().unwrap(), &(6, 10));
        assert_eq!(path.last().unwrap(), &(15, 10));
    }
}
