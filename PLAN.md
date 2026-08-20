# Deepdelve — Terminal Roguelike in Rust (Development Plan)

A production-quality, console-only roguelike written in Rust using `crossterm`
(via `ratatui`). Turn-based, grid-based, permadeath, procedurally generated,
with real (procedurally synthesized) sound effects.

---

## 1. Goals

- A comprehensive, classic roguelike (NetHack / ADOM / Stone Soup lineage) playable entirely in a terminal.
- Hours of replayable gameplay: 25 themed dungeon levels with a victory goal, bosses, quests, shop, traps, status effects, and endless mode after victory.
- Production quality, not a prototype: polished menus, save/load, high scores, deterministic seeding, graceful terminal handling, full test suite, CI, and documentation.

### User-confirmed decisions

| Decision | Choice |
|---|---|
| Rendering stack | `ratatui` 0.30.2 on the official `crossterm` 0.29.0 backend |
| Sound | `rodio` 0.22.2 (cpal/ALSA) with **procedurally synthesized SFX** (no audio asset files); automatic fallback to the terminal bell when no audio device exists; `M` mutes |
| Game structure | Winnable crawl: 25 themed levels, victory item (Amulet of the Abyss) on D25, endless scaling beyond D25 |

---

## 2. Research: standard roguelike feature set

Reference points: the Berlin Interpretation (2008) of the "canon" roguelikes
(Rogue, NetHack, ADOM, Angband, DCSS) and the conventions of modern classics.

High-value factors (Berlin Interpretation) and our coverage:

| Feature | Coverage |
|---|---|
| Random dungeon generation | Rooms + corridors, cellular-automata cave sections, handcrafted vaults |
| Permadeath | Character death ends the run; tombstone + cause of death; save file deleted |
| Turn-based | One player action per turn; monsters and time advance in response |
| Grid-based / ASCII | 80×25 tile grid, ASCII glyphs + colors, first-person viewport |
| Non-modal | Every action available at all times; panels toggle, never force a menu |
| Complexity / emergent play | Cursed items, traps, status effects, quest chains, identification risk |
| Resource management | **Hunger system** (well-fed → starving → dying), limited potions/food, gold economy |
| Hack-and-slash | No diplomacy or negotiation; combat is the only interaction with monsters |
| Exploration + item discovery | Fog of war, remembered tiles, unidentified potions/scrolls/wands |

Low-value / classic extras we include:

- Race & class character creation with distinct stats and starting kits
- XP / experience levels, attribute growth
- Monster behaviors similar to the player: ranged attacks, item drops, casting, special abilities
- Tactical depth: FOV, bump-to-attack, doors, traps, ranged wands
- Numeric status interface: HP, EP, hunger, AC, to-hit, damage, gold
- Goal item on the deepest level (Amulet of the Abyss, D25)
- Score at death/victory and a persistent top-10 high-score file
- A shop (Moria-style town element) with merchant + identifying wizard
- Tombstones/grave marks from previous runs (late milestone)

---

## 3. Tech stack (all verified on this machine)

| Component | Version / facts |
|---|---|
| Toolchain | Rust 1.97.1 stable, x86_64-unknown-linux-gnu (edition 2024) |
| Terminal UI | `ratatui` 0.30.2 (default crossterm backend) |
| Terminal I/O | `crossterm` 0.29.0 (raw mode, events, resize) |
| Audio | `rodio` 0.22.2 with `default-features = false, features = ["playback"]`. Verified 0.22 API: `stream::DeviceSinkBuilder::open_default()` → `MixerDeviceSink` → `Mixer`; `buffer::SamplesBuffer` for procedural samples. ALSA 1.2.15.3 + dev libs confirmed present → real playback works here |
| RNG | `rand` 0.10.2 (`StdRng` + `SeedableRng`; serializable state → deterministic `--seed`) |
| Serialization | `serde` 1.0 + `serde_json` (save files, high scores) |
| Paths / errors | `dirs` (data dir), `thiserror` |
| Dev / QA | `proptest` (generator invariants), `clippy -D warnings`, `rustfmt`, GitHub Actions CI |
| Build feature | `audio` (default on) → CI/headless builds can compile with `--no-default-features` |

Audio fallback chain: rodio stream init failure → terminal BEL (`\x07`) for
critical cues + one-time "audio unavailable" note. Cargo feature `audio`
removes the rodio dependency entirely for headless/CI builds.

---

## 4. Architecture

**Core principle: the game core is 100% UI-agnostic and headless-testable.**
`core::Game::do_turn(action)` is the only way game state changes. The UI maps
terminal input → `Action`, renders `Game` state, and consumes `GameEvent`s
(the single hook for both audio and UI effects/animations).

```
src/
├── main.rs            # CLI (--seed, --headless, --no-audio), terminal bootstrap, app state machine
├── core/
│   ├── game.rs        # Game struct: floors, entities, turn pump, victory/death checks
│   ├── rng.rs         # seeded StdRng wrapper (SeedableRng), save-safe
│   ├── message.rs     # message log (ring buffer)
│   ├── score.rs       # score calculation (gold + XP + depth + quests)
│   └── events.rs      # GameEvent enum → audio + UI effects
├── map/
│   ├── level.rs       # Level: 80×25 grid, tiles, doors, stairs, traps, features
│   ├── fov.rs         # 3x3 shadowcasting field of view
│   ├── path.rs        # A* pathfinding for monsters
│   └── gen/
│       ├── mod.rs     # orchestration, theme by depth, guarantees
│       ├── rooms.rs   # rooms & corridors
│       ├── cellular.rs# cellular-automata cave sections (~15% of levels)
│       ├── vaults.rs  # handcrafted vault placement (shop, shrine, wizard, arena, amulet chamber)
│       └── decorate.rs# monsters/items/traps/doors/gold placement + connectivity repair
├── entities/
│   ├── entity.rs      # base: pos, glyph, color, HP, flags
│   ├── player.rs      # stats, inventory, equipment, hunger, XP, status
│   ├── monster.rs     # MonsterDef + instance (abilities, AI state, drops)
│   ├── ai.rs          # AI state machine
│   └── npc.rs         # shopkeeper, wizard, quest givers
├── items/
│   ├── item.rs        # Item, ItemKind, stacks, enchant, identification
│   ├── catalog.rs     # all item definitions
│   ├── loot.rs        # per-depth loot tables + rarity tiers
│   └── equip.rs       # slots, wield/wear rules, 2H vs shield
├── combat.rs          # hit calc, damage rolls, crits, death/XP
├── status.rs          # status effects + hunger states
├── magic.rs           # wand/potion/scroll effects, targeting
├── quest.rs           # quest definitions, state, rewards
├── shop.rs            # pricing, buy/sell
├── save.rs            # serde JSON save/load, autosave, permadeath delete
├── hiscore.rs         # persistent top-10
├── data/
│   ├── monsters.rs    # ~45 monster definitions
│   ├── classes.rs     # 5 classes × 4 races
│   ├── themes.rs      # 5 dungeon zone themes
│   └── quests.rs      # 3 quests
├── ui/
│   ├── app.rs         # input → Action mapping, panel management
│   ├── render.rs      # main frame: viewport + bars + status + messages
│   ├── menu.rs        # generic list menu (targeting, inventory ops)
│   ├── palette.rs     # per-theme color palettes
│   └── panels/        # inventory, character, help, history, quests,
│                      # shop, targeting, character-creation, hiscore
└── audio/
    ├── sfx.rs         # SfxEngine: Mixer on worker thread, voice pool, mute
    └── synth.rs       # oscillators, noise, envelopes, filters, pitch ramps
tests/
├── gen_invariants.rs  # proptest: connectivity, stairs reachable, no isolated tiles
├── combat.rs
├── inventory.rs
├── save_roundtrip.rs  # serialize → deserialize → state equality
└── sim_game.rs        # headless bot plays a full run (D5+ or death), asserts invariants
.github/workflows/ci.yml
README.md
```

### Turn pump (order of operations)

1. Player action resolves (move / bump-attack / item / wait).
2. Tile effects: trap triggers, stairs prompt (explicit confirm to change level).
3. Monster turns: each active monster acts per its AI state machine.
4. Time ticks: hunger drain, poison/disease damage, status durations, wand charge, random spawn timer.
5. FOV update (3x3 shadowcasting), remembered-tile refresh.
6. Death / victory checks → `GameEvent` emission (drives audio + UI).

---

## 5. Game design

### World structure
- **25 levels + endless**, grid 80×25 per level, grouped in 5 themed zones of 5 levels:
  1. **D1–5 The Barrow Halls** — earth tones; goblins, orcs, skeletons, bats, rats, hounds
  2. **D6–10 The Fungal Grottos** — green/purple; spore-gas patches, poison; fungi, ghouls, vampires, basilisk
  3. **D11–15 The Drowned Vaults** — blue; water pools (slow movement); ghosts, golems, lich, demons
  4. **D16–20 The Ember Works** — red/orange; lava pools (damage); elementals, hellhounds, dragons, golem lords
  5. **D21–25 The Abyss** — dark/purple; reduced FOV; ancient horrors, death knights, Abyss Lord
- Generation: rooms & corridors baseline; ~15% of levels include cellular-automata cave sections; handcrafted vaults placed per depth table (shop on D2, shrines, wizard tower D8–12, arena D15, amulet chamber D25).
- **Guarantees per level** (enforced in `decorate.rs` + property-tested): player start tile, down-stairs (and up-stairs on D>1), all placed features reachable from player start (auto-carve repair), 3–8 monsters scaled by depth, 0–3 vaults, gold/items/traps.
- **Bosses** (unique, named, multi-ability: summon adds, enrage <30% HP, special attack):
  - D5 Gorehorn the Troll King · D10 the Basilisk Matriarch · D15 the Lich of the Drowned
  - D20 the Ember Drake · D25 the Abyss Lord (guards the Amulet)
- **Victory**: take the Amulet of the Abyss from its D25 vault → victory screen with score → **endless mode**: D26+ generated on demand with scaling monster stats and loot.

### Player
- **6 attributes**: STR, DEX, CON, INT, WIS, CHA. Derived stats: to-hit, damage, AC, stealth, darkvision, magic power.
- **Races (4)**: Human (versatile +1 all), Elf (+DEX, +INT, +darkvision), Dwarf (+CON, +AC, fire resistance), Halfling (+DEX, +CON, +stealth, +crit luck).
- **Classes (5)**: Warrior, Thief (lockpicking, +stealth), Ranger, Mage, Cleric — distinct base stats, HP/EP, starting kits, kit-specific perks.
- **XP/leveling**: XP table (tunable in one place); level up grants +max HP, +max EP, and a player-chosen +1 attribute; no auto-heal.
- **Hunger**: 0–1200, drains over turns; states well-fed → hungry → starving (HP drain) → weak (-to-hit, no regen) → dying; restored by food; ring of Sustenance halts drain.
- **Death causes** tracked for tombstones ("slain by X", "starved", "poisoned", "drowned in lava"…).

### Monsters (~45 species, 5 tiers)
- Tier 1 (D1–5): bat, giant rat, cave slime, goblin, hobgoblin, orc, skeleton, zombie, cultist, hound, spider, snake, gnoll, slime
- Tier 2 (D6–10): fungus (splits on death), ghoul (drain), vampire, wraith, ogre, troll (regen), harpy, minotaur, basilisk (petrify), cockatrice, medusa, ghost
- Tier 3 (D11–15): demon, archer, wizard, fire/ice elemental, manticore, chimera, iron golem, lich
- Tier 4 (D16–20): hellhound, balrog, dragon (fire breath), stone colossus, vampire lord, demon lord, beholder (eye ray)
- Tier 5 (D21–25): abomination, ancient dragon, death knight, abyss horror, Abyss Lord (boss)
- **Abilities**: melee, ranged (arrows/balls), specials with cooldowns — drain, poison spit, petrify, paralyze, sleep, confusion, blink, summon, split, regen, enrage.
- **AI** (state machine): wander / investigate (hears noise) / chase (A*, remembers last seen player pos) / attack / flee; shooters keep range; ability cooldowns.
- **Uniques**: ~25% of levels spawn one unique (named prefix like *Foul* / *Ancient* / *Elder*, +50% stats, guaranteed named drop).
- **Spawning**: 1–3 random spawns per N turns (depth-scaled), only on unexplored floor tiles, capped per floor; XP awards with diminishing returns for level delta.

### Items (~65 templates)
- **Weapons (12)**: dagger, shortsword, longsword, greatsword, battle axe, war hammer, mace, flail, spear, trident, morning star, war flail (1H/2H split).
- **Armor (8)**: chain mail, plate; leather/iron helm; leather/plate gloves; leather/iron boots. **Shields (4)**: small, large, tower.
- **Wands (11)**: fire bolt, lightning, healing, cure poison, sleep, confusion, paralyze, blink, teleport control, monster removal, magic mapping.
- **Potions (9)**: healing (small/large/super), cure poison, restore, identify, invisibility, energy, antidote (fake), mutation.
- **Scrolls (9)**: identify, teleport, enchant weapon, enchant armor, remove curse, mapping, god's message (quest hint), opening, fear.
- **Rings (6)**: protection, energy, stealth, infravision, sustenance, poison resistance.
- **Food (6)**: trail rations, apple, mushroom, steak, energy drink, (rare) potion-flavored candy.
- **Gold** (unlimited, part of score) and the **Amulet of the Abyss** (victory).
- **Random magic**: enchantment +0..+5 (attack/defense), special effects (flame, lightning, poison, sleep, fear, blink), **cursed** flag (can't remove without scroll, -1 penalty).
- **Identification**: items unknown until identified — by using them, scroll of identify, wizard NPC, or passive chance by level/INT.
- **Loot**: per-depth tables with rarity tiers (common/uncommon/rare/legendary); uniques/bosses drop named, high-enchant gear.

### Status effects
Poison (dmg/tick → disease chance), disease (stacking), sleep, confusion (random movement), paralysis, blessed (resist hostile teleport/confusion), cursed, invisible (monsters lose aggro). All have tick durations, messages, and SFX.

### Quests (3, quest log panel, completion SFX)
1. **The Lost Signet** (D1): an old man seeks his signet ring, lost with a named guard on D2. Reward: potion + XP.
2. **Blood on the Altar** (D7–9): kill 4 Cultists of the Abyss. Reward: wand + XP.
3. **The Sealed Chamber** (D13–14): recover the Iron Key, open a sealed vault → legendary item + gold.

### Shop (D2 vault, "The Wandering Emporium")
- **Merchant**: buy (125% of value) / sell (80%) — inventory-driven, price by item value + enchant + rarity.
- **Wizard**: identifies items for gold.

### Traps (6) & doors
Arrow, dart, falling item (takes a random inventory item), teleport, sleep gas, acid pool. Doors: open/locked, keys, thief lockpicking skill.

### Combat model
- **To-hit**: base 50% + 2%/DEX + weapon-level bonus, reduced by target AC; d100 roll.
- **Damage**: weapon die (d4–d12) + STR bonus; crit (5% + luck) doubles.
- **AC**: 10 + DEX/2 + armor + rings.
- Bump-to-attack; monster death → XP + drop roll.

---

## 6. UI/UX

### Layout (minimum terminal 80×24)
```
┌─ VIEWPORT (80×21, centered) ────────────────────────────────────┐
│  bright = in-FOV, dim = remembered, blank = unexplored         │
├─────────────────────────────────────────────────────────────────┤
│ HP [██████████░░] 45/45  EP [███░░░] 12/25  FU ████████ 840    │
│ @ Aldric the Bold, Lv 5 Warrior, D3: The Fungal Grottos  $127  │
├─────────────────────────────────────────────────────────────────┤
│ > A goblin stabs you!                                          │
│ > You feel a presence in the dark…                             │
└─────────────────────────────────────────────────────────────────┘
```
- HP bar color red→green; EP blue; hunger yellow→red.
- Per-theme palette shifts for viewport tiles; monsters color-coded by species.

### Keymap (classic)
| Key | Action |
|---|---|
| `hjkl` / arrows / numpad | move (bump-to-attack) |
| `.` | wait |
| `>` / `<` | stairs down / up (confirm prompt) |
| `g` pickup · `d` drop · `i` inventory |
| `u` use (wand/potion/scroll) · `e` eat · `q` quaff · `r` read |
| `w` wield · `W` wear · `x` take off · `o`/`O` ring on/off |
| `s` shop (near merchant) |
| `c` character sheet · `H` message history · `m` minimap |
| `?` help · `M` mute · `X` score summary |
| `Q` save & quit (confirm) · `Ctrl-C` abort without save · `Esc` close |

Wands needing a target open a **cursor targeting mode** (move cursor, Enter fire, Esc cancel).

### Screens & state machine
`title → character creation (name → race → class, stat previews) → play → death | victory`
- **Title**: New Game / Load Game / High Scores / Quit.
- **Death screen**: tombstone, cause of death, stats (level, depth, gold, XP, kills, turns, time), high-score entry.
- **Victory screen**: score breakdown, hiscore entry, then endless-continue or new run.
- Panels: inventory, character, help, history, quest log, shop, targeting, hiscore, creation.
- Terminal hygiene: alternate screen, raw mode, cursor hidden; restored on clean exit **and** panic (Drop guard + panic hook); resize re-centers viewport, ratatui re-lays-out.

---

## 7. Audio (procedural — zero asset files)

`SfxEngine`: lazy-init rodio `Mixer` on a worker thread; on init failure → BEL fallback.
`synth.rs` builds `SamplesBuffer`s (44.1 kHz, f32, mono): sine/square/saw oscillators, white noise, ADSR envelopes, simple filters, pitch ramps. ~8-voice pool, master gain, `M` mute.

SFX set (~20):
footstep, hit, miss, crit, monster death, player death, pickup, equip, quaff, eat,
wand zap, potion splash, scroll shimmer, level-up arpeggio, stairs chord, door,
trap buzz, quest fanfare, victory fanfare, coin, teleport whoosh.

---

## 8. Production quality

- **Save/load**: full `Game` state as serde JSON (all generated floors, entities, inventory, quests, RNG state, message log) in `~/.local/share/deepdelve/saves/`. Autosave on level change + on `Q`. **Save deleted on death** (true permadeath). Versioned schema field for forward compatibility.
- **Determinism**: all randomness flows through one injectable `StdRng`; `--seed` reproduces a run exactly; seed recorded in high scores.
- **High scores**: top-10 JSON — name, class, race, depth, gold, score, turns, seed, date.
- **CLI**: `--seed <u64>`, `--headless` (run turns without UI; for tests/debug), `--no-audio`.
- **Testing**:
  - Unit: FOV vs brute-force LOS reference, A* correctness, combat math, inventory/equip ops, status tick timing, save round-trip state equality.
  - Property (proptest): generated levels always connected, stairs reachable, no isolated tiles, vaults don't overlap, monster counts in range.
  - Integration (`sim_game`): headless bot plays a full run (to D5+ or death) asserting no panics and invariants hold — also the balance-tuning harness.
- **CI** (GitHub Actions): `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test --no-default-features` (headless, no ALSA needed on runners).
- **Error handling**: no `unwrap` in game paths; `thiserror` error types; terminal always restored; resize-safe.
- **README**: features, install, controls, architecture notes, balance knobs.

---

## 9. Milestones (each ends with clippy + tests green)

| # | Milestone | Deliverable |
|---|---|---|
| M0 | Scaffold | Cargo project + deps, terminal bootstrap (ratatui+crossterm), app state machine, empty map render, input loop, panic/exit hygiene |
| M1 | Core loop | Level gen (rooms/corridors + guarantees), FOV, movement, bump-attack, monster AI (chase/attack), message log, HP, death, status bars, SFX engine + first 5 sounds |
| M2 | Items & combat | Full item catalog, inventory/equip (2H vs shield), weapon/armor combat with AC, gold, loot tables, identification, hunger + food |
| M3 | Magic & progression | Wands/potions/scrolls/rings + targeting, EP, XP/leveling, all status effects, race/class creation, character sheet |
| M4 | Depth | Themes D1–25, caves, vaults, uniques/bosses, traps, doors/keys, shop+wizard, quests, Amulet victory + endless mode, hiscore, save/load, tombstones |
| M5 | Polish & production | Full SFX set, help screen, minimap, death/victory screens, hiscore UI, clippy/fmt clean, proptest + sim integration tests, CI workflow, README, balance pass via sim bot |

---

## 10. Risks & mitigations

| Risk | Mitigation |
|---|---|
| rodio fails on audio-less machines/CI | Init-failure → BEL fallback; `audio` cargo feature for headless builds |
| Generator produces unreachable stairs | Connectivity verification + auto-carve repair in `decorate.rs`; proptest coverage |
| Balance too easy/hard | Headless sim bot as tuning harness; all balance numbers in `data/` tables in one place |
| Scope creep | Milestones are independent shippable states; M2–M5 each add a full feature layer over an already-working game |
| Terminal left in raw mode on crash | Drop guard + panic hook restore terminal |

---

## 11. Estimated size

~60–70 Rust source files, ~12–18k LOC including tests and data tables.
