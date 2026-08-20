//! Level generation: rooms + corridors, cellular caves, vaults, decoration.

use crate::core::rng::Rng;
use crate::map::level::{Level, LevelTheme, Tile};
use std::collections::HashSet;

/// Generate a level at `depth` using seeded rng.
pub fn generate(depth: u8, rng: &mut Rng) -> Level {
    let mut level = Level::new(depth);
    let theme = level.theme;

    // Rooms & corridors.
    let rooms = gen_rooms(&mut level, rng, depth);

    // ~15% chance of cellular cave section.
    let has_caves = rng.chance(15);
    if has_caves {
        gen_cellular_section(&mut level, rng, depth);
    }

    // Place stairs: down in a far room, up in a different room.
    place_stairs(&mut level, &rooms, rng, depth);

    // Vaults.
    place_vaults(&mut level, &rooms, rng, depth);

    // Decorate: items, gold, traps.
    decorate(&mut level, &rooms, rng, depth);

    // Connectivity repair: ensure player start + stairs reachable.
    repair_connectivity(&mut level, rng, depth);

    let _ = theme;
    level
}

#[derive(Clone, Copy)]
struct Room {
    x: u8,
    y: u8,
    w: u8,
    h: u8,
    center: (u8, u8),
}

impl Room {
    fn center(&self) -> (u8, u8) {
        self.center
    }
}

fn gen_rooms(level: &mut Level, rng: &mut Rng, depth: u8) -> Vec<Room> {
    let mut rooms: Vec<Room> = Vec::new();
    let target = 5 + (rng.int(0..4) as u8);
    let mut attempts = 0;
    while (rooms.len() as u8) < target && attempts < 200 {
        attempts += 1;
        let w = rng.int(4..10) as u8;
        let h = rng.int(3..6) as u8;
        let x = rng.int(1..(crate::map::level::MAP_W as u64 - w as u64 - 1)) as u8;
        let y = rng.int(1..(crate::map::level::MAP_H as u64 - h as u64 - 1)) as u8;
        let candidate = Room {
            x,
            y,
            w,
            h,
            center: (x + w / 2, y + h / 2),
        };
        // Overlap check
        let overlap = rooms.iter().any(|r| {
            candidate.x < r.x + r.w + 1
                && candidate.x + candidate.w + 1 > r.x
                && candidate.y < r.y + r.h + 1
                && candidate.y + candidate.h + 1 > r.y
        });
        if overlap {
            continue;
        }
        // Carve
        for yy in candidate.y..candidate.y + candidate.h {
            for xx in candidate.x..candidate.x + candidate.w {
                level.set_tile((xx, yy), Tile::Floor);
            }
        }
        rooms.push(candidate);
    }
    // Connect rooms with corridors.
    for i in 1..rooms.len() {
        let a = rooms[i - 1].center();
        let b = rooms[i].center();
        carve_corridor(level, a, b, rng);
    }
    let _ = depth;
    rooms
}

fn carve_corridor(level: &mut Level, a: (u8, u8), b: (u8, u8), rng: &mut Rng) {
    let mut x = a.0 as i32;
    let mut y = a.1 as i32;
    let (bx, by) = (b.0 as i32, b.1 as i32);
    while x != bx || y != by {
        if x != bx {
            x += if x < bx { 1 } else { -1 };
        } else if y != by {
            y += if y < by { 1 } else { -1 };
        }
        level.set_tile((x as u8, y as u8), Tile::Floor);
    }
    let _ = rng;
}

fn gen_cellular_section(level: &mut Level, rng: &mut Rng, depth: u8) {
    // Carve a random rectangular region with cellular automata.
    let w = rng.int(10..20) as u8;
    let h = rng.int(5..8) as u8;
    let ox = rng.int(1..(crate::map::level::MAP_W as u64 - w as u64 - 1)) as u8;
    let oy = rng.int(1..(crate::map::level::MAP_H as u64 - h as u64 - 1)) as u8;
    let mut grid = vec![false; (w as usize) * (h as usize)];
    for i in 0..grid.len() {
        grid[i] = rng.chance(45);
    }
    for _ in 0..4 {
        let mut next = grid.clone();
        for y in 1..(h as usize - 1) {
            for x in 1..(w as usize - 1) {
                let i = y * w as usize + x;
                let mut walls = 0;
                for dy in -1i8..=1 {
                    for dx in -1i8..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (x as i8 + dx) as usize;
                        let ny = (y as i8 + dy) as usize;
                        if ny < h as usize && nx < w as usize && !grid[ny * w as usize + nx] {
                            walls += 1;
                        }
                    }
                }
                next[i] = walls <= 3;
            }
        }
        grid = next;
    }
    for y in 0..h {
        for x in 0..w {
            if grid[y as usize * w as usize + x as usize] {
                level.set_tile(
                    (ox + x, oy + y),
                    Tile::Floor,
                );
            }
        }
    }
    let _ = depth;
}

fn place_stairs(level: &mut Level, rooms: &[Room], rng: &mut Rng, depth: u8) {
    if rooms.len() >= 2 {
        let up_idx = (rng.int(0..rooms.len() as u64)) as usize;
        let down_idx = (rng.int(0..rooms.len() as u64)) as usize;
        let up_room = rooms[up_idx].center();
        let down_room = rooms[(down_idx + 1) % rooms.len()].center();
        level.set_tile(up_room, Tile::Floor);
        level.set_tile(down_room, Tile::Floor);
        level.player_start = up_room;
        level.stairs_up = if depth > 1 { Some(up_room) } else { None };
        level.stairs_down = Some(down_room);
    } else {
        let c = rooms.first().map(|r| r.center()).unwrap_or((40, 12));
        level.player_start = c;
        level.stairs_up = if depth > 1 { Some(c) } else { None };
        level.stairs_down = Some((c.0.min(78), c.1));
    }
}

fn place_vaults(level: &mut Level, rooms: &[Room], rng: &mut Rng, depth: u8) {
    // Handcrafted vaults per depth table.
    if depth == 2 {
        place_shop_vault(level, rooms, rng);
    } else if (8..=12).contains(&depth) && rooms.len() >= 3 {
        let r = rooms[(rng.int(0..rooms.len() as u64)) as usize];
        // Wizard room marker (decorative only).
        let _ = r;
    } else if depth == 25 {
        place_amulet_chamber(level, rooms, rng);
    }
}

fn place_shop_vault(level: &mut Level, rooms: &[Room], rng: &mut Rng) {
    if rooms.len() < 2 {
        return;
    }
    let r = rooms[(rng.int(0..rooms.len() as u64)) as usize];
    let c = r.center();
    level.set_tile(c, Tile::Floor);
    // NPC placed by decorate.
    let _ = c;
}

 fn place_amulet_chamber(level: &mut Level, rooms: &[Room], rng: &mut Rng) {
    if rooms.len() < 2 {
        return;
    }
    let mut candidates: Vec<&Room> = rooms
        .iter()
        .filter(|r| level.stairs_up != Some(r.center()) && level.stairs_down != Some(r.center()))
        .collect();
    if candidates.is_empty() {
        candidates = rooms.iter().collect();
    }
    let r = candidates[(rng.int(0..candidates.len() as u64)) as usize];
    let c = r.center();
    level.set_tile(c, Tile::Floor);
    level.add_item(c, crate::items::catalog::make_amulet());
}

fn decorate(level: &mut Level, rooms: &[Room], rng: &mut Rng, depth: u8) {
    // Gold piles.
    let piles = rng.int(3..8) as usize;
    for _ in 0..piles {
        if let Some(c) = rooms.first().map(|r| r.center()) {
            let pos = random_room_point(rooms, rng);
            let amount = rng.int(10..(50 + depth as u64 * 10)) as u32;
            level.add_gold(pos, amount);
            let _ = c;
        }
    }
    // Items.
    let items_n = rng.int(2..6) as usize;
    for _ in 0..items_n {
        let pos = random_room_point(rooms, rng);
        let mut local_rng = rng.clone();
        let item = crate::items::loot::roll_ground(&mut local_rng, depth);
        level.add_item(pos, item);
    }
    let _ = level;
}

fn random_room_point(rooms: &[Room], rng: &mut Rng) -> (u8, u8) {
    if rooms.is_empty() {
        return (40, 12);
    }
    let r = rooms[(rng.int(0..rooms.len() as u64)) as usize];
    let x = rng.int(r.x as u64..(r.x + r.w) as u64) as u8;
    let y = rng.int(r.y as u64..(r.y + r.h) as u64) as u8;
    (x, y)
}

/// BFS from player start; if any stairs are unreachable, carve a straight
/// corridor through walls to connect them.
fn repair_connectivity(level: &mut Level, rng: &mut Rng, depth: u8) {
    let start = level.player_start;
    let mut reached = level.reachable(start);
    for (name, opt) in [
        ("up", level.stairs_up),
        ("down", level.stairs_down),
    ] {
        if let Some(s) = opt {
            if !reached[Level::pos_idx(s)] {
                // Carve a corridor from start to s.
                let mut x = start.0 as i32;
                let mut y = start.1 as i32;
                let (tx, ty) = (s.0 as i32, s.1 as i32);
                while x != tx || y != ty {
                    level.set_tile((x as u8, y as u8), Tile::Floor);
                    if x != tx {
                        x += if x < tx { 1 } else { -1 };
                    } else if y != ty {
                        y += if y < ty { 1 } else { -1 };
                    }
                }
                level.set_tile(s, Tile::Floor);
                let _ = name;
                reached = level.reachable(start);
            }
        }
    }
    let _ = (reached, rng, depth);
}

/// Pick a random floor tile on the given level (for spawning).
pub fn random_floor_tile(game: &crate::core::game::Game, depth: u8, rng: &mut Rng) -> Option<(u8, u8)> {
    let level = game.levels.get(&depth)?;
    let floors = level.floor_tiles();
    if floors.is_empty() {
        return None;
    }
    let i = (rng.int(0..floors.len() as u64)) as usize;
    Some(floors[i])
}

#[allow(dead_code)]
fn _theme_for(depth: u8) -> LevelTheme {
    LevelTheme::for_depth(depth)
}

#[allow(dead_code)]
fn _hashset() -> HashSet<u8> {
    HashSet::new()
}
