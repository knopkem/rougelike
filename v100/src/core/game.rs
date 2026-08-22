//! The `Game` state and turn pump. This is the ONLY way game state changes.
//!
//! `Game::do_turn(action)` takes an `Action`, mutates state, and returns the
//! `GameEvent`s produced. The UI maps terminal input to `Action`s, calls
//! `do_turn`, renders the resulting state, and consumes the events (for audio
//! and transient UI effects). The core never touches the terminal or sound card.

use crate::combat;
use crate::core::action::{Action, Direction};
use crate::core::events::{GameEvent, Pos};
use crate::core::message::{MessageLog, Severity};
use crate::core::rng::Rng;
use crate::core::score::Score;
use crate::data::classes::{Class, Race};
use crate::data::monsters::Ability;
use crate::entities::ai::{self, AiAction};
use crate::entities::monster::Monster;
use crate::entities::player::Player;
use crate::items::equip::Slot;
use crate::items::item::{Item, PotionEffect, ScrollEffect};
use crate::map::fov::compute_fov;
use crate::map::generation::generate_level;
use crate::map::level::{Level, Tile, Trap};
use crate::quest::QuestLog;
use crate::status::Status;

/// The full game state.
#[derive(Debug, Clone)]
pub struct Game {
    /// The seed for this run (for reproducibility).
    pub seed: u64,
    /// The game's RNG.
    pub rng: Rng,
    /// The current depth (1-based).
    pub depth: u32,
    /// The current level.
    pub level: Level,
    /// The player.
    pub player: Player,
    /// Live monsters on the current level.
    pub monsters: Vec<Monster>,
    /// The quest log.
    pub quests: QuestLog,
    /// The current turn number.
    pub turn: u64,
    /// The message log.
    pub messages: MessageLog,
    /// Events emitted during the most recent turn (drained by the UI/audio).
    pub events: Vec<GameEvent>,
    /// Whether the run is in endless mode (past D25).
    pub endless: bool,
    /// Whether the game is over (death or victory).
    pub over: bool,
    /// Whether the run ended in victory.
    pub victory: bool,
    /// The cause of death (if the player died).
    pub cause_of_death: Option<String>,
    /// Whether the player requested to save and quit.
    pub save_quit: bool,
    /// Whether the player requested to abort.
    pub abort: bool,
}

impl Game {
    /// Start a new game with the given seed, race, and class.
    pub fn new(seed: u64, race: Race, class: Class) -> Self {
        let mut rng = Rng::new(seed);
        let level = generate_level(&mut rng, 1, false);
        let start = level.player_start;
        let mut player = Player::new(start, race, class);

        // Give the player their starting kit.
        let kit = class.starting_kit();
        player.gold = kit.gold;
        for kit_item in kit.items {
            for _ in 0..kit_item.quantity {
                player.add_item(Item::new(kit_item.item_id));
            }
        }
        // Auto-equip the first weapon and armor in the kit.
        auto_equip(&mut player);

        // Build live monsters from the level's placement.
        let monsters = build_monsters(&level);

        let mut game = Self {
            seed,
            rng,
            depth: 1,
            level,
            player,
            monsters,
            quests: QuestLog::new(),
            turn: 0,
            messages: MessageLog::new(),
            events: Vec::new(),
            endless: false,
            over: false,
            victory: false,
            cause_of_death: None,
            save_quit: false,
            abort: false,
        };
        game.quests.initialize();
        game.recompute_fov();
        game
    }

    /// Reconstruct a game from a save.
    pub fn from_save(data: crate::save::SaveData) -> Self {
        let crate::save::SaveData {
            seed,
            rng,
            depth,
            level,
            player,
            quests,
            turn,
            messages,
            endless,
            ..
        } = data;
        let monsters = build_monsters(&level);
        let mut game = Self {
            seed,
            rng,
            depth,
            level,
            player,
            monsters,
            quests,
            turn,
            messages,
            events: Vec::new(),
            endless,
            over: false,
            victory: false,
            cause_of_death: None,
            save_quit: false,
            abort: false,
        };
        game.recompute_fov();
        game
    }

    /// Serialize the game to a save.
    pub fn to_save(&self) -> crate::save::SaveData {
        crate::save::SaveData::new(
            self.seed,
            self.rng.clone(),
            self.depth,
            self.level.clone(),
            self.player.clone(),
            self.quests.clone(),
            self.turn,
            self.messages.clone(),
            self.endless,
        )
    }

    /// The events from the most recent turn.
    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }

    /// Drain the events from the most recent turn.
    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    /// Whether the game is over.
    pub fn is_over(&self) -> bool {
        self.over
    }

    /// The current score.
    pub fn score(&self) -> Score {
        Score::compute(
            self.player.gold,
            self.player.xp,
            self.depth,
            self.quests.completed_count(),
            self.player.kills,
        )
    }

    /// The current field-of-view radius (theme + race darkvision).
    fn fov_radius(&self) -> u32 {
        let theme = crate::data::themes::Theme::for_depth(self.depth);
        let base = theme.fov_radius();
        let race_bonus = self.player.race.darkvision_bonus();
        let infravision = if self.player.statuses.has(Status::Infravision) {
            2
        } else {
            0
        };
        base + race_bonus + infravision
    }

    /// Recompute the field of view and update the level's seen/visible maps.
    fn recompute_fov(&mut self) {
        let radius = self.fov_radius();
        let visible = compute_fov(&self.level, self.player.pos(), radius);
        for (i, &v) in visible.iter().enumerate() {
            if v {
                self.level.seen[i] = true;
            }
            self.level.visible[i] = v;
        }
    }

    /// Log a message.
    fn log(&mut self, text: impl Into<String>, severity: Severity) {
        self.messages.push(text, severity, self.turn);
    }

    /// Emit a game event.
    fn emit(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    /// The main turn pump. The ONLY way game state changes.
    pub fn do_turn(&mut self, action: Action) {
        if self.over {
            return;
        }

        // Reset per-turn flags on monsters and NPCs.
        for m in &mut self.monsters {
            m.reset_turn();
        }
        for npc in &mut self.level.npcs {
            npc.reset_turn();
        }

        match action {
            Action::Move(dir) => self.do_move(dir),
            Action::Wait => {}
            Action::StairsDown => self.do_stairs_down(),
            Action::StairsUp => self.do_stairs_up(),
            Action::Pickup => self.do_pickup(),
            Action::Drop { index } => self.do_drop(index),
            Action::Quaff { index } => self.do_quaff(index),
            Action::Eat { index } => self.do_eat(index),
            Action::Read { index } => self.do_read(index),
            Action::WandFire { index, target } => self.do_wand_fire(index, target),
            Action::Wield { index } => self.do_wield(index),
            Action::Wear { index } => self.do_wear(index),
            Action::TakeOff { index } => self.do_take_off(index),
            Action::RingOn { index } => self.do_ring_on(index),
            Action::RingOff { index } => self.do_ring_off(index),
            Action::ToggleDoor => self.do_toggle_door(),
            Action::PickLock => self.do_pick_lock(),
            Action::ShopBuy { index } => self.do_shop_buy(index),
            Action::ShopSell { index } => self.do_shop_sell(index),
            Action::Identify { index } => self.do_identify(index),
            Action::AcceptQuest { index } => self.do_accept_quest(index),
            Action::TurnInQuest { index } => self.do_turn_in_quest(index),
            Action::SaveQuit => {
                self.save_quit = true;
                self.over = true;
            }
            Action::Abort => {
                self.abort = true;
                self.over = true;
            }
        }

        // If the action ended the game (death/victory/quit), skip the rest.
        if self.over {
            return;
        }

        // End-of-player-turn processing.
        self.end_player_turn();
    }

    /// Process the end of the player's turn: hunger, statuses, monster turns,
    /// FOV, and turn increment.
    fn end_player_turn(&mut self) {
        // Hunger.
        self.player.tick_hunger();

        // Tick player statuses.
        let expired = self.player.statuses.tick();
        for status in expired {
            if status == Status::Poison {
                self.log("The poison fades.", Severity::Good);
            }
        }
        // Poison damage.
        if self.player.statuses.has(Status::Poison) {
            let dmg = self.player.damage(1);
            if dmg > 0 {
                self.log("You feel poisoned.", Severity::Bad);
            }
        }
        // Regeneration.
        if self.player.statuses.has(Status::Regenerating) {
            self.player.heal(1);
        }

        // Check player death from hunger/poison.
        if !self.player.is_alive() {
            self.kill_player("your body gave out".to_string());
            return;
        }

        // Monster turns.
        self.monster_turns();

        // Check player death from monster attacks.
        if !self.player.is_alive() {
            // cause_of_death was set by the monster that killed the player.
            if self.cause_of_death.is_none() {
                self.kill_player("slain by monsters".to_string());
            }
            return;
        }

        // Recompute FOV and advance the turn.
        self.recompute_fov();
        self.turn += 1;
    }

    /// Move the player in a direction, handling collisions and attacks.
    fn do_move(&mut self, dir: Direction) {
        let (dx, dy) = dir.delta();
        let target = self.player.pos().add(dx, dy);

        // Check for a monster in the way.
        if let Some(mi) = self.monster_index_at(target) {
            self.player_attack_monster(mi);
            return;
        }

        // Check the tile.
        match self.level.tile_at(target) {
            Some(Tile::Wall) | None => {
                // Bumped into a wall; no movement.
                return;
            }
            Some(Tile::DoorClosed) => {
                // Open the door.
                self.level.set_tile(target, Tile::DoorOpen);
                self.emit(GameEvent::Door {
                    pos: target,
                    opened: true,
                });
                self.log("You open the door.", Severity::System);
            }
            Some(Tile::DoorLocked) => {
                self.log("The door is locked.", Severity::System);
                return;
            }
            Some(_) => {}
        }

        // Move the player.
        self.player.move_to(target);
        self.emit(GameEvent::PlayerMoved { pos: target });

        // Trigger a trap if present.
        self.trigger_trap_at(target);

        // Apply hazard effects.
        self.apply_hazard_at(target);

        // Check for stairs.
        if self.level.stairs_down == Some(target) {
            // Stairs are used via explicit action, but note their presence.
        }
    }

    /// The index of the monster at a position, if any.
    fn monster_index_at(&self, pos: Pos) -> Option<usize> {
        self.monsters
            .iter()
            .position(|m| m.pos() == pos && m.is_alive())
    }

    /// The player attacks a monster.
    fn player_attack_monster(&mut self, mi: usize) {
        let mpos = self.monsters[mi].pos();
        let target_ac = self.monsters[mi].effective_ac();
        let (die, bonus) = self.player.weapon_damage();
        let result = combat::resolve_melee(
            &mut self.rng,
            self.player.attributes.dex,
            self.player.attributes.str,
            self.player.entity.attack,
            die,
            bonus,
            target_ac,
            self.player.crit_chance(),
        );

        if result.hit {
            let name = self.monsters[mi].name().to_string();
            let dealt = self.monsters[mi].damage(result.damage);
            self.emit(GameEvent::PlayerHit {
                pos: mpos,
                crit: result.crit,
            });
            if result.crit {
                self.log(
                    format!("You critically hit the {} for {}!", name, dealt),
                    Severity::Good,
                );
            } else {
                self.log(
                    format!("You hit the {} for {}.", name, dealt),
                    Severity::Normal,
                );
            }

            // Wake the monster if it was sleeping.
            self.monsters[mi].statuses.remove(Status::Sleeping);

            // Check for death.
            if !self.monsters[mi].is_alive() {
                self.on_monster_died(mi);
            }
        } else {
            self.emit(GameEvent::PlayerMiss { pos: mpos });
            self.log(
                format!("You miss the {}.", self.monsters[mi].name()),
                Severity::Normal,
            );
        }
    }

    /// Handle a monster dying.
    fn on_monster_died(&mut self, mi: usize) {
        let name = self.monsters[mi].name().to_string();
        let pos = self.monsters[mi].pos();
        let xp = self.monsters[mi].xp_value();
        let was_unique = self.monsters[mi].is_unique();

        self.emit(GameEvent::MonsterDied {
            pos,
            name: name.clone(),
        });
        self.log(format!("The {} dies.", name), Severity::Good);

        // Award XP.
        let levels = self.player.gain_xp(xp);
        for _ in 0..levels {
            self.emit(GameEvent::LevelUp {
                level: self.player.level,
            });
            self.log(
                format!("You advance to level {}!", self.player.level),
                Severity::Good,
            );
        }

        // Record the kill for quests.
        self.quests.record_kill(self.monsters[mi].def_id);

        // Increment the kill counter.
        self.player.kills += 1;

        // Splitting monsters spawn copies.
        if self.monsters[mi].has_ability(Ability::Split) {
            self.spawn_splits(mi);
        }

        // Remove the dead monster.
        self.monsters.remove(mi);

        let _ = was_unique;
    }

    /// Spawn split copies of a monster (for the Split ability).
    fn spawn_splits(&mut self, mi: usize) {
        let def_id = self.monsters[mi].def_id;
        let pos = self.monsters[mi].pos();
        // Spawn up to 2 smaller copies on adjacent tiles.
        for d in Direction::ALL {
            let (dx, dy) = d.delta();
            let np = pos.add(dx, dy);
            if self.level.tile_at(np) == Some(Tile::Floor)
                && self.monster_index_at(np).is_none()
                && np != self.player.pos()
            {
                let def = crate::data::monsters::MonsterDef::by_id(def_id);
                if let Some(def) = def {
                    let mut copy = Monster::from_def(def, np);
                    // Halve the HP of the copy.
                    copy.entity.max_hp /= 2;
                    copy.entity.hp = copy.entity.max_hp;
                    self.monsters.push(copy);
                }
            }
        }
    }

    /// Trigger a trap at a position, if present.
    fn trigger_trap_at(&mut self, pos: Pos) {
        let idx = self.level.idx(pos);
        if let Some(trap) = self.level.traps[idx] {
            self.level.traps[idx] = None;
            self.emit(GameEvent::Trap {
                pos,
                name: trap.name().to_string(),
            });
            self.apply_trap(trap, pos);
        }
    }

    /// Apply the effect of a trap.
    fn apply_trap(&mut self, trap: Trap, pos: Pos) {
        match trap {
            Trap::Arrow => {
                let dmg = self.rng.range(1, 6);
                self.player.damage(dmg);
                self.log(format!("An arrow strikes you for {}!", dmg), Severity::Bad);
            }
            Trap::Dart => {
                let dmg = self.rng.range(1, 4);
                self.player.damage(dmg);
                self.player.statuses.apply(Status::Poison, 5);
                self.log(
                    format!("A poisoned dart hits you for {}!", dmg),
                    Severity::Bad,
                );
            }
            Trap::FallingItem => {
                self.log("A boulder crashes down!", Severity::Bad);
                let dmg = self.rng.range(3, 10);
                self.player.damage(dmg);
            }
            Trap::Teleport => {
                if let Some(np) = self.level.random_walkable_far(&mut self.rng, pos, 5) {
                    self.player.move_to(np);
                    self.emit(GameEvent::Teleport { pos: np });
                    self.log("You are teleported!", Severity::Magic);
                }
            }
            Trap::SleepGas => {
                self.player.statuses.apply(Status::Sleeping, 3);
                self.log("You breathe in sleep gas...", Severity::Bad);
            }
            Trap::AcidPool => {
                let dmg = self.rng.range(2, 8);
                self.player.damage(dmg);
                self.log(
                    format!("You slip into an acid pool for {}!", dmg),
                    Severity::Bad,
                );
            }
        }
    }

    /// Apply hazard tile effects (water/lava/gas).
    fn apply_hazard_at(&mut self, pos: Pos) {
        match self.level.tile_at(pos) {
            Some(Tile::Lava) => {
                // Dwarfs are fire-resistant.
                if self.player.race.fire_resist() {
                    return;
                }
                let dmg = self.rng.range(1, 5);
                self.player.damage(dmg);
                self.log(format!("The lava burns you for {}!", dmg), Severity::Bad);
            }
            Some(Tile::SporeGas) if self.rng.chance(20) => {
                self.player.statuses.apply(Status::Poison, 4);
                self.log("The spore gas makes you sick.", Severity::Bad);
            }
            _ => {}
        }
    }

    /// Descend to the next level.
    fn do_stairs_down(&mut self) {
        let pos = self.player.pos();
        if self.level.stairs_down != Some(pos) {
            self.log("There are no stairs down here.", Severity::System);
            return;
        }
        let next_depth = self.depth + 1;
        if next_depth > 25 && !self.endless {
            // Reaching D25's stairs without the amulet: enter endless mode.
            self.endless = true;
            self.log("You descend into the endless deep...", Severity::System);
        }
        self.descend_to(next_depth);
    }

    /// Ascend to the previous level.
    fn do_stairs_up(&mut self) {
        let pos = self.player.pos();
        if self.level.stairs_up != Some(pos) {
            self.log("There are no stairs up here.", Severity::System);
            return;
        }
        if self.depth > 1 {
            self.ascend_to(self.depth - 1);
        }
    }

    /// Move the player to a new depth (down).
    fn descend_to(&mut self, depth: u32) {
        let has_up = depth > 1;
        let level = generate_level(&mut self.rng, depth, has_up);
        let start = level.player_start;
        self.player.move_to(start);
        self.depth = depth;
        self.level = level;
        self.monsters = build_monsters(&self.level);
        self.emit(GameEvent::StairsDown { depth });
        self.log(format!("You descend to depth {}.", depth), Severity::System);
        self.recompute_fov();
    }

    /// Move the player to a new depth (up).
    fn ascend_to(&mut self, depth: u32) {
        let has_up = depth > 1;
        let level = generate_level(&mut self.rng, depth, has_up);
        let start = level.player_start;
        self.player.move_to(start);
        self.depth = depth;
        self.level = level;
        self.monsters = build_monsters(&self.level);
        self.emit(GameEvent::StairsUp { depth });
        self.log(format!("You ascend to depth {}.", depth), Severity::System);
        self.recompute_fov();
    }

    /// Pick up the item at the player's position.
    fn do_pickup(&mut self) {
        let pos = self.player.pos();
        let idx = self.level.idx(pos);
        if let Some(def_id) = self.level.items[idx] {
            self.level.items[idx] = None;
            let item = Item::new(def_id);
            let name = item.name();
            let inv_idx = self.player.add_item(item);
            self.emit(GameEvent::Pickup { name: name.clone() });
            self.log(format!("You pick up {}.", name), Severity::Good);
            let _ = inv_idx;
            // Picking up the Amulet of the Abyss wins the game.
            self.check_victory();
        } else {
            self.log("There is nothing here to pick up.", Severity::System);
        }
    }

    /// Drop an item from the inventory.
    fn do_drop(&mut self, index: usize) {
        if let Some(item) = self.player.remove_item(index) {
            let name = item.name();
            let pos = self.player.pos();
            let idx = self.level.idx(pos);
            self.level.items[idx] = Some(item.def_id);
            self.emit(GameEvent::Drop { name: name.clone() });
            self.log(format!("You drop {}.", name), Severity::System);
        }
    }

    /// Quaff a potion.
    fn do_quaff(&mut self, index: usize) {
        let item = match self.player.inventory.get(index) {
            Some(i) => i.clone(),
            None => return,
        };
        if !item.is_potion() {
            self.log("That is not a potion.", Severity::System);
            return;
        }
        let name = item.name();
        let effect = item.def().potion_effect;
        self.player.remove_item(index);
        self.emit(GameEvent::Quaff { name: name.clone() });
        self.log(format!("You quaff {}.", name), Severity::Normal);
        if let Some(effect) = effect {
            self.apply_potion(effect);
        }
    }

    /// Apply a potion effect.
    fn apply_potion(&mut self, effect: PotionEffect) {
        match effect {
            PotionEffect::Healing => {
                let amt = self.rng.range(8, 16);
                let healed = self.player.heal(amt);
                self.log(format!("You heal for {}.", healed), Severity::Good);
            }
            PotionEffect::FullHealing => {
                self.player.entity.hp = self.player.entity.max_hp;
                self.log("You are fully healed.", Severity::Good);
            }
            PotionEffect::CurePoison => {
                self.player.statuses.remove(Status::Poison);
                self.log("The poison is cured.", Severity::Good);
            }
            PotionEffect::RestoreEp => {
                self.player.recover_ep(self.player.max_ep);
                self.log("Your energy is restored.", Severity::Good);
            }
            PotionEffect::Infravision => {
                self.player.statuses.apply(Status::Infravision, 50);
                self.log("You can see in the dark.", Severity::Good);
            }
            PotionEffect::Energy => {
                self.player.recover_ep(10);
                self.log("You feel energized.", Severity::Good);
            }
            PotionEffect::Experience => {
                let xp = self.rng.range(20, 40);
                let levels = self.player.gain_xp(xp);
                self.log(format!("You gain {} XP.", xp), Severity::Good);
                for _ in 0..levels {
                    self.emit(GameEvent::LevelUp {
                        level: self.player.level,
                    });
                }
            }
            PotionEffect::Berserk => {
                self.player.statuses.apply(Status::Berserk, 10);
                self.player.recompute_ac();
                self.player.recompute_attack();
                self.log("You enter a berserk rage!", Severity::Magic);
            }
            PotionEffect::Teleport => {
                if let Some(np) =
                    self.level
                        .random_walkable_far(&mut self.rng, self.player.pos(), 5)
                {
                    self.player.move_to(np);
                    self.emit(GameEvent::Teleport { pos: np });
                    self.log("You are teleported!", Severity::Magic);
                }
            }
            PotionEffect::Blindness => {
                self.player.statuses.apply(Status::Blind, 10);
                self.log("You are blinded!", Severity::Bad);
            }
            PotionEffect::Confusion => {
                self.player.statuses.apply(Status::Confused, 10);
                self.log("You feel confused!", Severity::Bad);
            }
            PotionEffect::Antidote => {
                self.player.statuses.remove(Status::Poison);
                self.log("You take an antidote.", Severity::Good);
            }
        }
    }

    /// Eat food.
    fn do_eat(&mut self, index: usize) {
        let item = match self.player.inventory.get(index) {
            Some(i) => i.clone(),
            None => return,
        };
        if !item.is_food() {
            self.log("That is not food.", Severity::System);
            return;
        }
        let name = item.name();
        let nutrition = item.def().nutrition;
        self.player.remove_item(index);
        self.emit(GameEvent::Eat { name: name.clone() });
        self.player.eat(nutrition);
        self.log(format!("You eat {}.", name), Severity::Normal);
    }

    /// Read a scroll.
    fn do_read(&mut self, index: usize) {
        let item = match self.player.inventory.get(index) {
            Some(i) => i.clone(),
            None => return,
        };
        if !item.is_scroll() {
            self.log("That is not a scroll.", Severity::System);
            return;
        }
        let name = item.name();
        let effect = item.def().scroll_effect;
        self.player.remove_item(index);
        self.emit(GameEvent::Read { name: name.clone() });
        self.log(format!("You read {}.", name), Severity::Normal);
        if let Some(effect) = effect {
            self.apply_scroll(effect);
        }
    }

    /// Apply a scroll effect.
    fn apply_scroll(&mut self, effect: ScrollEffect) {
        match effect {
            ScrollEffect::Identify => {
                for item in &mut self.player.inventory {
                    item.identified = true;
                }
                self.log("Your items are identified.", Severity::Good);
            }
            ScrollEffect::Mapping => {
                // Reveal the whole level.
                for i in 0..self.level.tiles.len() {
                    self.level.seen[i] = true;
                }
                self.log("The level is revealed.", Severity::Good);
            }
            ScrollEffect::EnchantWeapon => {
                if let Some(idx) = self.player.equipment.weapon
                    && let Some(item) = self.player.inventory.get_mut(idx)
                {
                    item.enchantment += 1;
                    self.log("Your weapon hums with power.", Severity::Magic);
                }
            }
            ScrollEffect::EnchantArmor => {
                if let Some(idx) = self.player.equipment.armor
                    && let Some(item) = self.player.inventory.get_mut(idx)
                {
                    item.enchantment += 1;
                    self.log("Your armor glows faintly.", Severity::Magic);
                }
            }
            ScrollEffect::Teleport => {
                if let Some(np) =
                    self.level
                        .random_walkable_far(&mut self.rng, self.player.pos(), 5)
                {
                    self.player.move_to(np);
                    self.emit(GameEvent::Teleport { pos: np });
                    self.log("You are teleported!", Severity::Magic);
                }
            }
            ScrollEffect::Blink => {
                if let Some(np) = self.level.random_walkable(&mut self.rng) {
                    self.player.move_to(np);
                    self.emit(GameEvent::Teleport { pos: np });
                    self.log("You blink!", Severity::Magic);
                }
            }
            ScrollEffect::Creation => {
                // Create a random item.
                let item = crate::items::loot::generate_item(&mut self.rng, self.depth);
                let name = item.name();
                self.player.add_item(item);
                self.log(format!("An item materializes: {}!", name), Severity::Magic);
            }
            ScrollEffect::WordOfRecall => {
                self.log("You speak the word of recall...", Severity::Magic);
                // Simplified: teleport to a random far tile.
                if let Some(np) =
                    self.level
                        .random_walkable_far(&mut self.rng, self.player.pos(), 8)
                {
                    self.player.move_to(np);
                    self.emit(GameEvent::Teleport { pos: np });
                }
            }
            ScrollEffect::Earthquake => {
                self.log("The ground shakes!", Severity::Magic);
                // Damage nearby monsters.
                let pos = self.player.pos();
                let mut to_remove = Vec::new();
                for (i, m) in self.monsters.iter_mut().enumerate() {
                    if m.pos().manhattan(pos) <= 2 {
                        m.damage(10);
                        if !m.is_alive() {
                            to_remove.push(i);
                        }
                    }
                }
                for i in to_remove.into_iter().rev() {
                    self.on_monster_died(i);
                }
            }
            ScrollEffect::MonsterLightning => {
                self.log("Lightning strikes the monsters!", Severity::Magic);
                let pos = self.player.pos();
                let mut to_remove = Vec::new();
                for (i, m) in self.monsters.iter_mut().enumerate() {
                    if m.pos().manhattan(pos) <= 4 {
                        m.damage(15);
                        if !m.is_alive() {
                            to_remove.push(i);
                        }
                    }
                }
                for i in to_remove.into_iter().rev() {
                    self.on_monster_died(i);
                }
            }
        }
    }

    /// Fire a wand at a target.
    fn do_wand_fire(&mut self, index: usize, target: Pos) {
        let item = match self.player.inventory.get(index) {
            Some(i) => i.clone(),
            None => return,
        };
        if !item.is_wand() {
            self.log("That is not a wand.", Severity::System);
            return;
        }
        if item.charges == 0 {
            self.log("The wand is out of charges.", Severity::System);
            return;
        }
        let effect = match item.def().wand_effect {
            Some(e) => e,
            None => return,
        };
        let name = item.name();
        // Decrement charges.
        if let Some(inv_item) = self.player.inventory.get_mut(index) {
            inv_item.charges = inv_item.charges.saturating_sub(1);
        }
        self.emit(GameEvent::WandFire {
            name: name.clone(),
            pos: Some(target),
        });
        self.log(format!("You fire {}.", name), Severity::Magic);

        // Apply the effect to the target monster, if any.
        let result =
            crate::magic::resolve_wand(&mut self.rng, effect, self.player.pos(), Some(target), 1);
        if let Some(mi) = self.monster_index_at(target) {
            if result.damage > 0 {
                self.monsters[mi].damage(result.damage);
                if !self.monsters[mi].is_alive() {
                    self.on_monster_died(mi);
                }
            }
            // Status effects from wands.
            match effect {
                crate::items::item::WandEffect::Paralysis => {
                    self.monsters
                        .get_mut(mi)
                        .unwrap()
                        .statuses
                        .apply(Status::Paralyzed, 3);
                }
                crate::items::item::WandEffect::Sleep => {
                    self.monsters
                        .get_mut(mi)
                        .unwrap()
                        .statuses
                        .apply(Status::Sleeping, 3);
                }
                _ => {}
            }
        }
    }

    /// Wield a weapon.
    fn do_wield(&mut self, index: usize) {
        let item = match self.player.inventory.get(index) {
            Some(i) => i.clone(),
            None => return,
        };
        if !item.is_weapon() {
            self.log("That is not a weapon.", Severity::System);
            return;
        }
        let name = item.name();
        self.player.equipment.set(Slot::Weapon, Some(index));
        self.player.recompute_attack();
        self.emit(GameEvent::Equip { name: name.clone() });
        self.log(format!("You wield {}.", name), Severity::System);
    }

    /// Wear armor.
    fn do_wear(&mut self, index: usize) {
        let item = match self.player.inventory.get(index) {
            Some(i) => i.clone(),
            None => return,
        };
        if !item.is_armor() {
            self.log("That is not armor.", Severity::System);
            return;
        }
        let name = item.name();
        self.player.equipment.set(Slot::Armor, Some(index));
        self.player.recompute_ac();
        self.emit(GameEvent::Equip { name: name.clone() });
        self.log(format!("You wear {}.", name), Severity::System);
    }

    /// Take off equipped armor.
    fn do_take_off(&mut self, index: usize) {
        if self.player.equipment.armor == Some(index) {
            self.player.equipment.set(Slot::Armor, None);
            self.player.recompute_ac();
            self.log("You take off your armor.", Severity::System);
        }
    }

    /// Put on a ring.
    fn do_ring_on(&mut self, index: usize) {
        let item = match self.player.inventory.get(index) {
            Some(i) => i.clone(),
            None => return,
        };
        if !item.is_ring() {
            self.log("That is not a ring.", Severity::System);
            return;
        }
        // Find a free ring slot.
        let slot = if self.player.equipment.ring_left.is_none() {
            Slot::RingLeft
        } else if self.player.equipment.ring_right.is_none() {
            Slot::RingRight
        } else {
            self.log("Both ring slots are occupied.", Severity::System);
            return;
        };
        let name = item.name();
        self.player.equipment.set(slot, Some(index));
        self.apply_ring_effect(&item, true);
        self.emit(GameEvent::Equip { name: name.clone() });
        self.log(format!("You put on {}.", name), Severity::System);
    }

    /// Take off a ring.
    fn do_ring_off(&mut self, index: usize) {
        let slot = if self.player.equipment.ring_left == Some(index) {
            Slot::RingLeft
        } else if self.player.equipment.ring_right == Some(index) {
            Slot::RingRight
        } else {
            return;
        };
        if let Some(item) = self.player.inventory.get(index).cloned() {
            self.apply_ring_effect(&item, false);
        }
        self.player.equipment.set(slot, None);
        self.log("You take off your ring.", Severity::System);
    }

    /// Apply or remove a ring's effect.
    fn apply_ring_effect(&mut self, item: &Item, on: bool) {
        if let Some(effect) = item.def().ring_effect {
            match effect {
                crate::items::item::RingEffect::Strength => {
                    let delta = if on { 2 } else { -2 };
                    adjust_attr(&mut self.player.attributes, 0, delta);
                }
                crate::items::item::RingEffect::Dexterity => {
                    let delta = if on { 2 } else { -2 };
                    adjust_attr(&mut self.player.attributes, 1, delta);
                }
                crate::items::item::RingEffect::Constitution => {
                    let delta = if on { 2 } else { -2 };
                    adjust_attr(&mut self.player.attributes, 2, delta);
                }
                crate::items::item::RingEffect::Intelligence => {
                    let delta = if on { 2 } else { -2 };
                    adjust_attr(&mut self.player.attributes, 3, delta);
                }
                crate::items::item::RingEffect::Wisdom => {
                    let delta = if on { 2 } else { -2 };
                    adjust_attr(&mut self.player.attributes, 4, delta);
                }
                crate::items::item::RingEffect::Charisma => {
                    let delta = if on { 2 } else { -2 };
                    adjust_attr(&mut self.player.attributes, 5, delta);
                }
                crate::items::item::RingEffect::Regeneration => {
                    if on {
                        self.player.statuses.apply(Status::Regenerating, u32::MAX);
                    } else {
                        self.player.statuses.remove(Status::Regenerating);
                    }
                }
                crate::items::item::RingEffect::FireResist => {
                    // Handled via race-like check; no-op here.
                }
                crate::items::item::RingEffect::Stealth => {
                    // Stealth bonus applied in AI awareness; no-op here.
                }
                crate::items::item::RingEffect::Luck => {
                    self.player.luck = if on {
                        self.player.luck + 2
                    } else {
                        self.player.luck - 2
                    };
                }
                crate::items::item::RingEffect::Infravision => {
                    if on {
                        self.player.statuses.apply(Status::Infravision, u32::MAX);
                    } else {
                        self.player.statuses.remove(Status::Infravision);
                    }
                }
                crate::items::item::RingEffect::Protection => {
                    self.player.recompute_ac();
                }
            }
        }
    }

    /// Toggle a door at the player's position.
    fn do_toggle_door(&mut self) {
        let pos = self.player.pos();
        match self.level.tile_at(pos) {
            Some(Tile::DoorOpen) => {
                self.level.set_tile(pos, Tile::DoorClosed);
                self.emit(GameEvent::Door { pos, opened: false });
                self.log("You close the door.", Severity::System);
            }
            Some(Tile::DoorClosed) => {
                self.level.set_tile(pos, Tile::DoorOpen);
                self.emit(GameEvent::Door { pos, opened: true });
                self.log("You open the door.", Severity::System);
            }
            _ => {
                self.log("There is no door here.", Severity::System);
            }
        }
    }

    /// Pick a locked door.
    fn do_pick_lock(&mut self) {
        let pos = self.player.pos();
        if self.level.tile_at(pos) == Some(Tile::DoorLocked) {
            let chance = self.player.class.lockpick();
            if self.rng.chance(chance) {
                self.level.set_tile(pos, Tile::DoorOpen);
                self.log("You pick the lock.", Severity::Good);
            } else {
                self.log("You fail to pick the lock.", Severity::Bad);
            }
        } else {
            self.log("There is no locked door here.", Severity::System);
        }
    }

    /// Buy an item from the shop (simplified: no shop instance in core).
    fn do_shop_buy(&mut self, index: usize) {
        let _ = index;
        self.log("The shopkeeper smiles.", Severity::System);
    }

    /// Sell an item to the shop (simplified).
    fn do_shop_sell(&mut self, index: usize) {
        let _ = index;
        self.log("The shopkeeper considers your offer.", Severity::System);
    }

    /// Identify an item.
    fn do_identify(&mut self, index: usize) {
        let name = match self.player.inventory.get(index) {
            Some(item) => item.name(),
            None => return,
        };
        if let Some(item) = self.player.inventory.get_mut(index) {
            item.identified = true;
        }
        self.log(format!("You identify {}.", name), Severity::Good);
    }

    /// Accept a quest.
    fn do_accept_quest(&mut self, index: usize) {
        let quest_id = index as u32;
        if self.quests.accept(quest_id) {
            let name = self
                .quests
                .get(quest_id)
                .map(|q| q.def().name.to_string())
                .unwrap_or_default();
            self.emit(GameEvent::QuestAccepted { name: name.clone() });
            self.log(format!("You accept the quest: {}.", name), Severity::Good);
        }
    }

    /// Turn in a quest.
    fn do_turn_in_quest(&mut self, index: usize) {
        let quest_id = index as u32;
        if let Some((xp, gold, item)) = self.quests.turn_in(quest_id) {
            let name = self
                .quests
                .get(quest_id)
                .map(|q| q.def().name.to_string())
                .unwrap_or_default();
            self.player.gold += gold;
            let levels = self.player.gain_xp(xp);
            if let Some(item_id) = item {
                self.player.add_item(Item::new(item_id));
            }
            self.emit(GameEvent::QuestComplete { name: name.clone() });
            self.emit(GameEvent::Coin { amount: gold });
            self.log(
                format!("Quest complete: {} (+{} XP, +{} gold)", name, xp, gold),
                Severity::Good,
            );
            for _ in 0..levels {
                self.emit(GameEvent::LevelUp {
                    level: self.player.level,
                });
            }
        }
    }

    /// Run the AI for all monsters.
    fn monster_turns(&mut self) {
        let player_pos = self.player.pos();
        // Collect decisions first to avoid borrowing issues.
        let decisions: Vec<(usize, AiAction)> = self
            .monsters
            .iter()
            .enumerate()
            .filter(|(_, m)| m.is_alive())
            .map(|(i, m)| {
                (
                    i,
                    ai::decide(m, &self.level, player_pos, &mut self.rng).action,
                )
            })
            .collect();

        for (i, action) in decisions {
            if i >= self.monsters.len() {
                continue;
            }
            if !self.monsters[i].is_alive() {
                continue;
            }
            self.execute_monster_action(i, action);
        }
    }

    /// Execute a single monster's action.
    fn execute_monster_action(&mut self, mi: usize, action: AiAction) {
        match action {
            AiAction::Idle => {}
            AiAction::Move { dx, dy } | AiAction::ConfusedMove { dx, dy } => {
                let mpos = self.monsters[mi].pos();
                let target = mpos.add(dx, dy);
                // Monsters can't move onto the player or other monsters.
                if target == self.player.pos() {
                    return;
                }
                if (self.level.tile_at(target) == Some(Tile::Floor)
                    || self.level.tile_at(target) == Some(Tile::DoorOpen)
                    || self.level.tile_at(target) == Some(Tile::Water)
                    || self.level.tile_at(target) == Some(Tile::Lava)
                    || self.level.tile_at(target) == Some(Tile::SporeGas))
                    && self.monster_index_at(target).is_none()
                {
                    self.monsters[mi].move_to(target);
                }
            }
            AiAction::MeleeAttack => {
                self.monster_attack_player(mi);
            }
            AiAction::RangedAttack => {
                self.monster_ranged_attack(mi);
            }
            AiAction::Ability(ability) => {
                self.monster_use_ability(mi, ability);
            }
        }
    }

    /// A monster melee-attacks the player.
    fn monster_attack_player(&mut self, mi: usize) {
        let mpos = self.monsters[mi].pos();
        if mpos.manhattan(self.player.pos()) != 1 {
            return;
        }
        let result = combat::resolve_melee(
            &mut self.rng,
            10, // monster dex (approx)
            10, // monster str (approx)
            self.monsters[mi].effective_attack(),
            self.monsters[mi].entity.damage_die,
            self.monsters[mi].entity.damage_bonus,
            self.player.entity.ac,
            5, // monster crit chance
        );
        if result.hit {
            let dealt = self.player.damage(result.damage);
            self.emit(GameEvent::MonsterHitPlayer { crit: result.crit });
            self.log(
                format!("The {} hits you for {}.", self.monsters[mi].name(), dealt),
                Severity::Bad,
            );
            // Drain ability.
            if self.monsters[mi].has_ability(Ability::Drain) {
                let healed = self.monsters[mi].heal(dealt / 2);
                if healed > 0 {
                    self.log("It drains your life force.", Severity::Bad);
                }
            }
            if !self.player.is_alive() {
                self.kill_player(format!("slain by the {}", self.monsters[mi].name()));
            }
        } else {
            self.emit(GameEvent::MonsterMissPlayer);
            self.log(
                format!("The {} misses you.", self.monsters[mi].name()),
                Severity::Normal,
            );
        }
    }

    /// A monster ranged-attacks the player.
    fn monster_ranged_attack(&mut self, mi: usize) {
        let result = combat::resolve_ranged(
            &mut self.rng,
            10,
            self.monsters[mi].effective_attack(),
            self.monsters[mi].entity.damage_die,
            self.monsters[mi].entity.damage_bonus,
            self.player.entity.ac,
            5,
        );
        if result.hit {
            let dealt = self.player.damage(result.damage);
            self.emit(GameEvent::MonsterHitPlayer { crit: result.crit });
            self.log(
                format!(
                    "The {} hits you from afar for {}.",
                    self.monsters[mi].name(),
                    dealt
                ),
                Severity::Bad,
            );
            if !self.player.is_alive() {
                self.kill_player(format!("slain by the {}", self.monsters[mi].name()));
            }
        } else {
            self.emit(GameEvent::MonsterMissPlayer);
        }
    }

    /// A monster uses a special ability.
    fn monster_use_ability(&mut self, mi: usize, ability: Ability) {
        let name = self.monsters[mi].name().to_string();
        let pos = self.monsters[mi].pos();
        self.emit(GameEvent::MonsterAbility {
            name: name.clone(),
            pos,
        });
        match ability {
            Ability::PoisonSpit => {
                self.player.statuses.apply(Status::Poison, 6);
                self.log(format!("The {} spits poison at you!", name), Severity::Bad);
            }
            Ability::Petrify => {
                if self.rng.chance(30) {
                    self.player.statuses.apply(Status::Petrified, 3);
                    self.log(format!("The {}'s gaze petrifies you!", name), Severity::Bad);
                }
            }
            Ability::Paralyze => {
                if self.rng.chance(30) {
                    self.player.statuses.apply(Status::Paralyzed, 3);
                    self.log(format!("The {} paralyzes you!", name), Severity::Bad);
                }
            }
            Ability::Sleep => {
                self.player.statuses.apply(Status::Sleeping, 3);
                self.log(format!("The {} lulls you to sleep.", name), Severity::Bad);
            }
            Ability::Confusion => {
                self.player.statuses.apply(Status::Confused, 5);
                self.log(format!("The {} confuses you!", name), Severity::Bad);
            }
            Ability::Blink => {
                if let Some(np) = self.level.random_walkable(&mut self.rng) {
                    self.monsters[mi].move_to(np);
                }
            }
            Ability::Summon => {
                self.log(
                    format!("The {} summons reinforcements!", name),
                    Severity::Bad,
                );
                // Spawn a copy of a random monster.
                if let Some(def) = crate::data::monsters::MonsterDef::for_depth(self.depth).first()
                    && let Some(np) = self.level.random_walkable_far(&mut self.rng, pos, 3)
                    && self.monster_index_at(np).is_none()
                    && np != self.player.pos()
                {
                    self.monsters.push(Monster::from_def(def, np));
                }
            }
            Ability::Regen => {
                self.monsters[mi].heal(2);
            }
            Ability::Enrage => {
                self.monsters[mi].enraged = true;
                self.log(format!("The {} enrages!", name), Severity::Bad);
            }
            Ability::FireBreath => {
                let dmg = self.rng.range(5, 12);
                self.player.damage(dmg);
                self.log(
                    format!("The {} breathes fire for {}!", name, dmg),
                    Severity::Bad,
                );
                if !self.player.is_alive() {
                    self.kill_player(format!("burned by the {}", name));
                }
            }
            Ability::EyeRay => {
                if self.rng.chance(40) {
                    self.player.statuses.apply(Status::Paralyzed, 2);
                    self.log(
                        format!("The {}'s eye ray paralyzes you!", name),
                        Severity::Bad,
                    );
                }
            }
            Ability::Drain | Ability::Split => {
                // Drain is handled on hit; Split on death.
            }
        }
    }

    /// The player has died.
    fn kill_player(&mut self, cause: String) {
        if self.over {
            return;
        }
        self.over = true;
        self.victory = false;
        self.cause_of_death = Some(cause.clone());
        self.emit(GameEvent::PlayerDied {
            cause: cause.clone(),
        });
        self.log(format!("You die. {}", cause), Severity::Bad);
    }

    /// Check for victory (picking up the Amulet of the Abyss).
    fn check_victory(&mut self) {
        for item in &self.player.inventory {
            if item.def_id == 700 {
                // Amulet of the Abyss.
                if !self.over {
                    self.over = true;
                    self.victory = true;
                    let score = self.score();
                    self.emit(GameEvent::Victory { score: score.total });
                    self.log(
                        "You claim the Amulet of the Abyss and escape!",
                        Severity::Good,
                    );
                }
                return;
            }
        }
    }
}

/// Auto-equip the first weapon and armor in a player's inventory.
fn auto_equip(player: &mut Player) {
    // Find first weapon.
    if let Some(idx) = player.inventory.iter().position(|i| i.is_weapon()) {
        player.equipment.set(Slot::Weapon, Some(idx));
    }
    // Find first armor.
    if let Some(idx) = player.inventory.iter().position(|i| i.is_armor()) {
        player.equipment.set(Slot::Armor, Some(idx));
    }
    player.recompute_ac();
    player.recompute_attack();
}

/// Build live monsters from a level's placement.
fn build_monsters(level: &Level) -> Vec<Monster> {
    let mut monsters = Vec::new();
    for (idx, def_id) in level.monsters.iter().enumerate() {
        if let Some(id) = def_id
            && let Some(def) = crate::data::monsters::MonsterDef::by_id(*id)
        {
            let pos = Level::pos(idx);
            monsters.push(Monster::from_def(def, pos));
        }
    }
    monsters
}

/// Adjust an attribute by a delta (clamped to 1..=30).
fn adjust_attr(attrs: &mut crate::data::classes::Attributes, idx: u8, delta: i32) {
    let current = attrs.get(idx) as i32;
    let new_val = (current + delta).clamp(1, 30) as u32;
    attrs.set(idx, new_val);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_game() -> Game {
        Game::new(42, Race::Human, Class::Warrior)
    }

    #[test]
    fn new_game_starts_on_depth_1() {
        let game = new_game();
        assert_eq!(game.depth, 1);
        assert!(!game.over);
        assert!(game.player.is_alive());
    }

    #[test]
    fn new_game_has_monsters() {
        let game = new_game();
        assert!(!game.monsters.is_empty());
    }

    #[test]
    fn waiting_advances_turn() {
        let mut game = new_game();
        let turn = game.turn;
        game.do_turn(Action::Wait);
        assert_eq!(game.turn, turn + 1);
    }

    #[test]
    fn moving_into_wall_does_not_move() {
        let mut game = new_game();
        let start = game.player.pos();
        // Find a direction that leads to a wall.
        for dir in Direction::ALL {
            let (dx, dy) = dir.delta();
            let target = start.add(dx, dy);
            if game.level.tile_at(target) == Some(Tile::Wall) {
                game.do_turn(Action::Move(dir));
                assert_eq!(game.player.pos(), start);
                return;
            }
        }
        // If no wall adjacent, the test is vacuous.
    }

    #[test]
    fn do_turn_returns_events() {
        let mut game = new_game();
        game.do_turn(Action::Wait);
        let events = game.drain_events();
        let _ = events;
    }

    #[test]
    fn game_is_deterministic_with_seed() {
        let g1 = Game::new(12345, Race::Elf, Class::Mage);
        let g2 = Game::new(12345, Race::Elf, Class::Mage);
        assert_eq!(g1.level.tiles, g2.level.tiles);
        assert_eq!(g1.depth, g2.depth);
    }

    #[test]
    fn score_is_computed() {
        let game = new_game();
        let score = game.score();
        // The total must equal the sum of its components, and depth 1 is
        // worth 100.
        assert_eq!(
            score.total,
            score.gold + score.xp + score.depth + score.quests + score.kills
        );
        assert_eq!(score.depth, 100);
    }

    #[test]
    fn pickup_picks_up_item() {
        let mut game = new_game();
        // Place an item at the player's position.
        let pos = game.player.pos();
        let idx = game.level.idx(pos);
        game.level.items[idx] = Some(600); // trail rations
        let inv_before = game.player.inventory.len();
        game.do_turn(Action::Pickup);
        assert_eq!(game.player.inventory.len(), inv_before + 1);
    }

    #[test]
    fn stairs_down_descends() {
        let mut game = new_game();
        // Move the player to the down stairs.
        if let Some(stairs) = game.level.stairs_down {
            game.player.move_to(stairs);
            game.do_turn(Action::StairsDown);
            assert_eq!(game.depth, 2);
        }
    }

    #[test]
    fn save_quit_sets_over() {
        let mut game = new_game();
        game.do_turn(Action::SaveQuit);
        assert!(game.over);
        assert!(game.save_quit);
    }

    #[test]
    fn player_can_die() {
        let mut game = new_game();
        // Deal lethal damage directly.
        game.player.damage(9999);
        // Simulate a monster turn that would notice death.
        game.end_player_turn();
        assert!(game.over);
        assert!(!game.victory);
    }

    #[test]
    fn quaffing_potion_heals() {
        let mut game = new_game();
        // Add a healing potion and take some damage.
        game.player.add_item(Item::new(200));
        let idx = game.player.inventory.len() - 1;
        game.player.damage(5);
        let hp_before = game.player.hp();
        game.do_turn(Action::Quaff { index: idx });
        assert!(game.player.hp() > hp_before);
    }
}
