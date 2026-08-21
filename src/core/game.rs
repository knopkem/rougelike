//! Core game state: floors, player, turn pump, victory/death checks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::core::events::GameEvent;
use crate::core::message::MessageLog;
use crate::core::rng::Rng;
use crate::entities::monster::Monster;
use crate::entities::player::Player;
use crate::items::item::Item;
use crate::map::gen;
use crate::map::level::Level;
use crate::quest::QuestLog;
use crate::status::Statuses;

pub const MAX_LEVELS: u8 = 26;

/// Number of turns in one in-game day; `game_date` maps a run's turn count to days.
pub const TURNS_PER_DAY: u64 = 30;

/// The in-game date at the given run turn: day 1 is the first turn.
pub fn game_date(turn: u64) -> String {
    format!("Day {}", turn / TURNS_PER_DAY + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    Slain,
    Starved,
    Poisoned,
    Drowned,
    Burned,
    Petrified,
    Other,
}

/// The last damage the player took: the source reported as the cause of death.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastDamage {
    pub cause: DeathCause,
    /// Identity of the damage source, when it has one (e.g. the monster's name).
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreInfo {
    pub name: String,
    pub class: String,
    pub race: String,
    pub level: u8,
    pub depth: u8,
    pub gold: u32,
    pub score: u64,
    pub kills: u32,
    pub turns: u64,
    pub seed: u64,
    pub won: bool,
    pub cause: DeathCause,
    /// Identity of the killing source, when it has one (e.g. the monster's name).
    pub killed_by: Option<String>,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub seed: u64,
    pub rng: Rng,
    pub turn: u64,
    pub player: Player,
    pub current_level: u8,
    pub levels: HashMap<u8, Level>,
    pub monsters: Vec<Monster>,
    pub quests: QuestLog,
    pub statuses: Statuses,
    pub messages: MessageLog,
    pub events: VecDeque<GameEvent>,
    pub alive: bool,
    pub won: bool,
    /// Last damage the player took, if any; the killing blow once the player dies.
    pub last_damage: Option<LastDamage>,
    pub endless: bool,
    pub amulet_carried: bool,
    pub spawn_timer: u32,
}

impl Game {
    pub fn new(seed: u64, name: &str, race: &str, class: &str) -> Self {
        let rng = Rng::new(seed);
        let player = Player::new(name, race, class);
        let mut game = Game {
            seed,
            rng,
            turn: 0,
            player,
            current_level: 1,
            levels: HashMap::new(),
            monsters: Vec::new(),
            quests: QuestLog::default(),
            statuses: Statuses::default(),
            messages: MessageLog::new(),
            events: VecDeque::new(),
            alive: true,
            won: false,
            last_damage: None,
            endless: false,
            amulet_carried: false,
            spawn_timer: 0,
        };
        game.build_level(1);
        game
    }

    pub fn new_test(name: &str, race: &str, class: &str, seed: u64) -> Self {
        let mut g = Self::new(seed, name, race, class);
        g.messages.push(
            0,
            crate::core::message::MessageKind::System,
            "Welcome to Deepdelve!",
        );
        g
    }

    /// Ensure the given depth exists (generates if needed).
    pub fn ensure_level(&mut self, depth: u8) -> &mut Level {
        if !self.levels.contains_key(&depth) {
            let level = gen::generate(depth, &mut self.rng);
            let start = level.stairs_up.map(|s| s).unwrap_or_else(|| level.center());
            let populate = depth == self.current_level;
            self.levels.insert(depth, level);
            if populate {
                self.player.pos = start;
                self.populate_monsters(depth);
            }
        }
        self.levels.get_mut(&depth).unwrap()
    }

    fn build_level(&mut self, depth: u8) {
        self.current_level = depth;
        self.ensure_level(depth);
        let start = self
            .levels
            .get(&depth)
            .unwrap()
            .stairs_up
            .map(|s| s)
            .unwrap_or_else(|| self.levels.get(&depth).unwrap().center());
        self.player.pos = start;
        self.populate_monsters(depth);
    }

    fn populate_monsters(&mut self, depth: u8) {
        let n = self.rng.int(3..8) as usize;
        let mut placed = 0;
        let mut attempts = 0;
        let max_attempts = n + 4;
        while placed < n && attempts < max_attempts {
            attempts += 1;
            if let Some(p) = crate::map::gen::random_floor_tile(self, depth, &mut self.rng.clone())
            {
                if p != self.player.pos {
                    let mut m =
                        crate::entities::monster::spawn_monster(&mut self.rng, depth, self.endless);
                    m.pos = p;
                    self.monsters.push(m);
                    placed += 1;
                }
            }
        }
    }

    pub fn current(&self) -> &Level {
        &self.levels[&self.current_level]
    }

    pub fn current_mut(&mut self) -> &mut Level {
        self.ensure_level(self.current_level)
    }

    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        self.events.drain(..).collect()
    }

    pub fn emit(&mut self, ev: GameEvent) {
        self.events.push_back(ev);
    }

    pub fn log(&mut self, kind: crate::core::message::MessageKind, text: impl Into<String>) {
        self.messages.push(self.turn, kind, text);
    }

    /// Record damage the player just took. The last call before death is the
    /// cause of death; call this from every player-damage path.
    pub fn record_damage(&mut self, cause: DeathCause, source: Option<&str>) {
        self.last_damage = Some(LastDamage {
            cause,
            source: source.map(|s| s.to_string()),
        });
    }

    pub fn do_turn(&mut self, action: crate::core::action::Action) {
        if !self.alive {
            return;
        }
        self.player_turn(action);
        self.monster_turns();
        self.tick_time();
        self.update_fov();
        self.check_death();
        self.check_victory();
        self.turn += 1;
    }

    fn player_turn(&mut self, action: crate::core::action::Action) {
        // Status lockouts: the player cannot act, but the turn still passes.
        if self.statuses.is_paralyzed() {
            self.log(crate::core::message::MessageKind::Bad, "You are paralyzed!");
            return;
        }
        if self.statuses.is_petrified() {
            self.log(
                crate::core::message::MessageKind::Bad,
                "You are turned to stone!",
            );
            return;
        }
        if self.statuses.is_asleep() {
            self.log(
                crate::core::message::MessageKind::Bad,
                "You are fast asleep.",
            );
            return;
        }
        if self.statuses.is_slowed() && self.turn % 2 == 1 {
            self.log(
                crate::core::message::MessageKind::Normal,
                "You are too slow to act.",
            );
            return;
        }
        let action = self.derange_if_confused(action);
        match action {
            crate::core::action::Action::Move(dx, dy) => {
                self.try_move(dx, dy);
            }
            crate::core::action::Action::Wait => {}
            crate::core::action::Action::StairsDown => {
                self.try_stairs_down();
            }
            crate::core::action::Action::StairsUp => {
                self.try_stairs_up();
            }
            crate::core::action::Action::Pickup => {
                self.pickup_at(self.player.pos);
            }
            crate::core::action::Action::UseItem(s) => {
                match self.player.inventory.get(s).map(|i| &i.kind) {
                    Some(crate::items::item::ItemKind::Food(_)) => self.eat_item(s),
                    Some(crate::items::item::ItemKind::Potion(_)) => self.quaff_item(s),
                    Some(crate::items::item::ItemKind::Scroll(_)) => self.read_scroll(s),
                    Some(crate::items::item::ItemKind::Wand(_)) => {}
                    Some(crate::items::item::ItemKind::Weapon(_)) => self.wield_item(s),
                    Some(crate::items::item::ItemKind::Armor(_)) => self.wear_item(s),
                    Some(crate::items::item::ItemKind::Ring(_)) => self.ring_on(s),
                    _ => {}
                }
            }
            crate::core::action::Action::Drop(s) => self.drop_item(s),
            crate::core::action::Action::Wield(s) => self.wield_item(s),
            crate::core::action::Action::Wear(s) => self.wear_item(s),
            crate::core::action::Action::TakeOff(s) => self.take_off_item(s),
            crate::core::action::Action::RingOn(s) => self.ring_on(s),
            crate::core::action::Action::RingOff(s) => self.ring_off(s),
            crate::core::action::Action::Eat(s) => self.eat_item(s),
            crate::core::action::Action::Quaff(s) => self.quaff_item(s),
            crate::core::action::Action::Read(s) => self.read_scroll(s),
            crate::core::action::Action::FireWand(s, tx, ty) => self.fire_wand(s, tx, ty),
            crate::core::action::Action::Identify(s) => self.identify_item(s),
            _ => {}
        }
    }

    /// While confused, a move may be deranged into a random direction.
    /// Bump-moves (the target tile holds a monster) are never deranged.
    fn derange_if_confused(
        &mut self,
        action: crate::core::action::Action,
    ) -> crate::core::action::Action {
        let crate::core::action::Action::Move(dx, dy) = action else {
            return action;
        };
        if !self.statuses.is_confused() {
            return action;
        }
        let target = (
            (self.player.pos.0 as i32 + dx) as u8,
            (self.player.pos.1 as i32 + dy) as u8,
        );
        if self.monsters.iter().any(|m| m.pos == target && !m.dead) {
            return action;
        }
        if !self.rng.chance(50) {
            return action;
        }
        if self.statuses.is_blessed() && self.rng.chance(50) {
            self.log(
                crate::core::message::MessageKind::Good,
                "A blessing protects you.",
            );
            return action;
        }
        self.log(
            crate::core::message::MessageKind::Bad,
            "Your vision blurs; you stumble in a random direction.",
        );
        let dirs: [(i32, i32); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        let (ndx, ndy) = dirs[self.rng.int(0..8) as usize];
        crate::core::action::Action::Move(ndx, ndy)
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let (px, py) = (self.player.pos.0, self.player.pos.1);
        let nx = (px as i32 + dx) as u8;
        let ny = (py as i32 + dy) as u8;
        let np = (nx, ny);

        if let Some(m_idx) = self.monsters.iter().position(|m| m.pos == np && !m.dead) {
            self.attack_monster(m_idx);
            return true;
        }
        // Doors block movement; bumping a door opens it (or, when locked,
        // unlocks it with an iron key from the inventory).
        match self.current().tile_at(np) {
            crate::map::level::Tile::DoorClosed => {
                self.open_door(np);
                return true;
            }
            crate::map::level::Tile::DoorLocked => {
                self.unlock_door(np);
                return true;
            }
            _ => {}
        }
        if !self.current().is_walkable(np) {
            return true;
        }
        self.player.pos = np;
        self.emit(GameEvent::Footstep);
        let teleported = self.apply_tile_effects(np);
        // A teleport trap moved the player: don't auto-collect at the
        // tile they left.
        if !teleported {
            if let Some(gold) = self.current_mut().take_gold_at(np) {
                self.player.gold += gold;
                self.emit(GameEvent::Coin);
                self.log(
                    crate::core::message::MessageKind::Normal,
                    format!("You pick up {gold} gold."),
                );
            }
        }
        {
            let mut quests = self.quests.clone();
            quests.check_progress(self);
            self.quests = quests;
        }
        true
    }

    /// Effects of the tile the player just stepped onto. Traps trigger on
    /// entry and disarm; water slows, lava burns, spore gas poisons.
    fn apply_tile_effects(&mut self, pos: (u8, u8)) -> bool {
        if let Some(trap) = self.current().trap_at(pos) {
            self.trigger_trap(pos, trap);
            // A teleport trap may have moved the player.
            return self.player.pos != pos;
        }
        match self.current().tile_at(pos) {
            crate::map::level::Tile::Water => {
                self.statuses.slow = self.statuses.slow.max(2);
                self.log(
                    crate::core::message::MessageKind::Normal,
                    "The deep water slows your movements.",
                );
            }
            crate::map::level::Tile::Lava => {
                let dmg = 2 + self.rng.int(0..3) as u8;
                self.player.hp = self.player.hp.saturating_sub(dmg);
                if dmg > 0 {
                    self.record_damage(DeathCause::Burned, None);
                }
                self.log(
                    crate::core::message::MessageKind::Bad,
                    format!("The lava sears you for {dmg}!"),
                );
            }
            crate::map::level::Tile::SporeGas if self.rng.chance(25) => {
                self.statuses.poison = 5;
                self.log(
                    crate::core::message::MessageKind::Bad,
                    "Spore gas billows into your lungs!",
                );
            }
            _ => {}
        }
        false
    }

    /// Bump-to-open: the closed door swings open, costing the turn.
    fn open_door(&mut self, pos: (u8, u8)) {
        self.current_mut()
            .set_tile(pos, crate::map::level::Tile::Floor);
        self.emit(GameEvent::Door {
            opened: true,
            locked: false,
        });
        self.log(
            crate::core::message::MessageKind::Normal,
            "You open the door.",
        );
    }

    /// A locked door needs an iron key; the key is consumed and the door
    /// swings open, costing the turn.
    fn unlock_door(&mut self, pos: (u8, u8)) {
        if !self.player_has_key() {
            self.log(
                crate::core::message::MessageKind::Normal,
                "The door is locked. You need a key.",
            );
            return;
        }
        self.consume_key();
        self.current_mut()
            .set_tile(pos, crate::map::level::Tile::Floor);
        self.emit(GameEvent::Door {
            opened: true,
            locked: true,
        });
        self.log(
            crate::core::message::MessageKind::Normal,
            "You use the iron key to unlock the door.",
        );
    }

    pub(crate) fn player_has_key(&self) -> bool {
        self.player
            .inventory
            .iter()
            .any(|i| i.kind == crate::items::item::ItemKind::Key)
    }

    pub(crate) fn consume_key(&mut self) {
        if let Some(slot) = self
            .player
            .inventory
            .iter()
            .position(|i| i.kind == crate::items::item::ItemKind::Key)
        {
            self.player.inventory.remove(slot);
        }
    }

    /// Trigger the trap under the player. The trap disarms after firing.
    fn trigger_trap(&mut self, pos: (u8, u8), kind: crate::map::level::TrapKind) {
        self.current_mut()
            .set_tile(pos, crate::map::level::Tile::Floor);
        self.emit(GameEvent::Trap);
        match kind {
            crate::map::level::TrapKind::Arrow => {
                let dmg = 2 + self.rng.int(0..4) as u8;
                self.player.hp = self.player.hp.saturating_sub(dmg);
                if dmg > 0 {
                    self.record_damage(DeathCause::Other, None);
                }
                self.log(
                    crate::core::message::MessageKind::Bad,
                    format!("A trap hurls an arrow into you for {dmg}!"),
                );
            }
            crate::map::level::TrapKind::Dart => {
                self.player.hp = self.player.hp.saturating_sub(1);
                self.record_damage(DeathCause::Poisoned, None);
                self.statuses.poison = self.statuses.poison.max(3);
                self.log(
                    crate::core::message::MessageKind::Bad,
                    "A dart stings you! The tip is poisoned.",
                );
            }
            crate::map::level::TrapKind::FallingItem => {
                if !self.player.inventory.is_empty() {
                    let slot = self.rng.int(0..self.player.inventory.len() as u64) as usize;
                    let item = self.player.inventory.remove(slot);
                    let name = item.name();
                    self.current_mut().add_item(pos, item);
                    self.emit(GameEvent::Drop);
                    self.log(
                        crate::core::message::MessageKind::Bad,
                        format!("You fumble and drop the {name}!"),
                    );
                } else {
                    self.log(
                        crate::core::message::MessageKind::Normal,
                        "Your hands scramble, but you carry nothing to lose.",
                    );
                }
            }
            crate::map::level::TrapKind::Teleport => {
                let level = self.current();
                let candidates: Vec<(u8, u8)> = level
                    .floor_tiles()
                    .into_iter()
                    .filter(|p| *p != pos)
                    .filter(|p| !self.monsters.iter().any(|m| m.pos == *p && !m.dead))
                    .collect();
                match self.rng.pick(&candidates) {
                    Some(dst) => {
                        self.player.pos = dst;
                        self.emit(GameEvent::Teleport);
                        self.log(
                            crate::core::message::MessageKind::Bad,
                            "The floor gives way; you are teleported somewhere!",
                        );
                    }
                    None => {
                        self.log(
                            crate::core::message::MessageKind::Normal,
                            "The trap whirs, but nothing happens.",
                        );
                    }
                }
            }
            crate::map::level::TrapKind::SleepGas => {
                self.statuses.sleep = self.statuses.sleep.max(3);
                self.log(
                    crate::core::message::MessageKind::Bad,
                    "A sleep gas billows over you!",
                );
            }
            crate::map::level::TrapKind::AcidPool => {
                let dmg = 2 + self.rng.int(0..3) as u8;
                self.player.hp = self.player.hp.saturating_sub(dmg);
                if dmg > 0 {
                    self.record_damage(DeathCause::Other, None);
                }
                self.log(
                    crate::core::message::MessageKind::Bad,
                    format!("You plunge into an acid pool; it burns for {dmg}!"),
                );
            }
        }
    }

    fn attack_monster(&mut self, idx: usize) {
        let combat_rng = self.rng.clone();
        let mut combat = crate::combat::Combat::new(combat_rng);
        let m_name = self.monsters[idx].name.clone();
        let penalty = self.statuses.hunger_to_hit_penalty(self.player.hunger);
        let res = combat.player_attacks(&self.player, &self.monsters[idx], penalty);
        if res.hit {
            self.emit(GameEvent::Hit { crit: res.crit });
            let dmg = res.damage;
            self.monsters[idx].hp = self.monsters[idx].hp.saturating_sub(dmg);
            if res.crit {
                self.log(
                    crate::core::message::MessageKind::Combat,
                    format!("Critical! You hit {m_name} for {dmg}!"),
                );
            } else {
                self.log(
                    crate::core::message::MessageKind::Combat,
                    format!("You hit {m_name} for {dmg}."),
                );
            }
            if self.monsters[idx].hp == 0 {
                self.kill_monster(idx, "slain");
            }
        } else {
            self.emit(GameEvent::Miss);
            self.log(
                crate::core::message::MessageKind::Combat,
                format!("You miss {m_name}."),
            );
        }
        if !self.monsters.is_empty() && idx < self.monsters.len() && self.monsters[idx].hp > 0 {
            let res2 = combat.monster_attacks(&self.monsters[idx], &self.player);
            if res2.hit {
                let dmg = res2.damage;
                let attacker = self.monsters[idx].name.clone();
                self.player.hp = self.player.hp.saturating_sub(dmg);
                if dmg > 0 {
                    self.record_damage(DeathCause::Slain, Some(&attacker));
                }
                self.emit(GameEvent::Hit { crit: false });
                self.log(
                    crate::core::message::MessageKind::Combat,
                    format!("{m_name} hits you for {dmg}."),
                );
            }
        }
    }

    fn kill_monster(&mut self, idx: usize, how: &str) {
        let m = self.monsters[idx].clone();
        let tier = m.tier();
        let name = m.name.clone();
        self.player.kills += 1;
        let xp = m.xp;
        let leveled_up = self.player.gain_xp(xp);
        self.log(
            crate::core::message::MessageKind::Good,
            format!("{name} is {how} (+{xp} XP)"),
        );
        if leveled_up {
            self.log(
                crate::core::message::MessageKind::Good,
                format!("You are now level {}!", self.player.level),
            );
            self.emit(GameEvent::LevelUp);
        }
        self.emit(GameEvent::MonsterDeath { tier });
        if crate::items::loot::maybe_drop(&mut self.rng, tier) {
            let drop = crate::items::loot::roll_drop(&mut self.rng, self.current_level);
            let pos = m.pos;
            self.current_mut().add_item(pos, drop);
            self.log(
                crate::core::message::MessageKind::Normal,
                format!("The {name} drops something."),
            );
        }
        {
            let mut quests = self.quests.clone();
            quests.on_kill(self, &m.def);
            self.quests = quests;
        }
        self.monsters.remove(idx);
    }

    fn try_stairs_down(&mut self) -> bool {
        if !self.current().stairs_down_at(self.player.pos) {
            self.log(
                crate::core::message::MessageKind::Normal,
                "No down-stairs here.",
            );
            return false;
        }
        // Endless mode lifts the max-depth cap; the u8 cap still bounds it.
        let Some(next) = self.current_level.checked_add(1) else {
            return false;
        };
        if !self.endless && next > MAX_LEVELS {
            return false;
        }
        self.current_level = next;
        self.ensure_level(next);
        self.emit(GameEvent::Stairs);
        self.log(
            crate::core::message::MessageKind::System,
            format!("You descend to depth {}.", next),
        );
        crate::save::autosave(self);
        true
    }

    fn try_stairs_up(&mut self) -> bool {
        if !self.current().stairs_up_at(self.player.pos) {
            self.log(
                crate::core::message::MessageKind::Normal,
                "No up-stairs here.",
            );
            return false;
        }
        if self.current_level == 1 {
            return false;
        }
        let prev = self.current_level - 1;
        self.current_level = prev;
        self.ensure_level(prev);
        self.emit(GameEvent::Stairs);
        self.log(
            crate::core::message::MessageKind::System,
            format!("You climb to depth {}.", prev),
        );
        true
    }

    fn pickup_at(&mut self, pos: (u8, u8)) {
        if let Some(item) = self.current_mut().take_item_at(pos) {
            let amulet = item.kind == crate::items::item::ItemKind::Amulet;
            let name = item.name().to_string();
            self.player.inventory.push(item);
            self.emit(GameEvent::Pickup);
            self.log(
                crate::core::message::MessageKind::Normal,
                format!("You pick up a {}.", name),
            );
            if amulet {
                self.amulet_carried = true;
            }
        } else if let Some(gold) = self.current_mut().take_gold_at(pos) {
            self.player.gold += gold;
            self.emit(GameEvent::Coin);
            self.log(
                crate::core::message::MessageKind::Normal,
                format!("You pick up {gold} gold."),
            );
        }
    }

    fn drop_item(&mut self, slot: usize) {
        if slot >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.remove(slot);
        let name = item.name().to_string();
        let pos = self.player.pos;
        self.current_mut().add_item(pos, item);
        self.emit(GameEvent::Drop);
        self.log(
            crate::core::message::MessageKind::Normal,
            format!("You drop the {name}."),
        );
    }

    fn wield_item(&mut self, slot: usize) {
        if slot >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.get(slot).cloned();
        if let Some(item) = item {
            if !matches!(
                item.kind,
                crate::items::item::ItemKind::Weapon(_) | crate::items::item::ItemKind::Shield(_)
            ) {
                return;
            }
            let was = self.player.wielded.take();
            if let Some(w) = was {
                self.player.inventory.push(w);
            }
            self.player.inventory.remove(slot);
            self.player.wielded = Some(item);
            self.emit(GameEvent::Equip);
        }
    }

    fn wear_item(&mut self, slot: usize) {
        if slot >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.get(slot).cloned();
        if let Some(item) = item {
            if !matches!(item.kind, crate::items::item::ItemKind::Armor(_)) {
                return;
            }
            self.player.inventory.remove(slot);
            let was = self.player.armor.take();
            if let Some(a) = was {
                self.player.inventory.push(a);
            }
            self.player.armor = Some(item);
            self.emit(GameEvent::Equip);
        }
    }

    /// Take off the equipment at the given *equipment slot*: 0 = wielded
    /// weapon/shield, 1 = armor, 2+ = rings (2 = first ring, 3 = second, ...).
    /// The removed item goes back into the inventory; an empty slot logs a
    /// message instead. Matches the `Action::TakeOff(slot)` argument.
    fn take_off_item(&mut self, slot: usize) {
        match slot {
            0 => match self.player.wielded.take() {
                Some(w) => self.player.inventory.push(w),
                None => self.log(
                    crate::core::message::MessageKind::Normal,
                    "You aren't wielding anything.",
                ),
            },
            1 => match self.player.armor.take() {
                Some(a) => self.player.inventory.push(a),
                None => self.log(
                    crate::core::message::MessageKind::Normal,
                    "You aren't wearing any armor.",
                ),
            },
            n => {
                let ring = n - 2;
                if ring < self.player.rings.len() {
                    let r = self.player.rings.remove(ring);
                    self.player.inventory.push(r);
                } else {
                    self.log(
                        crate::core::message::MessageKind::Normal,
                        "You aren't wearing a ring.",
                    );
                }
            }
        }
    }

    fn ring_on(&mut self, slot: usize) {
        if slot >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.get(slot).cloned();
        if let Some(item) = item {
            if !matches!(item.kind, crate::items::item::ItemKind::Ring(_)) {
                return;
            }
            self.player.inventory.remove(slot);
            self.player.rings.push(item);
            self.emit(GameEvent::Equip);
        }
    }

    fn ring_off(&mut self, slot: usize) {
        if slot >= self.player.rings.len() {
            return;
        }
        let ring = self.player.rings.remove(slot);
        self.player.inventory.push(ring);
    }

    fn eat_item(&mut self, slot: usize) {
        if slot >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.get(slot).cloned();
        if let Some(item) = item {
            if !matches!(item.kind, crate::items::item::ItemKind::Food(_)) {
                return;
            }
            self.player.inventory.remove(slot);
            self.player.hunger = (self.player.hunger + item.food_value() as u16).min(1200);
            self.emit(GameEvent::Eat);
            self.log(
                crate::core::message::MessageKind::Normal,
                format!("You eat the {}.", item.name()),
            );
            // Wild mushrooms are a gamble.
            if matches!(
                item.kind,
                crate::items::item::ItemKind::Food(crate::items::item::FoodKind::Mushroom)
            ) && self.rng.chance(50)
            {
                self.statuses.poison = 5;
                self.log(
                    crate::core::message::MessageKind::Bad,
                    "The mushroom was bad! You feel poisoned.",
                );
            }
        }
    }

    fn quaff_item(&mut self, slot: usize) {
        if slot >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.get(slot).cloned();
        if let Some(item) = item {
            if !matches!(item.kind, crate::items::item::ItemKind::Potion(_)) {
                return;
            }
            self.apply_potion(&item);
            self.player.inventory.remove(slot);
            self.emit(GameEvent::Quaff);
            self.emit(GameEvent::PotionSplash);
        }
    }

    fn read_scroll(&mut self, slot: usize) {
        if slot >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.get(slot).cloned();
        if let Some(item) = item {
            if !matches!(item.kind, crate::items::item::ItemKind::Scroll(_)) {
                return;
            }
            self.apply_scroll(&item);
            self.player.inventory.remove(slot);
            self.emit(GameEvent::ScrollRead);
        }
    }

    fn fire_wand(&mut self, slot: usize, tx: u8, ty: u8) {
        if slot >= self.player.inventory.len() {
            return;
        }
        let item = self.player.inventory.get(slot).cloned();
        if let Some(item) = item {
            if !matches!(item.kind, crate::items::item::ItemKind::Wand(_)) {
                return;
            }
            if self.player.ep < item.ep_cost() {
                self.log(
                    crate::core::message::MessageKind::Normal,
                    "You are too tired to use that.",
                );
                return;
            }
            self.player.ep = self.player.ep.saturating_sub(item.ep_cost());
            let mut rng = self.rng.clone();
            crate::magic::cast_wand(&mut rng, self, &item, (tx, ty));
            self.rng = rng;
            self.emit(GameEvent::WandCast {
                kind: item.name().to_string(),
            });
        }
    }

    pub fn identify_item(&mut self, slot: usize) {
        if slot < self.player.inventory.len() {
            self.player.inventory[slot].identified = true;
        }
    }

    fn apply_potion(&mut self, item: &Item) {
        if let crate::items::item::ItemKind::Potion(kind) = item.kind {
            crate::magic::apply_potion(self, kind);
        }
    }

    fn apply_scroll(&mut self, item: &Item) {
        if let crate::items::item::ItemKind::Scroll(kind) = item.kind {
            crate::magic::apply_scroll(self, kind);
        }
    }

    fn monster_turns(&mut self) {
        let player_invisible = self.statuses.is_invisible();
        let mut i = 0;
        while i < self.monsters.len() {
            let m = self.monsters[i].clone();
            let mut rng = self.rng.clone();
            let decision = {
                let mut ai_game = crate::entities::ai::AiGame::new(
                    self.current(),
                    self.player.pos,
                    &self.monsters,
                    player_invisible,
                );
                crate::entities::ai::act(&mut rng, &mut ai_game, &m)
            };
            self.rng = rng;
            if let Some(decision) = decision {
                match decision {
                    crate::entities::ai::AiDecision::MoveTo(p) => {
                        if self.current().is_walkable(p)
                            && self.player.pos != p
                            && !self.monsters.iter().any(|o| o.pos == p && !o.dead)
                        {
                            self.monsters[i].pos = p;
                        }
                    }
                    crate::entities::ai::AiDecision::AttackPlayer => {
                        let m2 = self.monsters[i].clone();
                        let mut combat = crate::combat::Combat::new(self.rng.clone());
                        let dx = (m2.pos.0 as i32 - self.player.pos.0 as i32).abs();
                        let dy = (m2.pos.1 as i32 - self.player.pos.1 as i32).abs();
                        if dx + dy <= 1 {
                            let res = combat.monster_attacks(&m2, &self.player);
                            if res.hit {
                                let dmg = res.damage;
                                let attacker = m2.name.clone();
                                self.player.hp = self.player.hp.saturating_sub(dmg);
                                if dmg > 0 {
                                    self.record_damage(DeathCause::Slain, Some(&attacker));
                                }
                                self.log(
                                    crate::core::message::MessageKind::Combat,
                                    format!("{} hits you for {dmg}.", self.monsters[i].name),
                                );
                                self.emit(GameEvent::Hit { crit: false });
                            }
                        }
                    }
                    crate::entities::ai::AiDecision::Wait => {}
                }
            }
            i += 1;
        }
    }

    fn tick_time(&mut self) {
        let has_sustenance = self.player.rings.iter().any(|r| {
            matches!(
                r.kind,
                crate::items::item::ItemKind::Ring(crate::items::item::RingKind::Sustenance)
            )
        });
        if !has_sustenance {
            self.player.hunger = self.player.hunger.saturating_sub(1);
        }
        let alive = self.player.hp > 0;
        let death_cause = {
            let mut statuses = self.statuses.clone();
            let cause = statuses.tick(&mut self.player);
            self.statuses = statuses;
            cause
        };
        if alive {
            // The status tick reports the last damage it dealt; that is the
            // cause of death if HP hit 0 this turn.
            if let Some(cause) = death_cause {
                self.record_damage(cause, None);
            }
        }
        if self.player.hunger > 400 {
            self.player.ep = (self.player.ep + 1).min(self.player.max_ep);
        }
        self.spawn_timer += 1;
        if self.spawn_timer >= 50 {
            self.spawn_timer = 0;
            self.maybe_spawn();
        }
    }

    fn maybe_spawn(&mut self) {
        if self.current_level > 25 && !self.endless {
            return;
        }
        if !self.rng.chance(30) {
            return;
        }
        if let Some(p) =
            crate::map::gen::random_floor_tile(self, self.current_level, &mut self.rng.clone())
        {
            if p != self.player.pos {
                let mut m = crate::entities::monster::spawn_monster(
                    &mut self.rng,
                    self.current_level,
                    self.endless,
                );
                m.pos = p;
                self.monsters.push(m);
            }
        }
    }

    fn update_fov(&mut self) {
        let lvl = self.current_level;
        let player_pos = self.player.pos;
        let radius = self.player.darkvision_radius();
        let level = self.ensure_level(lvl);
        crate::map::fov::compute_fov(level, player_pos, radius);
    }

    fn check_death(&mut self) {
        if self.player.hp > 0 {
            return;
        }
        self.alive = false;
        self.emit(GameEvent::PlayerDeath);
        self.log(
            crate::core::message::MessageKind::Bad,
            "You die. The dungeon claims another.",
        );
        crate::save::delete_save(self);
        crate::hiscore::record(self);
    }

    fn check_victory(&mut self) {
        if self.alive && !self.won && self.amulet_carried && self.current_level >= 25 {
            self.won = true;
            self.emit(GameEvent::Victory);
            self.log(
                crate::core::message::MessageKind::Good,
                "You raise the Amulet of the Abyss! VICTORY!",
            );
            crate::hiscore::record(self);
            crate::save::delete_save(self);
        }
    }

    pub fn score_info(&self) -> ScoreInfo {
        let score = crate::core::score::compute(self);
        let (cause, killed_by) = if self.won {
            (DeathCause::Other, None)
        } else {
            let last = self.last_damage.clone();
            (
                last.as_ref().map(|d| d.cause).unwrap_or(DeathCause::Other),
                last.and_then(|d| d.source),
            )
        };
        ScoreInfo {
            name: self.player.name.clone(),
            class: self.player.class.to_string(),
            race: self.player.race.to_string(),
            level: self.player.level,
            depth: self.current_level,
            gold: self.player.gold,
            score,
            kills: self.player.kills,
            turns: self.turn,
            seed: self.seed,
            won: self.won,
            cause,
            killed_by,
            date: game_date(self.turn),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_with_kill() -> Game {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let mut m = crate::entities::monster::Monster::new(
            crate::data::monsters::MONSTERS[0].clone(),
            (5, 5),
        );
        m.xp = 5;
        g.monsters.push(m);
        g
    }

    #[test]
    fn kill_crossing_threshold_levels_up() {
        let mut g = game_with_kill();
        let (max_hp_before, max_ep_before) = (g.player.max_hp, g.player.max_ep);
        g.player.xp = 99;
        g.player.hp = 10;
        g.kill_monster(0, "slain");
        assert_eq!(g.player.level, 2);
        assert_eq!(g.player.xp, 4);
        assert_eq!(g.player.max_hp, max_hp_before + 2);
        assert_eq!(g.player.max_ep, max_ep_before + 2);
        assert_eq!(g.player.hp, g.player.max_hp);
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text == "You are now level 2!"));
        assert!(g.events.iter().any(|e| matches!(e, GameEvent::LevelUp)));
    }

    fn game_equipped() -> Game {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        g.player.inventory.clear();
        g.player.wielded = Some(crate::items::catalog::make_weapon(
            crate::items::item::WeaponKind::Dagger,
            0,
            false,
        ));
        g.player.armor = Some(crate::items::catalog::make_armor(
            crate::items::item::ArmorKind::Chainmail,
            0,
            false,
        ));
        g.player.rings.push(crate::items::catalog::make_ring(
            crate::items::item::RingKind::Protection,
        ));
        g.player.rings.push(crate::items::catalog::make_ring(
            crate::items::item::RingKind::Energy,
        ));
        g
    }

    #[test]
    fn take_off_worn_armor_moves_it_to_inventory() {
        let mut g = game_equipped();
        let armor = g.player.armor.clone().unwrap();
        g.do_turn(crate::core::action::Action::TakeOff(1));
        assert!(g.player.armor.is_none());
        assert!(g.player.inventory.contains(&armor));
    }

    #[test]
    fn take_off_worn_ring_removes_that_ring_not_the_armor() {
        let mut g = game_equipped();
        let armor = g.player.armor.clone().unwrap();
        let first = g.player.rings[0].clone();
        g.do_turn(crate::core::action::Action::TakeOff(2));
        assert!(
            g.player
                .armor
                .as_ref()
                .map(|a| a == &armor)
                .unwrap_or(false),
            "armor untouched"
        );
        assert_eq!(g.player.rings.len(), 1);
        assert_eq!(
            g.player.rings[0].kind,
            crate::items::item::ItemKind::Ring(crate::items::item::RingKind::Energy)
        );
        assert!(g.player.inventory.contains(&first));
    }

    #[test]
    fn take_off_wielded_weapon_moves_it_to_inventory() {
        let mut g = game_equipped();
        let weapon = g.player.wielded.clone().unwrap();
        g.do_turn(crate::core::action::Action::TakeOff(0));
        assert!(g.player.wielded.is_none());
        assert!(g.player.inventory.contains(&weapon));
    }

    #[test]
    fn take_off_empty_equipment_slot_logs_and_does_nothing() {
        let mut g = game_equipped();
        g.player.armor = None;
        g.player.rings.clear();
        let inventory = g.player.inventory.clone();
        let messages = g.messages.all().len();
        g.do_turn(crate::core::action::Action::TakeOff(1));
        g.do_turn(crate::core::action::Action::TakeOff(2));
        g.do_turn(crate::core::action::Action::TakeOff(99));
        assert_eq!(g.player.inventory, inventory);
        let all = g.messages.all();
        assert!(all.len() >= messages + 3);
        assert!(all
            .iter()
            .any(|m| m.text == "You aren't wearing any armor."));
        assert!(all.iter().any(|m| m.text == "You aren't wearing a ring."));
    }

    #[test]
    fn monster_kill_reports_slain_cause_and_killer_name() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let mut m = crate::entities::monster::Monster::new(
            crate::data::monsters::MONSTERS[0].clone(),
            (5, 5),
        );
        let m_name = m.name.clone();
        m.hp = 255;
        m.max_hp = 255;
        m.def.attack = 200; // to-hit 250: always hits
        m.def.damage_die = 20;
        g.monsters.push(m);
        g.player.hp = 1;
        for _ in 0..200 {
            if g.player.hp == 0 {
                break;
            }
            g.monsters[0].hp = 255;
            g.attack_monster(0);
        }
        assert_eq!(g.player.hp, 0, "the monster should have killed the player");
        let info = g.score_info();
        assert_eq!(info.cause, DeathCause::Slain);
        assert_eq!(info.killed_by, Some(m_name));
    }

    #[test]
    fn starving_death_reports_starved_cause() {
        crate::tests_harness::with_isolated_data_dir("starved_death", || {
            let mut g = Game::new_test("Test", "Human", "Warrior", 42);
            g.monsters.clear();
            g.player.hp = 1;
            g.player.hunger = 0;
            g.do_turn(crate::core::action::Action::Wait);
            assert!(!g.alive);
            let info = g.score_info();
            assert_eq!(info.cause, DeathCause::Starved);
            assert_eq!(info.killed_by, None);
            assert_eq!(info.date, "Day 1");
        });
    }

    #[test]
    fn poison_death_reports_poisoned_cause() {
        crate::tests_harness::with_isolated_data_dir("poison_death", || {
            let mut g = Game::new_test("Test", "Human", "Warrior", 42);
            g.monsters.clear();
            g.player.hp = 1;
            g.player.hunger = 300; // hungry (not well-fed) so no regen outpaces poison
            g.statuses.poison = 1;
            g.do_turn(crate::core::action::Action::Wait);
            assert!(!g.alive);
            let info = g.score_info();
            assert_eq!(info.cause, DeathCause::Poisoned);
            assert_eq!(info.killed_by, None);
        });
    }

    #[test]
    fn score_date_is_derived_from_turns() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.turn = 0;
        assert_eq!(g.score_info().date, "Day 1");
        g.turn = 30;
        assert_eq!(g.score_info().date, "Day 2");
        g.turn = 299;
        assert_eq!(g.score_info().date, "Day 10");
    }

    #[test]
    fn kill_without_levelup_only_adds_xp() {
        let mut g = game_with_kill();
        let (max_hp_before, max_ep_before) = (g.player.max_hp, g.player.max_ep);
        g.kill_monster(0, "slain");
        assert_eq!(g.player.level, 1);
        assert_eq!(g.player.xp, 5);
        assert_eq!(g.player.max_hp, max_hp_before);
        assert_eq!(g.player.max_ep, max_ep_before);
        assert!(!g
            .messages
            .all()
            .iter()
            .any(|m| m.text.starts_with("You are now level")));
        assert!(g
            .drain_events()
            .iter()
            .all(|e| !matches!(e, GameEvent::LevelUp)));
    }

    fn walkable_dir(g: &Game) -> (i32, i32) {
        let (px, py) = g.player.pos;
        for (dx, dy) in [(1i32, 0), (0, 1), (-1, 0), (0, -1)] {
            let nx = px as i32 + dx;
            let ny = py as i32 + dy;
            if crate::map::level::Level::in_bounds(nx as u8, ny as u8)
                && g.current().is_walkable((nx as u8, ny as u8))
            {
                return (dx, dy);
            }
        }
        panic!("no walkable tile adjacent to the player");
    }

    #[test]
    fn paralyzed_player_cannot_act() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        g.statuses.paralysis = 5;
        let start = g.player.pos;
        let before = g.turn;
        g.do_turn(crate::core::action::Action::Move(1, 0));
        assert_eq!(g.player.pos, start, "paralysis must block movement");
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text == "You are paralyzed!"));
        assert_eq!(g.turn, before + 1, "the turn still passes");
        assert!(g.alive);
    }

    #[test]
    fn petrified_and_asleep_players_cannot_act() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let start = g.player.pos;
        g.statuses.petrification = 3;
        g.do_turn(crate::core::action::Action::Move(1, 0));
        assert_eq!(g.player.pos, start);
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text == "You are turned to stone!"));

        let mut g2 = Game::new_test("Test", "Human", "Warrior", 7);
        g2.monsters.clear();
        let start2 = g2.player.pos;
        g2.statuses.sleep = 3;
        g2.do_turn(crate::core::action::Action::Pickup);
        assert_eq!(g2.player.pos, start2);
        assert!(g2
            .messages
            .all()
            .iter()
            .any(|m| m.text == "You are fast asleep."));
    }

    #[test]
    fn slowed_player_skips_every_other_turn() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        g.statuses.slow = 10;
        let (dx, dy) = walkable_dir(&g);
        let start = g.player.pos;
        g.turn = 1; // odd turn: skipped
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert_eq!(g.player.pos, start);
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text == "You are too slow to act."));
        let (dx2, dy2) = walkable_dir(&g);
        g.turn = 2; // even turn: the move goes through
        g.do_turn(crate::core::action::Action::Move(dx2, dy2));
        assert_ne!(g.player.pos, start, "the move must land on the even turn");
    }

    #[test]
    fn confusion_deranges_open_moves_but_not_bumps_or_waits() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        g.statuses.confusion = 8;
        // Non-move actions are never deranged.
        assert_eq!(
            g.derange_if_confused(crate::core::action::Action::Wait),
            crate::core::action::Action::Wait
        );
        // Bump-moves (a monster on the target tile) are never deranged.
        let (px, py) = g.player.pos;
        let bump = (px + 1, py);
        let m = crate::entities::monster::Monster::new(
            crate::data::monsters::MONSTERS[0].clone(),
            bump,
        );
        g.monsters.push(m);
        assert_eq!(
            g.derange_if_confused(crate::core::action::Action::Move(1, 0)),
            crate::core::action::Action::Move(1, 0)
        );
        g.monsters.clear();
        // Open moves stay moves but are redirected: over enough rolls the
        // direction must change at least once.
        let mut deranged = false;
        for _ in 0..100 {
            if let crate::core::action::Action::Move(ndx, ndy) =
                g.derange_if_confused(crate::core::action::Action::Move(1, 0))
            {
                if (ndx, ndy) != (1, 0) {
                    deranged = true;
                    break;
                }
            } else {
                panic!("confusion must never block a move, only redirect it");
            }
        }
        assert!(deranged, "a confused player's moves must sometimes derange");
    }

    #[test]
    fn spore_gas_tile_can_poison_the_player() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut()
            .set_tile(np, crate::map::level::Tile::SporeGas);
        for _ in 0..200 {
            if g.statuses.poison > 0 {
                break;
            }
            g.try_move(dx, dy);
            g.try_move(-dx, -dy);
        }
        assert!(
            g.statuses.poison > 0,
            "stepping into spore gas must sometimes poison the player"
        );
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text == "Spore gas billows into your lungs!"));
    }

    #[test]
    fn bumping_closed_door_opens_it_and_costs_a_turn() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut()
            .set_tile(np, crate::map::level::Tile::DoorClosed);
        let turn_before = g.turn;
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert_eq!(g.turn, turn_before + 1, "bumping a door must cost a turn");
        assert_eq!(
            g.player.pos,
            (px, py),
            "the player must not walk through a door"
        );
        assert_eq!(
            g.current().tile_at(np),
            crate::map::level::Tile::Floor,
            "the door must open"
        );
        assert!(g.drain_events().iter().any(|e| matches!(
            e,
            GameEvent::Door {
                opened: true,
                locked: false
            }
        )));
    }

    #[test]
    fn locked_door_opens_only_with_a_key() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut()
            .set_tile(np, crate::map::level::Tile::DoorLocked);
        // No key: the door stays locked.
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert_eq!(
            g.current().tile_at(np),
            crate::map::level::Tile::DoorLocked,
            "a locked door must not open without a key"
        );
        assert_eq!(g.player.pos, (px, py));
        // With a key: the key is consumed and the door opens.
        g.player.inventory.push(crate::items::catalog::make_key());
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert_eq!(g.current().tile_at(np), crate::map::level::Tile::Floor);
        assert!(!g.player_has_key(), "the key must be consumed");
        assert!(g.drain_events().iter().any(|e| matches!(
            e,
            GameEvent::Door {
                opened: true,
                locked: true
            }
        )));
    }

    #[test]
    fn stepping_on_a_trap_triggers_and_disarms_it() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut().set_tile(
            np,
            crate::map::level::Tile::Trap(crate::map::level::TrapKind::Arrow),
        );
        let hp_before = g.player.hp;
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert_eq!(g.player.pos, np);
        assert_eq!(
            g.current().tile_at(np),
            crate::map::level::Tile::Floor,
            "the trap must disarm after firing"
        );
        assert!(g.player.hp < hp_before, "the arrow trap must damage");
        assert!(g
            .drain_events()
            .iter()
            .any(|e| matches!(e, GameEvent::Trap)));
    }

    #[test]
    fn falling_item_trap_drops_a_random_item() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.player
            .inventory
            .push(crate::items::catalog::make_amulet());
        g.current_mut().set_tile(
            np,
            crate::map::level::Tile::Trap(crate::map::level::TrapKind::FallingItem),
        );
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert!(g.player.inventory.is_empty(), "the item must be dropped");
        assert!(
            !g.current().items_at(np).is_empty(),
            "the item is on the ground"
        );
    }

    #[test]
    fn teleport_trap_moves_the_player() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut().set_tile(
            np,
            crate::map::level::Tile::Trap(crate::map::level::TrapKind::Teleport),
        );
        let hp_before = g.player.hp;
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert_eq!(
            g.current().tile_at(np),
            crate::map::level::Tile::Floor,
            "the trap must disarm"
        );
        assert_eq!(g.player.hp, hp_before, "teleport must not damage");
        assert!(g
            .drain_events()
            .iter()
            .any(|e| matches!(e, GameEvent::Teleport)));
        assert!(
            g.current().is_walkable(g.player.pos),
            "the player must land on a walkable tile"
        );
    }

    #[test]
    fn sleep_gas_and_acid_traps_apply_their_effects() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut().set_tile(
            np,
            crate::map::level::Tile::Trap(crate::map::level::TrapKind::SleepGas),
        );
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert!(
            g.statuses.sleep > 0,
            "sleep gas must put the player to sleep"
        );

        let (dx2, dy2) = walkable_dir(&g);
        let (px2, py2) = g.player.pos;
        let np2 = ((px2 as i32 + dx2) as u8, (py2 as i32 + dy2) as u8);
        g.statuses.sleep = 0;
        g.current_mut().set_tile(
            np2,
            crate::map::level::Tile::Trap(crate::map::level::TrapKind::AcidPool),
        );
        let hp_before = g.player.hp;
        g.do_turn(crate::core::action::Action::Move(dx2, dy2));
        assert!(g.player.hp < hp_before, "the acid pool must damage");
    }

    #[test]
    fn dart_trap_poisons_the_player() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut().set_tile(
            np,
            crate::map::level::Tile::Trap(crate::map::level::TrapKind::Dart),
        );
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert!(g.statuses.poison > 0, "the dart must poison");
    }

    #[test]
    fn water_tile_slows_the_player() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut().set_tile(np, crate::map::level::Tile::Water);
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert!(g.statuses.slow > 0, "deep water must slow the player");
    }

    #[test]
    fn lava_tile_damages_the_player() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        let (dx, dy) = walkable_dir(&g);
        let (px, py) = g.player.pos;
        let np = ((px as i32 + dx) as u8, (py as i32 + dy) as u8);
        g.current_mut().set_tile(np, crate::map::level::Tile::Lava);
        let hp_before = g.player.hp;
        g.do_turn(crate::core::action::Action::Move(dx, dy));
        assert!(g.player.hp < hp_before, "lava must damage the player");
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text.starts_with("The lava sears you")));
    }

    #[test]
    fn endless_mode_lifts_the_max_depth_cap_on_descend() {
        for endless in [false, true] {
            let mut g = Game::new_test("Test", "Human", "Warrior", 42);
            g.current_level = 26;
            g.endless = endless;
            g.monsters.clear();
            let level = g.ensure_level(26);
            let down = level
                .stairs_down
                .expect("generated levels have down stairs");
            g.player.pos = down;
            g.do_turn(crate::core::action::Action::StairsDown);
            if endless {
                assert_eq!(g.current_level, 27, "endless must lift the max-depth cap");
                assert!(
                    g.levels.contains_key(&27),
                    "the level beyond the cap must be generated"
                );
            } else {
                assert_eq!(
                    g.current_level, 26,
                    "the max-depth cap must hold without endless mode"
                );
            }
        }
    }

    #[test]
    fn bad_mushroom_can_poison_the_player() {
        let mut g = Game::new_test("Test", "Human", "Warrior", 42);
        g.monsters.clear();
        g.player.inventory.push(crate::items::catalog::make_food(
            crate::items::item::FoodKind::Mushroom,
        ));
        for _ in 0..200 {
            if g.statuses.poison > 0 {
                break;
            }
            g.player.inventory.push(crate::items::catalog::make_food(
                crate::items::item::FoodKind::Mushroom,
            ));
            g.eat_item(g.player.inventory.len() - 1);
        }
        assert!(
            g.statuses.poison > 0,
            "a bad mushroom must sometimes poison the player"
        );
        assert!(g
            .messages
            .all()
            .iter()
            .any(|m| m.text == "The mushroom was bad! You feel poisoned."));
    }
}
