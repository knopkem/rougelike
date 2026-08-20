# Deepdelve — Issues Found During Plan-vs-Code Review

Generated from a full review of `src/` against `PLAN.md` (plus `cargo check`, which passes
with 17 warnings). Grouped by priority: **P0** = real bugs / broken behavior, **P1** =
plan features that are stubbed, dead, or unreachable. Line numbers refer to the code
state at review time.

---

## P0 — Real bugs

### 1. Hiscore double-recorded on death and victory
`check_death` (src/core/game.rs:738) and `check_victory` (src/core/game.rs:749) each call
`hiscore::record(self)`, then main.rs:93 and main.rs:96 call `hiscore::record(game)`
*again* after `do_turn` returns. Every death/victory writes two hiscore entries.
- Fix: remove the `hiscore::record` calls from main.rs (keep the in-game ones).

### 2. Dead autosave resurrects a dead game
main.rs:98 calls `save::autosave(game)` unconditionally after every turn, including the
death turn. `check_death` deletes the save (game.rs:737) and then main re-writes a save
with `alive = false`. Loading it yields an unplayable dead game (and the title screen
shows a stale "Continue" entry).
- Fix: only autosave when `game.alive && !game.won`.

### 3. `Monster::Deserialize` is a broken stub
src/entities/monster.rs:30-48 deserializes the whole struct as `IgnoredAny` and returns
a placeholder: `MONSTERS[0]` def, empty name, pos (0,0), **hp 0**. Loading any save
corrupts the entire monster list into dead placeholders at the origin.
- Fix: `#[derive(Serialize, Deserialize)]` on `Monster` (all fields are
  serde-compatible; `MonsterDef` needs `Deserialize` derived in src/data/monsters.rs).

### 4. RNG stream resets on save/load
src/core/rng.rs:20-32 serializes only the seed and reconstructs via `Rng::new(seed)`.
After loading, the RNG restarts from the beginning of the stream, so subsequent random
events replay the same prefix as the start of the run — save/resume is not faithful.
- Fix: enable the `serde` feature of `rand_chacha` in Cargo.toml and persist the
  `ChaCha8Rng` inner state instead of the seed.

### 5. `spawn_monster` ignores tier variety
src/entities/monster.rs:72-80 filters all defs for the tier, then takes `.first()`.
Only one monster species per tier ever spawns. `rng` is used only for the unique prefix.
- Fix: `self.rng.pick(&defs)`.

### 6. Player weapon deals no damage
src/combat.rs:33 — `player_attacks` rolls damage on the **monster's** die
(`m.def.damage_die`), not the player's weapon. The weapon only affects to-hit
(src/entities/player.rs:124-128 via `enchant`). AC also never affects hit chance, only
raw damage subtraction, for both sides.
- Fix: player rolls a weapon damage die (unarmed 1d4, weapon die + enchant flat bonus).

### 7. Player never levels up
`Player::gain_xp` (src/entities/player.rs:157) is never called. `kill_monster` does a
raw `self.player.xp += xp` (game.rs:325), so the level-up branch (max_hp/max_ep growth,
full heal) never fires. The player is permanently level 1.
- Fix: call `self.player.gain_xp(xp)` in `kill_monster` and log the level-up.

### 8. `take_off_item` ignores its slot
game.rs:473-477 — the `slot` parameter is `_slot`; the function unconditionally takes
off the armor slot. `Action::TakeOff` for a ring does the wrong thing (and the core
`player_turn` route `Action::TakeOff(s)` passes an *inventory* index, not a ring slot).
- Fix: match the item kind at `slot` and remove armor or the matching ring.

### 9. A\* heap ordering is inverted
src/map/path.rs:26-30 — `Node::Ord` compares `f` ascending, but `BinaryHeap` is a
max-heap, so the node with the **largest** f pops first. Paths still terminate and reach
the goal, but they are not optimal and the search degenerates on mazes.
- Fix: flip `Ord` (or use a min-heap) so minimum-f pops first.

### 10. Death cause and date are hardcoded
game.rs:768-769 — `score_info()` always returns `cause: DeathCause::Slain` and
`date: "1970-01-01"`. There is no tracking of the killing blow (monster vs. starvation
vs. poison) and no real date.
- Fix: track `last_damage_source` on `Game` (set in `attack_monster` / `monster_turns` /
  starvation path) and compute a real date.

### 11. Dead code in `magic.rs`; scrolls and most potions no-op
`magic::apply_potion_full` / `magic::apply_scroll_full` exist but are never called. The
live paths are game.rs:584 (`apply_potion`: only Healing/CurePoison work) and
game.rs:611 (`apply_scroll`: pure log line, **zero scroll effects** — no identify,
teleport, map, enchant, remove-curse). Also `magic.rs`'s own Teleport is a log-only
stub.
- Fix: route `quaff_item`/`read_scroll` through the `magic.rs` implementations (and
  implement real teleport), or delete the dead functions.

### 12. `App`'s `Game` import is unused; `amulet_taken` / `tombstones` are dead
- main.rs:7 `use deepdelve::core::game::Game;` — unused import (compiler warning).
- game.rs:66 `amulet_taken` — never set/read (pickup only sets `amulet_carried`).
- game.rs:67,71-77 `Tombstone` / `tombstones` — never created.
- Other compiler warnings from unused vars: `ignored` (monster.rs:33), `rng`
  (player.rs:37), `rarity` (items/loot.rs roll_drop).

---

## P1 — Plan features missing, stubbed, or unreachable

### 13. Item actions unreachable from the keyboard (biggest gap)
`App::handle_play_key` (src/ui/app.rs:99-142) binds only movement / wait / stairs /
pickup / panels. The core implements all item actions (game.rs:195-235: `UseItem`,
`Drop`, `Wield`, `Wear`, `TakeOff`, `RingOn`, `RingOff`, `Eat`, `Quaff`, `Read`,
`Identify`, `FireWand`) but **no key ever produces them**. The player can pick up
items but can never drink, eat, equip, drop, or read anything. The inventory panel
(src/ui/panels.rs) is display-only.
- Fix: key bindings (q=quaff, e=eat, d=drop, w=wield, r=read, z=wand, etc.) plus an
  item-selection picker for inventory slots.

### 14. Wand targeting unreachable
`App.targeting` (app.rs:49) is never set to `Some`; `handle_targeting_key` (app.rs:145)
is dead. No `z`/wand key exists, so `Action::FireWand` can never fire and all wands are
unusable.
- Fix: wand key → item picker → set `Targeting`, then existing arrow+Enter flow.

### 15. Quests can never progress
- No Offered→Active acceptance flow, no quest givers/NPCs.
- `QuestLog::on_kill` only advances quest 2 when it is already `Active`, so the kill
  counter can never start (src/quest.rs).
- `check_progress` is an empty hook (game.rs:266, 342 call it).
- No rewards (potions/XP/wands/legendaries per PLAN).
- Quest-specific items don't exist: no signet ring, no iron key, no sealed vault.
- Quest log is never rendered in any panel.
- The 3 quests are defined twice: `QuestLog::default` (quest.rs) and
  `data::quests::initial_quests()` (src/data/quests.rs) — pick one source of truth.

### 16. Shop is dead code
src/shop.rs — `buy_price` / `sell_price` / `buy` / `sell` are never called anywhere. No
merchant NPC, no shop UI, gold has no sink.

### 17. Audio is entirely dead code
src/audio/sfx.rs — `SfxEngine` is never instantiated; the rodio arm of `play()` is a
literal `// Real rodio playback would go here` no-op; all `synth.rs` functions are
uncalled. `App.muted` is toggled by `M` but nothing reads it.

### 18. Statuses have no mechanical effect
src/status.rs — confusion / paralysis / invisibility counters decrement but change no
behavior: no movement lockout, no AI reaction, no invisible flag for AI. Hunger is a
single stage (1 HP/turn at 0), not the planned 5-stage starving/weak/dying ladder. No
sickness / petrification / burn / slow exist.

### 19. Monster AI gaps
- No ranged attacks (the "archer" tier fights melee).
- No flee / investigate behaviors; `ability_cooldown` field exists but no abilities.
- Monster "sight" is a flat 40% random roll, not FOV-based.
- `is_boss` is never set; no boss monsters; no per-tier elite rolls besides the 25%
  unique prefix.

### 20. Map features missing
- No trap tiles or trap effects (6 trap types in PLAN).
- No doors / keys; any "closed door" tile is walkable.
- Water / lava / spore-gas tiles are never placed and would do nothing anyway.
- No D2 shop room / merchant NPC, no D15 boss arena.

### 21. Endless mode not implemented
`Game.endless` exists but is never set to `true`; `try_stairs_down` caps at
`MAX_LEVELS` (game.rs:357); victory ends the run. PLAN calls for optional endless
descent after picking up the Amulet.

### 22. main.rs infrastructure gaps
- No `--seed` / `--headless` / `--no-audio` CLI flags (all required by PLAN).
- No panic hook and no terminal-restore Drop guard — a panic leaves the terminal in
  raw mode on the alternate screen.
- Seed comes from the wall clock (main.rs:76-79), not reproducible from outside.

### 23. No integration tests, no CI
No `tests/` directory even though `lib.rs` exposes `tests_harness::new_game(seed)`.
PLAN requires suites: gen_invariants, combat, inventory, save_roundtrip, sim_game, plus
a CI workflow. `proptest` is a dev-dependency but unused.

### 24. No starting kit
Player starts with empty inventory, no weapon (src/entities/player.rs:115-118). PLAN
specifies class-appropriate starting gear.

---

## Suggested fix order
1. P0 bugs #1–#11 (small, local, unblock save/load and core loop).
2. P1 #13/#14 (keyboard + item picker + wand targeting) — makes the game actually playable.
3. P1 #7-level-ups is in P0; then #18 statuses, #19 AI, #20 map features, #15 quests,
   #16 shop, #17 audio, #21 endless, #22 main.rs, #23 tests/CI.
