//! Level: the 80x25 tile grid, doors, stairs, items, tiles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::items::item::Item;

pub const MAP_W: u8 = 80;
pub const MAP_H: u8 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Wall,
    Floor,
    DoorClosed,
    DoorLocked,
    StairsDown,
    StairsUp,
    Water,
    Lava,
    SporeGas,
    Grate,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelTheme {
    #[default]
    BarrowHalls,
    FungalGrottos,
    DrownedVaults,
    EmberWorks,
    Abyss,
}

impl LevelTheme {
    pub fn for_depth(depth: u8) -> LevelTheme {
        match depth {
            1..=5 => LevelTheme::BarrowHalls,
            6..=10 => LevelTheme::FungalGrottos,
            11..=15 => LevelTheme::DrownedVaults,
            16..=20 => LevelTheme::EmberWorks,
            _ => LevelTheme::Abyss,
        }
    }

   pub fn name(&self) -> &'static str {
        match self {
            LevelTheme::BarrowHalls => "The Barrow Halls",
            LevelTheme::FungalGrottos => "The Fungal Grottos",
            LevelTheme::DrownedVaults => "The Drowned Vaults",
            LevelTheme::EmberWorks => "The Ember Works",
            LevelTheme::Abyss => "The Abyss",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Level {
    pub depth: u8,
    pub theme: LevelTheme,
    /// Indexed [y * W + x].
    pub tiles: Vec<Tile>,
    pub explored: Vec<bool>,
    pub seen: Vec<bool>,
    /// Items on the ground keyed by tile index.
    pub items: HashMap<usize, Vec<Item>>,
    /// Gold on the ground keyed by tile index.
    pub gold: HashMap<usize, u32>,
    pub stairs_up: Option<(u8, u8)>,
    pub stairs_down: Option<(u8, u8)>,
    pub player_start: (u8, u8),
}

impl Level {
    pub fn new(depth: u8) -> Self {
        let theme = LevelTheme::for_depth(depth);
        Self {
            depth,
            theme,
            tiles: vec![Tile::Wall; (MAP_W as usize) * (MAP_H as usize)],
            explored: vec![false; (MAP_W as usize) * (MAP_H as usize)],
            seen: vec![false; (MAP_W as usize) * (MAP_H as usize)],
            items: HashMap::new(),
            gold: HashMap::new(),
            stairs_up: None,
            stairs_down: None,
            player_start: (MAP_W / 2, MAP_H / 2),
        }
    }

    pub fn idx(x: u8, y: u8) -> usize {
        (y as usize) * (MAP_W as usize) + x as usize
    }

    pub fn pos_idx(pos: (u8, u8)) -> usize {
        Self::idx(pos.0, pos.1)
    }

    pub fn in_bounds(x: u8, y: u8) -> bool {
        x < MAP_W && y < MAP_H
    }

    pub fn is_in_bounds(&self, x: u8, y: u8) -> bool {
        Self::in_bounds(x, y)
    }

    pub fn tile_at(&self, pos: (u8, u8)) -> Tile {
        if !self.is_in_bounds(pos.0, pos.1) {
            return Tile::Wall;
        }
        self.tiles[pos.1 as usize * MAP_W as usize + pos.0 as usize]
    }

    pub fn set_tile(&mut self, pos: (u8, u8), t: Tile) {
        if self.is_in_bounds(pos.0, pos.1) {
            self.tiles[pos.1 as usize * MAP_W as usize + pos.0 as usize] = t;
        }
    }

    pub fn is_walkable(&self, pos: (u8, u8)) -> bool {
        match self.tile_at(pos) {
            Tile::Floor | Tile::StairsUp | Tile::StairsDown | Tile::DoorClosed | Tile::Water | Tile::Lava => true,
            _ => false,
        }
    }

    pub fn is_transparent(&self, pos: (u8, u8)) -> bool {
        match self.tile_at(pos) {
            Tile::Wall | Tile::DoorClosed | Tile::DoorLocked => false,
            _ => true,
        }
    }

    pub fn center(&self) -> (u8, u8) {
        self.player_start
    }

    pub fn items_at(&self, pos: (u8, u8)) -> Vec<Item> {
        let i = Self::pos_idx(pos);
        self.items.get(&i).cloned().unwrap_or_default()
    }

    pub fn take_item_at(&mut self, pos: (u8, u8)) -> Option<Item> {
        let i = Self::pos_idx(pos);
        let v = self.items.get_mut(&i)?;
        if v.is_empty() {
            None
        } else {
            v.pop()
        }
    }

    pub fn add_item(&mut self, pos: (u8, u8), item: Item) {
        let i = Self::pos_idx(pos);
        self.items.entry(i).or_default().push(item);
    }

    pub fn take_gold_at(&mut self, pos: (u8, u8)) -> Option<u32> {
        let i = Self::pos_idx(pos);
        self.gold.remove(&i)
    }

    pub fn add_gold(&mut self, pos: (u8, u8), amount: u32) {
        let i = Self::pos_idx(pos);
        *self.gold.entry(i).or_insert(0) += amount;
    }

    pub fn stairs_down_at(&self, pos: (u8, u8)) -> bool {
        self.stairs_down == Some(pos)
    }

    pub fn stairs_up_at(&self, pos: (u8, u8)) -> bool {
        self.stairs_up == Some(pos)
    }

    /// All floor tiles (for placement).
    pub fn floor_tiles(&self) -> Vec<(u8, u8)> {
        let mut out = Vec::new();
        for y in 0..MAP_H {
            for x in 0..MAP_W {
                if self.is_walkable((x, y)) {
                    out.push((x, y));
                }
            }
        }
        out
    }

    /// Random unexplored floor tile (for spawning).
    pub fn random_floor(&self, exclude: &[(u8, u8)]) -> Option<(u8, u8)> {
        let floors: Vec<(u8, u8)> = self
            .floor_tiles()
            .into_iter()
            .filter(|p| !exclude.contains(p))
            .collect();
        floors.first().copied()
    }

    /// BFS connectivity: returns set of tiles reachable from `start`.
    pub fn reachable(&self, start: (u8, u8)) -> Vec<bool> {
        let mut reached = vec![false; self.tiles.len()];
        let mut queue = std::collections::VecDeque::new();
        if self.is_walkable(start) {
            reached[Self::pos_idx(start)] = true;
            queue.push_back(start);
        }
        while let Some((x, y)) = queue.pop_front() {
            for (dx, dy) in [(1, 0i8), (-1, 0i8), (0, 1i8), (0, -1i8)] {
                let nx = (x as i8 + dx) as u8;
                let ny = (y as i8 + dy) as u8;
                if !Level::in_bounds(nx, ny) {
                    continue;
                }
                if !self.is_walkable((nx, ny)) {
                    continue;
                }
                let i = Self::idx(nx, ny);
                if !reached[i] {
                    reached[i] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
        reached
    }
}
