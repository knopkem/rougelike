//! Core game state: floors, player, turn pump, victory/death checks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::core::events::GameEvent;
use crate::core::message::MessageLog;
use crate::core::rng::{Rng, RngLike};
use crate::entities::monster::Monster;
use crate::entities::player::Player;
use crate::items::item::Item;
use crate::map::fov;
use crate::map::gen;
use crate::map::level::{Level, LevelTheme};
use crate::quest::{Quest, QuestLog};
use crate::status::Statuses;

pub const MAX_LEVELS: u8 = 26;

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
    pub endless: bool,
    pub amulet_carried: bool,
    pub amulet_taken: bool,
    pub tombstones: Vec<Tombstone>,
    pub spawn_timer: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tombstone {
    pub level: u8,
    pub x: u8,
    pub y: u8,
    pub text: String,
}

impl Game {
    pub fn new(seed: u64, name: &str, race: &str, class: &str) -> Self {
        let mut rng = Rng::new(seed);
        let player = Player::new(name, race, class, &mut rng);
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
            endless: false,
            amulet_carried: false,
            amulet_taken: false,
            tombstones: Vec::new(),
            spawn_timer: 0,
        };
        game.build_level(1);
        game
    }

    pub fn new_test(name: &str, race: &str, class: &str, seed: u64) -> Self {
        let mut g = Self::new(seed, name, race, class);
        g.messages.push(0, crate::core::message::MessageKind::System, "Welcome to Deepdelve!");
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
        let max = n + 4;
        while placed < n && placed < max {
            if let Some(p) = crate::map::gen::random_floor_tile(self, depth, &mut self.rng.clone()) {
                if p != self.player.pos {
                    let mut m = crate::entities::monster::spawn_monster(
                        &mut self.rng,
                        depth,
                        self.endless,
                    );
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

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let (px, py) = (self.player.pos.0, self.player.pos.1);
        let nx = (px as i32 + dx) as u8;
        let ny = (py as i32 + dy) as u8;
        let np = (nx, ny);

        if let Some(m_idx) = self
            .monsters
            .iter()
            .position(|m| m.pos == np && !m.dead)
        {
            self.attack_monster(m_idx);
            return true;
        }
        if !self.current().is_walkable(np) {
            return true;
        }
        self.player.pos = np;
        self.emit(GameEvent::Footstep);
        if let Some(gold) = self.current_mut().take_gold_at(np) {
            self.player.gold += gold;
            self.emit(GameEvent::Coin);
            self.log(
                crate::core::message::MessageKind::Normal,
                format!("You pick up {gold} gold."),
            );
        }
        {
            let mut quests = self.quests.clone();
            quests.check_progress(self);
            self.quests = quests;
        }
        true
    }

    fn attack_monster(&mut self, idx: usize) {
        let combat_rng = self.rng.clone();
        let mut combat = crate::combat::Combat::new(combat_rng);
        let m_name = self.monsters[idx].name.clone();
        let res = combat.player_attacks(&self.player, &self.monsters[idx]);
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
        if !self.monsters.is_empty()
            && idx < self.monsters.len()
            && self.monsters[idx].hp > 0
        {
            let res2 = combat.monster_attacks(&self.monsters[idx], &self.player);
            if res2.hit {
                let dmg = res2.damage;
                self.player.hp = self.player.hp.saturating_sub(dmg);
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
        self.player.xp += xp;
        self.log(
            crate::core::message::MessageKind::Good,
            format!("{name} is {how} (+{xp} XP)"),
        );
        self.emit(GameEvent::MonsterDeath { tier });
        if crate::items::loot::maybe_drop(&mut self.rng, tier) {
            let drop = crate::items::loot::roll_drop(
                &mut self.rng,
                self.current_level,
                m.def.rarity,
            );
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
        let next = self.current_level + 1;
        if next > MAX_LEVELS {
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
                crate::items::item::ItemKind::Weapon(_)
                    | crate::items::item::ItemKind::Shield(_)
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

    fn take_off_item(&mut self, _slot: usize) {
        if let Some(a) = self.player.armor.take() {
            self.player.inventory.push(a);
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

    fn identify_item(&mut self, slot: usize) {
        if slot < self.player.inventory.len() {
            self.player.inventory[slot].identified = true;
        }
    }

    fn apply_potion(&mut self, item: &Item) {
        use crate::items::item::PotionKind;
        match item.kind {
            crate::items::item::ItemKind::Potion(PotionKind::Healing(small)) => {
                let heal = if small { 10 } else { 30 };
                self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                self.log(
                    crate::core::message::MessageKind::Good,
                    format!("You feel better. (+{heal} HP)"),
                );
            }
            crate::items::item::ItemKind::Potion(PotionKind::CurePoison) => {
                self.statuses.poison = 0;
                self.log(
                    crate::core::message::MessageKind::Good,
                    "You feel the poison leave your body.",
                );
            }
            _ => {
                self.log(
                    crate::core::message::MessageKind::Normal,
                    format!("You drink the {}.", item.name()),
                );
            }
        }
    }

    fn apply_scroll(&mut self, item: &Item) {
        self.log(
            crate::core::message::MessageKind::Normal,
            format!("You read the {}.", item.name()),
        );
    }

      fn monster_turns(&mut self) {
        let mut i = 0;
        while i < self.monsters.len() {
            let m = self.monsters[i].clone();
            let mut rng = self.rng.clone();
            let decision = {
                let mut ai_game = crate::entities::ai::AiGame::new(
                    self.current(),
                    self.player.pos,
                    &self.monsters,
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
                                self.player.hp = self.player.hp.saturating_sub(dmg);
                                self.log(
                                    crate::core::message::MessageKind::Combat,
                                    format!(
                                        "{} hits you for {dmg}.",
                                        self.monsters[i].name
                                    ),
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
        {
            let mut statuses = self.statuses.clone();
            statuses.tick(&mut self.player);
            self.statuses = statuses;
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
        if let Some(p) = crate::map::gen::random_floor_tile(
            self,
            self.current_level,
            &mut self.rng.clone(),
        ) {
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
            cause: DeathCause::Slain,
            date: "1970-01-01".to_string(),
        }
    }
}
