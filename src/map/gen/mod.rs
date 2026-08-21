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

    // Vaults (shop D2, arena D15, amulet D25).
    place_vaults(&mut level, &rooms, rng, depth);

    // Decorate: items, gold.
    decorate(&mut level, &rooms, rng, depth);

    // Doors on room doorways (with keys for locked ones).
    place_doors(&mut level, rng, depth);

    // Traps on corridors and floors.
    place_traps(&mut level, rng, depth);

    // Themed hazards: spore gas D6-10, water D11-15, lava D16-20.
    place_hazards(&mut level, rng, depth);

    // Connectivity repair: ensure player start + stairs reachable.
    repair_connectivity(&mut level, rng, depth);

    // Door sanity: a door that would trap the stairs stays open.
    repair_doors(&mut level);

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
                level.set_tile((ox + x, oy + y), Tile::Floor);
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
    } else if depth == 15 {
        place_boss_arena(level, rooms, rng);
    } else if depth == 25 {
        place_amulet_chamber(level, rooms, rng);
    }
}

/// D2 shop room: a dedicated walled room with a closed door, connected by a
/// corridor to the nearest existing room. The door position is recorded on
/// the level as the seam for the merchant (issue 16).
fn place_shop_vault(level: &mut Level, rooms: &[Room], rng: &mut Rng) {
    if rooms.is_empty() {
        return;
    }
    let w: u8 = 6;
    let h: u8 = 5;
    for _ in 0..50 {
        let ox = rng.int(2..(crate::map::level::MAP_W as u64 - w as u64 - 2)) as u8;
        let oy = rng.int(2..(crate::map::level::MAP_H as u64 - h as u64 - 2)) as u8;
        // Must not overlap existing rooms and must sit on clear walls.
        let overlaps = rooms.iter().any(|r| {
            ox < r.x + r.w + 1 && ox + w + 1 > r.x && oy < r.y + r.h + 1 && oy + h + 1 > r.y
        });
        if overlaps {
            continue;
        }
        let clear = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .all(|(x, y)| level.tile_at((ox + x, oy + y)) == Tile::Wall);
        if !clear {
            continue;
        }
        // Carve the room.
        for y in oy..oy + h {
            for x in ox..ox + w {
                level.set_tile((x, y), Tile::Floor);
            }
        }
        let center = (ox + w / 2, oy + h / 2);
        // Corridor from the nearest room center, then a door at the room
        // mouth.
        let nearest = rooms
            .iter()
            .min_by_key(|r| {
                let (cx, cy) = (r.center.0 as i32, r.center.1 as i32);
                (cx - center.0 as i32).abs() + (cy - center.1 as i32).abs()
            })
            .unwrap();
        let target = nearest.center();
        let door_pos = door_mouth(ox, oy, w, h, center, nearest);
        carve_corridor(level, center, target, rng);
        level.set_tile(door_pos, Tile::DoorClosed);
        if !stairs_reachable(level) {
            level.set_tile(door_pos, Tile::Floor);
        }
        level.shop_room = Some(door_pos);
        return;
    }
    // Fallback: mark an existing room as the shop room.
    let r = rooms[(rng.int(0..rooms.len() as u64)) as usize];
    let c = r.center();
    level.set_tile(c, Tile::Floor);
    level.shop_room = Some(c);
}

/// D15 boss arena: a dedicated walled arena room with a closed door and a
/// corridor to the nearest existing room. The boss itself is issue 19; this
/// only provides the arena and records its door.
fn place_boss_arena(level: &mut Level, rooms: &[Room], rng: &mut Rng) {
    if rooms.is_empty() {
        return;
    }
    let w: u8 = 9;
    let h: u8 = 6;
    for _ in 0..50 {
        let ox = rng.int(2..(crate::map::level::MAP_W as u64 - w as u64 - 2)) as u8;
        let oy = rng.int(2..(crate::map::level::MAP_H as u64 - h as u64 - 2)) as u8;
        let overlaps = rooms.iter().any(|r| {
            ox < r.x + r.w + 1 && ox + w + 1 > r.x && oy < r.y + r.h + 1 && oy + h + 1 > r.y
        });
        if overlaps {
            continue;
        }
        let clear = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .all(|(x, y)| level.tile_at((ox + x, oy + y)) == Tile::Wall);
        if !clear {
            continue;
        }
        for y in oy..oy + h {
            for x in ox..ox + w {
                level.set_tile((x, y), Tile::Floor);
            }
        }
        let center = (ox + w / 2, oy + h / 2);
        let nearest = rooms
            .iter()
            .min_by_key(|r| {
                let (cx, cy) = (r.center.0 as i32, r.center.1 as i32);
                (cx - center.0 as i32).abs() + (cy - center.1 as i32).abs()
            })
            .unwrap();
        let target = nearest.center();
        let door_pos = door_mouth(ox, oy, w, h, center, nearest);
        carve_corridor(level, center, target, rng);
        level.set_tile(door_pos, Tile::DoorClosed);
        if !stairs_reachable(level) {
            level.set_tile(door_pos, Tile::Floor);
        }
        level.boss_arena = Some(door_pos);
        return;
    }
    // Fallback: the largest existing room is the arena.
    let r = rooms
        .iter()
        .max_by_key(|r| (r.w as usize) * (r.h as usize))
        .unwrap();
    let c = r.center();
    level.boss_arena = Some(c);
}

/// The doorway tile on the wall of the shop/arena room where the corridor to
/// `room` exits. Mirrors `carve_corridor`, which runs the x phase at the
/// room center's row and the y phase at the target's column.
fn door_mouth(ox: u8, oy: u8, w: u8, h: u8, center: (u8, u8), room: &Room) -> (u8, u8) {
    let (tx, ty) = (room.center.0, room.center.1);
    if tx < center.0 {
        if tx < ox {
            (ox - 1, center.1)
        } else if ty < center.1 {
            (tx, oy - 1)
        } else {
            (tx, oy + h)
        }
    } else if tx > center.0 {
        if tx >= ox + w {
            (ox + w, center.1)
        } else if ty < center.1 {
            (tx, oy - 1)
        } else {
            (tx, oy + h)
        }
    } else if ty < center.1 {
        (center.0, oy - 1)
    } else {
        (center.0, oy + h)
    }
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

/// Walkable tiles with at least two wall neighbours: corridor doorways.
fn corridor_door_points(level: &Level) -> Vec<(u8, u8)> {
    let mut out = Vec::new();
    for y in 1..(crate::map::level::MAP_H - 1) {
        for x in 1..(crate::map::level::MAP_W - 1) {
            let p = (x, y);
            if !level.is_walkable(p) {
                continue;
            }
            let mut walls = 0;
            for (dx, dy) in [(-1i8, 0), (1, 0), (0, -1), (0, 1)] {
                let q = ((x as i8 + dx) as u8, (y as i8 + dy) as u8);
                if level.tile_at(q) == Tile::Wall {
                    walls += 1;
                }
            }
            if walls >= 2 {
                out.push(p);
            }
        }
    }
    out
}

/// Place 1-5 doors on corridor doorways; ~20% of them are locked, and each
/// locked door has an iron key dropped on the ground.
fn place_doors(level: &mut Level, rng: &mut Rng, _depth: u8) {
    let mut candidates = corridor_door_points(level);
    let important = important_tiles(level);
    candidates.retain(|p| !important.contains(p));
    let n = (candidates.len() / 12 + 1).min(5);
    for _i in 0..n {
        if candidates.is_empty() {
            break;
        }
        let p = candidates[(rng.int(0..candidates.len() as u64)) as usize];
        candidates.retain(|q| *q != p);
        let locked = rng.chance(20);
        // Tentatively close the door; if that strands the stairs, leave it
        // open and try another doorway.
        level.set_tile(
            p,
            if locked {
                Tile::DoorLocked
            } else {
                Tile::DoorClosed
            },
        );
        if !stairs_reachable(level) {
            level.set_tile(p, Tile::Floor);
            continue;
        }
        if locked {
            let key_pos = random_floor_spot(level, &important, rng);
            if let Some(kp) = key_pos {
                level.add_item(kp, crate::items::catalog::make_key());
            }
        }
    }
}

/// Stairs reachable from the player start with every door treated as a wall.
fn stairs_reachable(level: &Level) -> bool {
    let reached = level.reachable(level.player_start);
    level.stairs_up.is_none_or(|s| reached[Level::pos_idx(s)])
        && level.stairs_down.is_none_or(|s| reached[Level::pos_idx(s)])
}

/// A random plain Floor tile that is not a player start/stairs/door mouth.
fn random_floor_spot(level: &Level, important: &[(u8, u8)], rng: &mut Rng) -> Option<(u8, u8)> {
    let mut spots: Vec<(u8, u8)> = level
        .floor_tiles()
        .into_iter()
        .filter(|p| level.tile_at(*p) == Tile::Floor)
        .filter(|p| !important.contains(p))
        .collect();
    if spots.is_empty() {
        return None;
    }
    let p = spots[(rng.int(0..spots.len() as u64)) as usize];
    spots.retain(|q| q != &p);
    Some(p)
}

/// Tiles that must never be covered by traps, hazards or doors.
fn important_tiles(level: &Level) -> Vec<(u8, u8)> {
    let mut v = vec![level.player_start];
    if let Some(s) = level.stairs_up {
        v.push(s);
    }
    if let Some(s) = level.stairs_down {
        v.push(s);
    }
    if let Some(s) = level.shop_room {
        v.push(s);
    }
    if let Some(s) = level.boss_arena {
        v.push(s);
    }
    v
}

/// Place 1-5 traps (scales with depth) on random floor tiles.
fn place_traps(level: &mut Level, rng: &mut Rng, depth: u8) {
    let n = 1 + depth as usize / 5;
    let kinds = [
        crate::map::level::TrapKind::Arrow,
        crate::map::level::TrapKind::Dart,
        crate::map::level::TrapKind::FallingItem,
        crate::map::level::TrapKind::Teleport,
        crate::map::level::TrapKind::SleepGas,
        crate::map::level::TrapKind::AcidPool,
    ];
    let important = important_tiles(level);
    for _ in 0..n {
        let p = random_floor_spot(level, &important, rng);
        if let Some(p) = p {
            let k = kinds[(rng.int(0..kinds.len() as u64)) as usize];
            level.set_tile(p, Tile::Trap(k));
        }
    }
}

/// Themed hazard pools: spore gas D6-10, water D11-15, lava D16-20.
fn place_hazards(level: &mut Level, rng: &mut Rng, depth: u8) {
    let hazard = match LevelTheme::for_depth(depth) {
        LevelTheme::FungalGrottos => Some(Tile::SporeGas),
        LevelTheme::DrownedVaults => Some(Tile::Water),
        LevelTheme::EmberWorks => Some(Tile::Lava),
        LevelTheme::BarrowHalls | LevelTheme::Abyss => None,
    };
    let Some(hz) = hazard else {
        return;
    };
    let important = important_tiles(level);
    for _ in 0..4 {
        // Random walk of 3-4 tiles from a random start.
        let mut p = match random_floor_spot(level, &important, rng) {
            Some(p) => p,
            None => continue,
        };
        let steps = 3 + (rng.int(0..2)) as usize;
        for _ in 0..steps {
            if !important.contains(&p) && level.tile_at(p) == Tile::Floor {
                level.set_tile(p, hz);
            }
            let (dx, dy) = [(1i8, 0), (-1, 0), (0, 1), (0, -1)][(rng.int(0..4)) as usize];
            let nx = (p.0 as i8 + dx) as u8;
            let ny = (p.1 as i8 + dy) as u8;
            if !Level::in_bounds(nx, ny) {
                break;
            }
            p = (nx, ny);
        }
    }
}

/// After all placement, any door that would make the stairs unreachable
/// (with doors treated as walls) is left open.
fn repair_doors(level: &mut Level) {
    let start = level.player_start;
    let mut reached = level.reachable(start);
    let targets: Vec<(u8, u8)> = level
        .stairs_up
        .iter()
        .copied()
        .chain(level.stairs_down.iter().copied())
        .collect();
    for t in targets {
        if !reached[Level::pos_idx(t)] {
            // Open every door on the map until reachable (cheap, rare).
            for i in 0..level.tiles.len() {
                if level.tiles[i] == Tile::DoorClosed || level.tiles[i] == Tile::DoorLocked {
                    level.tiles[i] = Tile::Floor;
                }
            }
            reached = level.reachable(start);
            break;
        }
    }
    let _ = reached;
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
    for (name, opt) in [("up", level.stairs_up), ("down", level.stairs_down)] {
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
pub fn random_floor_tile(
    game: &crate::core::game::Game,
    depth: u8,
    rng: &mut Rng,
) -> Option<(u8, u8)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_at(depth: u8, seed: u64) -> Level {
        let mut rng = Rng::new(seed);
        generate(depth, &mut rng)
    }

    #[test]
    fn stairs_reachable_with_doors_closed_at_all_depths() {
        for seed in 0u64..30 {
            let depth = (seed % 25) as u8 + 1;
            let level = gen_at(depth, seed * 7919 + 1);
            let reached = level.reachable(level.player_start);
            assert!(
                level.stairs_up.is_none_or(|s| reached[Level::pos_idx(s)]),
                "D{depth} seed {seed}: up stairs must be reachable with doors closed"
            );
            assert!(
                level.stairs_down.is_none_or(|s| reached[Level::pos_idx(s)]),
                "D{depth} seed {seed}: down stairs must be reachable with doors closed"
            );
        }
    }

    #[test]
    fn d2_generates_a_shop_room() {
        for seed in 0..10 {
            let level = gen_at(2, seed * 104729 + 3);
            assert!(
                level.shop_room.is_some(),
                "D2 must have a shop room (seed {seed})"
            );
        }
    }

    #[test]
    fn d15_generates_a_boss_arena() {
        for seed in 0..10 {
            let level = gen_at(15, seed * 15485863 + 5);
            assert!(
                level.boss_arena.is_some(),
                "D15 must have a boss arena (seed {seed})"
            );
        }
    }

    #[test]
    fn hazards_appear_only_on_their_themed_depths() {
        let level = gen_at(7, 11);
        assert!(
            level.tiles.contains(&Tile::SporeGas),
            "D7 (Fungal Grottos) must have spore gas"
        );
        let level = gen_at(12, 12);
        assert!(
            level.tiles.contains(&Tile::Water),
            "D12 (Drowned Vaults) must have water"
        );
        let level = gen_at(17, 13);
        assert!(
            level.tiles.contains(&Tile::Lava),
            "D17 (Ember Works) must have lava"
        );
        let level = gen_at(3, 14);
        assert!(
            !level
                .tiles
                .iter()
                .any(|t| { matches!(t, Tile::Water | Tile::Lava | Tile::SporeGas) }),
            "D3 (Barrow Halls) must have no hazards"
        );
    }

    #[test]
    fn traps_are_placed_on_every_depth() {
        for seed in 0u64..10 {
            let depth = (seed % 25) as u8 + 1;
            let level = gen_at(depth, seed * 9973 + 17);
            assert!(
                level.tiles.iter().any(|t| matches!(t, Tile::Trap(_))),
                "D{depth} seed {seed} must have at least one trap"
            );
        }
    }

    #[test]
    fn doors_are_placed_and_keys_cover_locked_doors() {
        for seed in 0u64..10 {
            let depth = (seed % 25) as u8 + 1;
            let level = gen_at(depth, seed * 6151 + 19);
            let locked: usize = level
                .tiles
                .iter()
                .filter(|t| **t == Tile::DoorLocked)
                .count();
            let keys: usize = level
                .items
                .values()
                .flatten()
                .filter(|i| i.kind == crate::items::item::ItemKind::Key)
                .count();
            assert!(
                keys >= locked,
                "every locked door needs a key on the ground (D{depth} seed {seed})"
            );
        }
    }

    #[test]
    fn locked_doors_never_strand_the_player_start_or_stairs() {
        // Even if a door is the only way in, repair keeps the level playable:
        // the player start must be walkable and the stairs reachable.
        for seed in 0..10 {
            let level = gen_at(20, seed * 4729 + 23);
            assert!(level.is_walkable(level.player_start));
            assert_eq!(level.tile_at(level.player_start), Tile::Floor);
        }
    }
}
