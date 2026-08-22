//! A single dungeon level: an 80x25 grid of tiles plus the features placed on
//! it (traps, items, monsters, stairs).

use crate::core::events::Pos;
use crate::core::rng::Rng;
use crate::entities::npc::Npc;
use serde::{Deserialize, Serialize};

/// Level width in tiles.
pub const WIDTH: u32 = 80;
/// Level height in tiles.
pub const HEIGHT: u32 = 25;

/// A single tile type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tile {
    /// Solid rock.
    Wall,
    /// Open floor.
    Floor,
    /// A closed door (blocks movement and sight).
    DoorClosed,
    /// An open door (blocks nothing).
    DoorOpen,
    /// A locked door (blocks movement; needs a key or lockpick).
    DoorLocked,
    /// Water: walkable but slow.
    Water,
    /// Lava: walkable but damaging.
    Lava,
    /// Spore gas: walkable, may poison.
    SporeGas,
    /// Stairs down to the next level.
    StairsDown,
    /// Stairs up to the previous level.
    StairsUp,
}

impl Tile {
    /// Whether a creature can move onto this tile.
    pub fn walkable(self) -> bool {
        matches!(
            self,
            Tile::Floor
                | Tile::DoorOpen
                | Tile::Water
                | Tile::Lava
                | Tile::SporeGas
                | Tile::StairsDown
                | Tile::StairsUp
        )
    }

    /// Whether this tile blocks line of sight.
    pub fn transparent(self) -> bool {
        !matches!(self, Tile::Wall | Tile::DoorClosed | Tile::DoorLocked)
    }

    /// Whether this tile is a door of any kind.
    pub fn is_door(self) -> bool {
        matches!(self, Tile::DoorClosed | Tile::DoorOpen | Tile::DoorLocked)
    }

    /// Whether this tile is a hazard (water/lava/gas).
    pub fn is_hazard(self) -> bool {
        matches!(self, Tile::Water | Tile::Lava | Tile::SporeGas)
    }
}

/// A trap hidden on a floor tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trap {
    Arrow,
    Dart,
    FallingItem,
    Teleport,
    SleepGas,
    AcidPool,
}

impl Trap {
    pub fn name(self) -> &'static str {
        match self {
            Trap::Arrow => "arrow trap",
            Trap::Dart => "dart trap",
            Trap::FallingItem => "falling item trap",
            Trap::Teleport => "teleport trap",
            Trap::SleepGas => "sleep gas trap",
            Trap::AcidPool => "acid pool",
        }
    }
}

/// A single dungeon level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    /// 1-based depth (1..=25, then endless).
    pub depth: u32,
    /// The tile grid, row-major: `tiles[y * WIDTH + x]`.
    pub tiles: Vec<Tile>,
    /// Traps by tile index (None = no trap).
    pub traps: Vec<Option<Trap>>,
    /// Item ids on the floor, by tile index (None = empty).
    pub items: Vec<Option<u32>>,
    /// Monster ids on this level, by tile index (None = empty).
    pub monsters: Vec<Option<u32>>,
    /// Where the player starts (or re-enters) this level.
    pub player_start: Pos,
    /// Position of the down-stairs, if any.
    pub stairs_down: Option<Pos>,
    /// Position of the up-stairs, if any.
    pub stairs_up: Option<Pos>,
    /// NPCs on this level.
    pub npcs: Vec<Npc>,
    /// Tiles the player has ever seen (for remembered rendering).
    pub seen: Vec<bool>,
    /// Tiles currently in field of view.
    pub visible: Vec<bool>,
    /// Whether this level has been visited at least once.
    pub visited: bool,
}

impl Level {
    /// Create a blank level (all walls) at the given depth.
    pub fn blank(depth: u32) -> Self {
        let n = (WIDTH * HEIGHT) as usize;
        Self {
            depth,
            tiles: vec![Tile::Wall; n],
            traps: vec![None; n],
            items: vec![None; n],
            monsters: vec![None; n],
            player_start: Pos::new(1, 1),
            stairs_down: None,
            stairs_up: None,
            npcs: Vec::new(),
            seen: vec![false; n],
            visible: vec![false; n],
            visited: false,
        }
    }

    pub fn idx(&self, pos: Pos) -> usize {
        (pos.y as usize) * WIDTH as usize + pos.x as usize
    }

    pub fn pos(idx: usize) -> Pos {
        Pos::new((idx % WIDTH as usize) as u32, (idx / WIDTH as usize) as u32)
    }

    pub fn in_bounds(&self, pos: Pos) -> bool {
        pos.x < WIDTH && pos.y < HEIGHT
    }

    pub fn tile_at(&self, pos: Pos) -> Option<Tile> {
        if self.in_bounds(pos) {
            Some(self.tiles[self.idx(pos)])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, pos: Pos, tile: Tile) {
        if self.in_bounds(pos) {
            let idx = self.idx(pos);
            self.tiles[idx] = tile;
        }
    }

    /// Carve a floor tile (used by generation and connectivity repair).
    pub fn carve(&mut self, pos: Pos) {
        if self.in_bounds(pos) {
            let idx = self.idx(pos);
            self.tiles[idx] = Tile::Floor;
        }
    }

    /// All floor-ish (walkable) positions.
    pub fn walkable_positions(&self) -> Vec<Pos> {
        let mut out = Vec::new();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let p = Pos::new(x, y);
                if self.tiles[self.idx(p)].walkable() {
                    out.push(p);
                }
            }
        }
        out
    }

    /// A random walkable position (for spawning).
    pub fn random_walkable(&mut self, rng: &mut Rng) -> Option<Pos> {
        let positions = self.walkable_positions();
        rng.pick(&positions)
    }

    /// A random walkable position at least `min_dist` (Manhattan) from `from`.
    pub fn random_walkable_far(&mut self, rng: &mut Rng, from: Pos, min_dist: u32) -> Option<Pos> {
        let positions: Vec<Pos> = self
            .walkable_positions()
            .into_iter()
            .filter(|p| p.manhattan(from) >= min_dist)
            .collect();
        rng.pick(&positions)
    }

    /// BFS reachability from `start` over walkable tiles. Returns the set of
    /// reachable tile indices.
    pub fn reachable_from(&self, start: Pos) -> Vec<bool> {
        let mut reached = vec![false; self.tiles.len()];
        if !self.in_bounds(start) || !self.tiles[self.idx(start)].walkable() {
            return reached;
        }
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        reached[self.idx(start)] = true;
        while let Some(p) = queue.pop_front() {
            for d in crate::core::action::Direction::ALL {
                let (dx, dy) = d.delta();
                let np = Pos::new(
                    (p.x as i32 + dx).max(0) as u32,
                    (p.y as i32 + dy).max(0) as u32,
                );
                if !self.in_bounds(np) {
                    continue;
                }
                let i = self.idx(np);
                if reached[i] || !self.tiles[i].walkable() {
                    continue;
                }
                reached[i] = true;
                queue.push_back(np);
            }
        }
        reached
    }

    /// Verify that every walkable tile is reachable from `start`. Returns the
    /// list of unreachable walkable positions (empty = fully connected).
    pub fn unreachable_walkable(&self, start: Pos) -> Vec<Pos> {
        let reached = self.reachable_from(start);
        self.walkable_positions()
            .into_iter()
            .filter(|p| !reached[self.idx(*p)])
            .collect()
    }

    /// Carve a straight corridor between two positions (L-shaped), fixing
    /// connectivity. Used as a repair pass after generation.
    pub fn carve_corridor(&mut self, a: Pos, b: Pos) {
        let mut x = a.x as i32;
        let mut y = a.y as i32;
        let tx = b.x as i32;
        let ty = b.y as i32;
        // Horizontal first, then vertical (or vice versa, randomly-ish by parity).
        if (a.x + a.y).is_multiple_of(2) {
            while x != tx {
                x += if x < tx { 1 } else { -1 };
                self.carve(Pos::new(x as u32, y as u32));
            }
            while y != ty {
                y += if y < ty { 1 } else { -1 };
                self.carve(Pos::new(x as u32, y as u32));
            }
        } else {
            while y != ty {
                y += if y < ty { 1 } else { -1 };
                self.carve(Pos::new(x as u32, y as u32));
            }
            while x != tx {
                x += if x < tx { 1 } else { -1 };
                self.carve(Pos::new(x as u32, y as u32));
            }
        }
    }

    /// Repair connectivity: carve corridors from the start to every
    /// unreachable walkable region. Returns the number of corridors carved.
    pub fn repair_connectivity(&mut self, start: Pos) -> usize {
        let mut carved = 0;
        // Iterate until stable (carving can create new regions).
        for _ in 0..64 {
            let unreachable = self.unreachable_walkable(start);
            if unreachable.is_empty() {
                break;
            }
            // Connect the first unreachable tile to the nearest reachable one.
            let target = unreachable[0];
            let reached = self.reachable_from(start);
            let mut best: Option<(Pos, u32)> = None;
            for p in self.walkable_positions() {
                if !reached[self.idx(p)] {
                    continue;
                }
                let d = p.manhattan(target);
                if best.is_none_or(|(_, bd)| d < bd) {
                    best = Some((p, d));
                }
            }
            if let Some((from, _)) = best {
                self.carve_corridor(from, target);
                carved += 1;
            } else {
                break;
            }
        }
        carved
    }

    /// Place a trap on a floor tile.
    pub fn place_trap(&mut self, pos: Pos, trap: Trap) {
        if self.in_bounds(pos) {
            let idx = self.idx(pos);
            if self.tiles[idx].walkable() {
                self.traps[idx] = Some(trap);
            }
        }
    }

    /// Place an item id on a floor tile.
    pub fn place_item(&mut self, pos: Pos, item_id: u32) {
        if self.in_bounds(pos) {
            let idx = self.idx(pos);
            if self.tiles[idx].walkable() {
                self.items[idx] = Some(item_id);
            }
        }
    }

    /// Place a monster id on a floor tile.
    pub fn place_monster(&mut self, pos: Pos, monster_id: u32) {
        if self.in_bounds(pos) {
            let idx = self.idx(pos);
            if self.tiles[idx].walkable() {
                self.monsters[idx] = Some(monster_id);
            }
        }
    }

    /// The monster id at a position, if any.
    pub fn monster_at(&self, pos: Pos) -> Option<u32> {
        if self.in_bounds(pos) {
            self.monsters[self.idx(pos)]
        } else {
            None
        }
    }

    /// The item id at a position, if any.
    pub fn item_at(&self, pos: Pos) -> Option<u32> {
        if self.in_bounds(pos) {
            self.items[self.idx(pos)]
        } else {
            None
        }
    }

    /// The trap at a position, if any.
    pub fn trap_at(&self, pos: Pos) -> Option<Trap> {
        if self.in_bounds(pos) {
            self.traps[self.idx(pos)]
        } else {
            None
        }
    }

    /// Remove the trap at a position (after it triggers).
    pub fn remove_trap(&mut self, pos: Pos) {
        if self.in_bounds(pos) {
            let idx = self.idx(pos);
            self.traps[idx] = None;
        }
    }

    /// Remove the item at a position (after pickup).
    pub fn remove_item(&mut self, pos: Pos) {
        if self.in_bounds(pos) {
            let idx = self.idx(pos);
            self.items[idx] = None;
        }
    }

    /// Remove the monster at a position (after death).
    pub fn remove_monster(&mut self, pos: Pos) {
        if self.in_bounds(pos) {
            let idx = self.idx(pos);
            self.monsters[idx] = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_level_is_all_walls() {
        let lvl = Level::blank(1);
        assert!(lvl.tiles.iter().all(|t| *t == Tile::Wall));
        assert_eq!(lvl.tiles.len(), (WIDTH * HEIGHT) as usize);
    }

    #[test]
    fn carve_and_walkable() {
        let mut lvl = Level::blank(1);
        lvl.carve(Pos::new(5, 5));
        assert!(lvl.tile_at(Pos::new(5, 5)).unwrap().walkable());
        assert!(!lvl.tile_at(Pos::new(6, 5)).unwrap().walkable());
    }

    #[test]
    fn corridor_connects_regions() {
        let mut lvl = Level::blank(1);
        // Two isolated floor tiles.
        lvl.carve(Pos::new(2, 2));
        lvl.carve(Pos::new(10, 10));
        assert_eq!(lvl.unreachable_walkable(Pos::new(2, 2)).len(), 1);
        lvl.carve_corridor(Pos::new(2, 2), Pos::new(10, 10));
        assert!(lvl.unreachable_walkable(Pos::new(2, 2)).is_empty());
    }

    #[test]
    fn repair_connectivity_fixes_islands() {
        let mut lvl = Level::blank(1);
        lvl.carve(Pos::new(2, 2));
        lvl.carve(Pos::new(20, 20));
        lvl.carve(Pos::new(40, 5));
        let carved = lvl.repair_connectivity(Pos::new(2, 2));
        assert!(carved >= 2);
        assert!(lvl.unreachable_walkable(Pos::new(2, 2)).is_empty());
    }

    #[test]
    fn doors_block_movement_and_sight() {
        let mut lvl = Level::blank(1);
        lvl.set_tile(Pos::new(5, 5), Tile::DoorClosed);
        assert!(!lvl.tile_at(Pos::new(5, 5)).unwrap().walkable());
        assert!(!lvl.tile_at(Pos::new(5, 5)).unwrap().transparent());
        lvl.set_tile(Pos::new(5, 5), Tile::DoorOpen);
        assert!(lvl.tile_at(Pos::new(5, 5)).unwrap().walkable());
        assert!(lvl.tile_at(Pos::new(5, 5)).unwrap().transparent());
    }
}
