//! A* pathfinding for monsters. Finds the shortest walkable path between two
//! tiles on a level.

use crate::core::events::Pos;
use crate::map::level::Level;
use std::collections::{BinaryHeap, VecDeque};

/// The result of a pathfinding query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResult {
    /// A path from `start` to `goal` (inclusive of both endpoints).
    Found(Vec<Pos>),
    /// No path exists.
    Unreachable,
}

/// Run A* from `start` to `goal` over walkable tiles.
///
/// Returns the full path including both endpoints, or `Unreachable` if no path
/// exists. The path is optimal (shortest) in terms of number of steps.
pub fn astar(level: &Level, start: Pos, goal: Pos) -> PathResult {
    if !level.in_bounds(start) || !level.in_bounds(goal) {
        return PathResult::Unreachable;
    }
    if start == goal {
        return PathResult::Found(vec![start]);
    }
    if !level.tiles[level.idx(start)].walkable() || !level.tiles[level.idx(goal)].walkable() {
        return PathResult::Unreachable;
    }

    let start_idx = level.idx(start);
    let goal_idx = level.idx(goal);

    // Binary heap ordered by f-score (ascending). We store (f, tiebreak, idx)
    // where tiebreak is a counter to make the ordering total.
    #[derive(PartialEq)]
    struct Node {
        f: u32,
        tie: u32,
        idx: usize,
    }
    impl Eq for Node {}
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            // We want a min-heap on f, so reverse the natural ordering.
            other.f.cmp(&self.f).then_with(|| other.tie.cmp(&self.tie))
        }
    }

    let mut open = BinaryHeap::new();
    let mut g_score = vec![u32::MAX; level.tiles.len()];
    let mut came_from = vec![usize::MAX; level.tiles.len()];
    let mut counter = 0u32;

    g_score[start_idx] = 0;
    open.push(Node {
        f: heuristic(start, goal),
        tie: counter,
        idx: start_idx,
    });
    counter += 1;

    let mut closed = vec![false; level.tiles.len()];

    while let Some(node) = open.pop() {
        let current = node.idx;
        if closed[current] {
            continue;
        }
        closed[current] = true;

        if current == goal_idx {
            let mut path = vec![goal];
            let mut cur = goal_idx;
            while cur != start_idx {
                cur = came_from[cur];
                path.push(Level::pos(cur));
            }
            path.reverse();
            return PathResult::Found(path);
        }

        let cpos = Level::pos(current);
        for d in crate::core::action::Direction::ALL {
            let (dx, dy) = d.delta();
            let np = Pos::new(
                (cpos.x as i32 + dx).max(0) as u32,
                (cpos.y as i32 + dy).max(0) as u32,
            );
            if !level.in_bounds(np) {
                continue;
            }
            let nidx = level.idx(np);
            if closed[nidx] || !level.tiles[nidx].walkable() {
                continue;
            }
            let tentative_g = g_score[current] + 1;
            if tentative_g < g_score[nidx] {
                g_score[nidx] = tentative_g;
                came_from[nidx] = current;
                let f = tentative_g + heuristic(np, goal);
                open.push(Node {
                    f,
                    tie: counter,
                    idx: nidx,
                });
                counter += 1;
            }
        }
    }

    PathResult::Unreachable
}

/// Manhattan-distance heuristic (admissible for 8-directional movement with
/// unit cost, since it underestimates the true cost).
fn heuristic(a: Pos, b: Pos) -> u32 {
    a.manhattan(b)
}

/// A simple BFS path (used as a fallback and for testing A* correctness).
pub fn bfs_path(level: &Level, start: Pos, goal: Pos) -> Option<Vec<Pos>> {
    if !level.in_bounds(start) || !level.in_bounds(goal) {
        return None;
    }
    if start == goal {
        return Some(vec![start]);
    }
    if !level.tiles[level.idx(start)].walkable() || !level.tiles[level.idx(goal)].walkable() {
        return None;
    }
    let start_idx = level.idx(start);
    let goal_idx = level.idx(goal);
    let mut came_from = vec![usize::MAX; level.tiles.len()];
    let mut visited = vec![false; level.tiles.len()];
    let mut queue = VecDeque::new();
    queue.push_back(start_idx);
    visited[start_idx] = true;
    while let Some(cur) = queue.pop_front() {
        if cur == goal_idx {
            let mut path = vec![goal];
            let mut c = goal_idx;
            while c != start_idx {
                c = came_from[c];
                path.push(Level::pos(c));
            }
            path.reverse();
            return Some(path);
        }
        let cpos = Level::pos(cur);
        for d in crate::core::action::Direction::ALL {
            let (dx, dy) = d.delta();
            let np = Pos::new(
                (cpos.x as i32 + dx).max(0) as u32,
                (cpos.y as i32 + dy).max(0) as u32,
            );
            if !level.in_bounds(np) {
                continue;
            }
            let nidx = level.idx(np);
            if visited[nidx] || !level.tiles[nidx].walkable() {
                continue;
            }
            visited[nidx] = true;
            came_from[nidx] = cur;
            queue.push_back(nidx);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::level::Tile;

    fn corridor_level() -> Level {
        let mut lvl = Level::blank(1);
        // A straight horizontal corridor from (2,10) to (30,10).
        for x in 2..=30 {
            lvl.carve(Pos::new(x, 10));
        }
        lvl
    }

    #[test]
    fn astar_finds_straight_path() {
        let lvl = corridor_level();
        let start = Pos::new(2, 10);
        let goal = Pos::new(30, 10);
        match astar(&lvl, start, goal) {
            PathResult::Found(path) => {
                assert_eq!(path[0], start);
                assert_eq!(path[path.len() - 1], goal);
                assert_eq!(path.len(), 29); // 28 steps + 1
            }
            PathResult::Unreachable => panic!("should find a path"),
        }
    }

    #[test]
    fn astar_unreachable_when_walled() {
        let mut lvl = corridor_level();
        // Wall off the middle of the corridor.
        lvl.set_tile(Pos::new(15, 10), Tile::Wall);
        let start = Pos::new(2, 10);
        let goal = Pos::new(30, 10);
        assert_eq!(astar(&lvl, start, goal), PathResult::Unreachable);
    }

    #[test]
    fn astar_same_tile() {
        let lvl = corridor_level();
        let p = Pos::new(10, 10);
        assert_eq!(astar(&lvl, p, p), PathResult::Found(vec![p]));
    }

    #[test]
    fn astar_matches_bfs_length() {
        // Build a level with a bend so the path is non-trivial.
        let mut lvl = Level::blank(1);
        for x in 2..=20 {
            lvl.carve(Pos::new(x, 5));
        }
        for y in 5..=15 {
            lvl.carve(Pos::new(20, y));
        }
        for x in 5..=20 {
            lvl.carve(Pos::new(x, 15));
        }
        let start = Pos::new(2, 5);
        let goal = Pos::new(5, 15);
        let a = astar(&lvl, start, goal);
        let b = bfs_path(&lvl, start, goal);
        match (a, b) {
            (PathResult::Found(pa), Some(pb)) => {
                assert_eq!(pa.len(), pb.len(), "A* and BFS should agree on length");
                assert_eq!(pa[0], start);
                assert_eq!(pa[pa.len() - 1], goal);
            }
            _ => panic!("both should find a path"),
        }
    }

    #[test]
    fn astar_path_is_walkable_and_connected() {
        let lvl = corridor_level();
        let start = Pos::new(2, 10);
        let goal = Pos::new(30, 10);
        if let PathResult::Found(path) = astar(&lvl, start, goal) {
            for p in &path {
                assert!(
                    lvl.tiles[lvl.idx(*p)].walkable(),
                    "path tile {p:?} not walkable"
                );
            }
            for w in path.windows(2) {
                let (a, b) = (w[0], w[1]);
                assert!(a.manhattan(b) <= 1, "path step {a:?} -> {b:?} not adjacent");
            }
        }
    }
}
