//! Level generation: rooms, corridors, features, and placement.
//!
//! The generator produces a fully-connected level with rooms, corridors,
//! hazards (theme-specific), stairs, monsters, items, traps, and NPCs.
//! All randomness flows through the injected `Rng` for determinism.

use crate::core::events::Pos;
use crate::core::rng::Rng;
use crate::data::monsters::MonsterDef;
use crate::data::themes::Theme;
use crate::entities::npc::Npc;
use crate::items::loot;
use crate::map::level::{HEIGHT, Level, Tile, Trap, WIDTH};

/// A rectangular room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Room {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
}

impl Room {
    fn center(&self) -> Pos {
        Pos::new((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    #[allow(dead_code)]
    fn contains(&self, p: Pos) -> bool {
        p.x >= self.x1 && p.x <= self.x2 && p.y >= self.y1 && p.y <= self.y2
    }

    fn intersects(&self, other: &Room) -> bool {
        self.x1 <= other.x2 + 1
            && self.x2 >= other.x1 - 1
            && self.y1 <= other.y2 + 1
            && self.y2 >= other.y1 - 1
    }
}

/// Generate a complete level at the given depth.
///
/// `rng` is the game's RNG (for determinism). `has_upstairs` indicates whether
/// this level should have an up-stairs (i.e. depth > 1).
pub fn generate_level(rng: &mut Rng, depth: u32, has_upstairs: bool) -> Level {
    let theme = Theme::for_depth(depth);
    let mut level = Level::blank(depth);

    // 1. Generate rooms.
    let rooms = generate_rooms(rng, &mut level);

    if rooms.is_empty() {
        // Fallback: a single large room.
        let room = Room {
            x1: 5,
            y1: 5,
            x2: 74,
            y2: 19,
        };
        carve_room(&mut level, &room);
        finalize_level(rng, &mut level, depth, has_upstairs, theme, &[room]);
        return level;
    }

    // 2. Carve corridors connecting rooms in sequence.
    for i in 1..rooms.len() {
        let a = rooms[i - 1].center();
        let b = rooms[i].center();
        level.carve_corridor(a, b);
    }

    // 3. Add theme-specific hazards.
    add_hazards(rng, &mut level, theme);

    // 4. Add doors at room entrances (a few).
    add_doors(rng, &mut level, &rooms);

    // 5. Ensure connectivity.
    let start = rooms[0].center();
    level.repair_connectivity(start);

    // 6. Place stairs.
    place_stairs(rng, &mut level, &rooms, has_upstairs, depth);

    // 7. Place the player start.
    level.player_start = rooms[0].center();

    // 8. Place monsters.
    place_monsters(rng, &mut level, depth, &rooms);

    // 9. Place items.
    place_items(rng, &mut level, depth, &rooms);

    // 10. Place traps.
    place_traps(rng, &mut level, depth, &rooms);

    // 11. Place NPCs (quest givers / shopkeepers).
    place_npcs(rng, &mut level, depth, &rooms);

    level.visited = true;
    level
}

/// Generate a set of non-overlapping rooms.
fn generate_rooms(rng: &mut Rng, level: &mut Level) -> Vec<Room> {
    let mut rooms = Vec::new();
    let max_rooms = 12;
    let mut attempts = 0;
    while rooms.len() < max_rooms && attempts < 200 {
        attempts += 1;
        let w: u32 = rng.range(4, 11);
        let h: u32 = rng.range(3, 7);
        let x = rng.range(1, (WIDTH - w - 1) as usize) as u32;
        let y = rng.range(1, (HEIGHT - h - 1) as usize) as u32;
        let room = Room {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        };
        if rooms.iter().any(|r: &Room| r.intersects(&room)) {
            continue;
        }
        carve_room(level, &room);
        rooms.push(room);
    }
    rooms
}

/// Carve a room into the level.
fn carve_room(level: &mut Level, room: &Room) {
    for y in room.y1..=room.y2 {
        for x in room.x1..=room.x2 {
            level.carve(Pos::new(x, y));
        }
    }
}

/// Add theme-specific hazards (water, lava, spore gas) to some floor tiles.
fn add_hazards(rng: &mut Rng, level: &mut Level, theme: Theme) {
    let hazard_chance: u32 = match theme {
        Theme::DrownedVaults => 8,
        Theme::EmberWorks => 8,
        Theme::FungalGrottos => 8,
        _ => 0,
    };
    if hazard_chance == 0 {
        return;
    }
    let hazard = match theme {
        Theme::DrownedVaults => Tile::Water,
        Theme::EmberWorks => Tile::Lava,
        Theme::FungalGrottos => Tile::SporeGas,
        _ => return,
    };
    let positions = level.walkable_positions();
    for p in positions {
        if rng.chance(hazard_chance) {
            level.set_tile(p, hazard);
        }
    }
}

/// Add a few doors at random room edges.
fn add_doors(rng: &mut Rng, level: &mut Level, rooms: &[Room]) {
    for room in rooms {
        if !rng.chance(40) {
            continue;
        }
        // Pick a random edge tile of the room.
        let edge = random_room_edge(rng, room);
        if let Some(e) = edge
            && level.tile_at(e) == Some(Tile::Floor)
        {
            level.set_tile(e, Tile::DoorClosed);
        }
    }
}

/// Pick a random tile on the perimeter of a room.
fn random_room_edge(rng: &mut Rng, room: &Room) -> Option<Pos> {
    let edges: Vec<Pos> = {
        let mut v = Vec::new();
        for x in room.x1..=room.x2 {
            v.push(Pos::new(x, room.y1));
            v.push(Pos::new(x, room.y2));
        }
        for y in room.y1..=room.y2 {
            v.push(Pos::new(room.x1, y));
            v.push(Pos::new(room.x2, y));
        }
        v
    };
    rng.pick(&edges)
}

/// Place the down-stairs (and up-stairs if applicable) in far rooms.
fn place_stairs(_rng: &mut Rng, level: &mut Level, rooms: &[Room], has_upstairs: bool, depth: u32) {
    if rooms.len() >= 2 {
        // Down stairs in the last room.
        let down_room = rooms.last().unwrap();
        let down_pos = down_room.center();
        level.set_tile(down_pos, Tile::StairsDown);
        level.stairs_down = Some(down_pos);

        // Up stairs in the first room (if not the surface).
        if has_upstairs {
            let up_room = rooms.first().unwrap();
            let up_pos = up_room.center();
            // Avoid placing up-stairs on the player start.
            if up_pos != down_pos {
                level.set_tile(up_pos, Tile::StairsUp);
                level.stairs_up = Some(up_pos);
            }
        }
    } else if let Some(room) = rooms.first() {
        // Single room: place stairs at opposite corners.
        let down_pos = Pos::new(room.x2 - 1, room.y2 - 1);
        level.set_tile(down_pos, Tile::StairsDown);
        level.stairs_down = Some(down_pos);
        if has_upstairs {
            let up_pos = Pos::new(room.x1 + 1, room.y1 + 1);
            level.set_tile(up_pos, Tile::StairsUp);
            level.stairs_up = Some(up_pos);
        }
    }
    let _ = depth;
}

/// Place monsters appropriate for the depth.
fn place_monsters(rng: &mut Rng, level: &mut Level, depth: u32, rooms: &[Room]) {
    let candidates = MonsterDef::for_depth(depth);
    if candidates.is_empty() {
        return;
    }
    // Number of monsters scales with depth.
    let count = (3 + depth / 3).min(15) as usize;
    let start = level.player_start;

    for _ in 0..count {
        let def = match rng.pick(&candidates) {
            Some(d) => d,
            None => break,
        };
        // Place in a random room, away from the player start.
        if let Some(room) = rng.pick(rooms) {
            let pos = room.center();
            // Jitter the position within the room.
            let jittered = Pos::new(
                (pos.x as i32 + rng.range(-2, 3)) as u32,
                (pos.y as i32 + rng.range(-2, 3)) as u32,
            );
            let target = if level.in_bounds(jittered)
                && level.tile_at(jittered) == Some(Tile::Floor)
                && jittered.manhattan(start) >= 5
            {
                jittered
            } else if pos.manhattan(start) >= 5 {
                pos
            } else {
                continue;
            };
            let idx = level.idx(target);
            if level.monsters[idx].is_none() {
                level.monsters[idx] = Some(def.id);
            }
        }
    }

    // Place the boss on boss depths.
    if let Some(boss) = MonsterDef::boss_for_depth(depth)
        && let Some(room) = rooms.last()
    {
        let pos = room.center();
        let idx = level.idx(pos);
        if level.monsters[idx].is_none() {
            level.monsters[idx] = Some(boss.id);
        }
    }
}

/// Place items on the floor.
fn place_items(rng: &mut Rng, level: &mut Level, depth: u32, rooms: &[Room]) {
    let count = (2 + depth / 4).min(10) as usize;
    for _ in 0..count {
        if let Some(room) = rng.pick(rooms) {
            let pos = room.center();
            let jittered = Pos::new(
                (pos.x as i32 + rng.range(-2, 3)) as u32,
                (pos.y as i32 + rng.range(-2, 3)) as u32,
            );
            let target =
                if level.in_bounds(jittered) && level.tile_at(jittered) == Some(Tile::Floor) {
                    jittered
                } else {
                    pos
                };
            let idx = level.idx(target);
            if level.items[idx].is_none() {
                let item = loot::generate_item(rng, depth);
                level.items[idx] = Some(item.def_id);
            }
        }
    }
}

/// Place traps on the floor.
fn place_traps(rng: &mut Rng, level: &mut Level, depth: u32, rooms: &[Room]) {
    let count = (1 + depth / 5).min(6) as usize;
    let all_traps = [
        Trap::Arrow,
        Trap::Dart,
        Trap::FallingItem,
        Trap::Teleport,
        Trap::SleepGas,
        Trap::AcidPool,
    ];
    for _ in 0..count {
        if let Some(room) = rng.pick(rooms) {
            let pos = room.center();
            let jittered = Pos::new(
                (pos.x as i32 + rng.range(-2, 3)) as u32,
                (pos.y as i32 + rng.range(-2, 3)) as u32,
            );
            let target =
                if level.in_bounds(jittered) && level.tile_at(jittered) == Some(Tile::Floor) {
                    jittered
                } else {
                    pos
                };
            let idx = level.idx(target);
            if level.traps[idx].is_none() && level.monsters[idx].is_none() {
                let trap = rng.pick(&all_traps).unwrap_or(Trap::Arrow);
                level.traps[idx] = Some(trap);
            }
        }
    }
}

/// Place NPCs (quest givers and shopkeepers) on appropriate depths.
fn place_npcs(rng: &mut Rng, level: &mut Level, depth: u32, rooms: &[Room]) {
    if rooms.len() < 2 {
        return;
    }
    // Shopkeeper on every 3rd depth.
    if depth.is_multiple_of(3)
        && let Some(room) = rooms.get(1)
    {
        let pos = room.center();
        if level.tile_at(pos) == Some(Tile::Floor) {
            level.npcs.push(Npc::shopkeeper(pos, "Merchant", depth));
        }
    }
    // Quest givers on specific depths.
    for quest in crate::data::quests::QuestDef::ALL {
        if depth >= quest.giver_depth_min
            && depth <= quest.giver_depth_max
            && let Some(room) = rooms.get(rooms.len() / 2)
        {
            let pos = room.center();
            if level.tile_at(pos) == Some(Tile::Floor) && level.npcs.iter().all(|n| n.pos != pos) {
                level.npcs.push(Npc::quest_giver(pos, "Elder", quest.id));
            }
        }
    }
    let _ = rng;
}

/// Finalize a level (used by the fallback path).
fn finalize_level(
    rng: &mut Rng,
    level: &mut Level,
    depth: u32,
    has_upstairs: bool,
    theme: Theme,
    rooms: &[Room],
) {
    let _ = theme;
    place_stairs(rng, level, rooms, has_upstairs, depth);
    level.player_start = rooms[0].center();
    place_monsters(rng, level, depth, rooms);
    place_items(rng, level, depth, rooms);
    place_traps(rng, level, depth, rooms);
    place_npcs(rng, level, depth, rooms);
    level.visited = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_level_is_connected() {
        let mut rng = Rng::new(42);
        for depth in 1..=25 {
            let level = generate_level(&mut rng, depth, depth > 1);
            let unreachable = level.unreachable_walkable(level.player_start);
            assert!(
                unreachable.is_empty(),
                "depth {depth}: {} unreachable tiles",
                unreachable.len()
            );
        }
    }

    #[test]
    fn generated_level_has_stairs_down() {
        let mut rng = Rng::new(42);
        for depth in 1..=25 {
            let level = generate_level(&mut rng, depth, depth > 1);
            assert!(
                level.stairs_down.is_some(),
                "depth {depth} missing down stairs"
            );
        }
    }

    #[test]
    fn generated_level_has_upstairs_when_expected() {
        let mut rng = Rng::new(42);
        let level1 = generate_level(&mut rng, 1, false);
        assert!(level1.stairs_up.is_none());
        let level2 = generate_level(&mut rng, 2, true);
        assert!(level2.stairs_up.is_some());
    }

    #[test]
    fn generated_level_has_monsters() {
        let mut rng = Rng::new(42);
        let level = generate_level(&mut rng, 5, true);
        let monster_count = level.monsters.iter().filter(|m| m.is_some()).count();
        assert!(monster_count > 0);
    }

    #[test]
    fn generated_level_has_items() {
        let mut rng = Rng::new(42);
        let level = generate_level(&mut rng, 5, true);
        let item_count = level.items.iter().filter(|i| i.is_some()).count();
        assert!(item_count > 0);
    }

    #[test]
    fn generation_is_deterministic() {
        let mut rng1 = Rng::new(12345);
        let mut rng2 = Rng::new(12345);
        let level1 = generate_level(&mut rng1, 7, true);
        let level2 = generate_level(&mut rng2, 7, true);
        assert_eq!(level1.tiles, level2.tiles);
        assert_eq!(level1.monsters, level2.monsters);
        assert_eq!(level1.items, level2.items);
    }

    #[test]
    fn boss_present_on_boss_depth() {
        let mut rng = Rng::new(42);
        let level = generate_level(&mut rng, 5, true);
        let has_boss = level
            .monsters
            .iter()
            .any(|m| m.map(|id| id == 100).unwrap_or(false));
        assert!(has_boss, "D5 should have the Troll King boss");
    }

    #[test]
    fn player_start_is_walkable() {
        let mut rng = Rng::new(42);
        for depth in 1..=25 {
            let level = generate_level(&mut rng, depth, depth > 1);
            assert!(
                level.tile_at(level.player_start).unwrap().walkable(),
                "depth {depth}: player start not walkable"
            );
        }
    }
}
