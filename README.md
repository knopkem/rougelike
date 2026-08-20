# Local LLM Game Dev

Note: This is an experience with local LLM (qwen3.8-27B) on minimal hardware: GPU with 16GB VRAM only.

Baseline: IQ3-XXS model, q8_0 KV, 96K context

System:

- Ryzen 5700 32GB RAM
- AMD 6800 XT 16GB VRAM
- Ubuntu

Harness:

- llama-cpp-turboquant
- opencode
- mattpocock/skills

## The Game

A terminal roguelike written in Rust. Descend 25 floors of a procedurally
generated dungeon, fight monsters, collect loot, and raise the
**Amulet of the Abyss** to win.

## Building & Running

```sh
cargo run
```

(Requires Rust toolchain. Dependencies: ratatui, crossterm, rand, serde,
dirs.)

## The Goal

The Amulet of the Abyss lies on **depth 25**. Find it, pick it up, and win.
Deeper floors hold stronger monsters and better loot.

### Dungeon zones

| Depths | Zone              |
|--------|-------------------|
| 1-5    | The Barrow Halls  |
| 6-10   | The Fungal Grottos|
| 11-15  | The Drowned Vaults|
| 16-20  | The Ember Works   |
| 21+    | The Abyss         |

## Character Creation

Choose a race (Human, Elf, Dwarf, Halfling) and a class
(Warrior, Thief, Ranger, Mage, Cleric) on the creation screen.

## Controls

| Key            | Action                    |
|----------------|---------------------------|
| `hjkl` / arrows / `yubn` | Move / attack (bump to attack) |
| `.` or `5`     | Wait a turn               |
| `>` / `<`      | Descend / ascend stairs   |
| `g`            | Pick up item (gold is picked up automatically when you walk over it) |
| `i`            | Inventory                 |
| `c`            | Character sheet           |
| `H`            | Message history           |
| `?`            | Help                      |
| `M`            | Mute sound                |
| `q` / `Q`      | Quit — works from any screen (play, death, victory). During play your last autosave is kept, so `L` on the title screen resumes it |
| `Esc`          | Close panel / cancel targeting |

Using a wand puts you in **targeting mode**: move with `hjkl`/arrows,
fire with `Enter`, cancel with `Esc`.

## Systems

- **Combat** — bump into monsters to attack. Monsters see you and come
  hunting; ranged foes (archers, hellhounds) keep their distance.
- **Loot** — weapons, armor, wands, potions, scrolls, rings, food, and
  gold. Gold is collected by walking over it; other items need `g`.
  Monsters have a chance to drop loot when killed.
- **Hunger** — your hunger meter drops each turn. Keep it fed: energy
  (EP) only regenerates while well-fed.
- **Statuses** — poison, paralysis, etc. show on the character sheet.
- **Shop** — some vaults hold a shop where you can buy and sell.
- **Quests** — take on quests (The Lost Signet, Blood on the Altar,
  The Sealed Chamber) for score bonuses.
- **Score** — gold + XP + depth + quest bonuses, recorded on a hiscore
  table when you die or win.
- **Autosave** — the game autosaves each turn to
  `~/.local/share/deepdelve/saves/autosave.json` (or the platform
  data dir). The title screen offers `L` to continue a saved game.

## Monsters

From cave slimes and goblins up through vampires, trolls, liches, and
balrogs — down to the ancient dragons, death knights, and the Abyss
Lord on the deepest floors.

## Project Layout

```
src/
  main.rs         event loop, screens
  core/           game state, turns, messages, RNG, score
  entities/       player, monster (AI), events
  items/          catalog, loot tables, equipment
  map/            level, generation, FOV, A* pathfinding
  quest.rs, shop.rs, save.rs, hiscore.rs
  ui/             ratatui rendering, panels, menu, app state
```

## Tests

```sh
cargo test
```
