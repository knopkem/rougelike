# Deep Delve — V100 attempt

> **Attempt 2** in the [Local LLM Game Dev](../README.md) series.
> Built from scratch by **qwen3.8-27B** (Q4_K_M, DFlash2, 262K context) on a
> **NVIDIA V100 32GB** with an **Intel i9-14900KF** under **Windows 11**.

A production-quality, console-only roguelike written in Rust. Descend through
five themed zones of the deep, fight monsters, manage your inventory, complete
quests, and survive as long as you can.

## Features

- **Pure core, headless-testable.** The game logic (`deepdelve::core::Game`)
  is 100% UI-agnostic. `Game::do_turn(action)` is the *only* way state changes,
  and it returns `GameEvent`s — the single hook for audio and UI effects. The
  terminal UI (`deepdelve::ui`) is a thin layer that maps keys to `Action`s,
  renders state, and consumes events.
- **Deterministic.** All randomness flows through one injectable `Rng`
  (ChaCha12). `--seed <u64>` reproduces a run exactly.
- **Procedural levels.** Rooms, corridors, traps, items, monsters, NPCs, and
  stairs are generated per depth, with a theme per zone.
- **Combat, items, equipment, magic, quests, shops, status effects, hunger.**
- **Save / load.** Versioned JSON saves in the platform data directory
  (`~/.local/share/deepdelve/saves/` on Linux, equivalent elsewhere). Saves are
  deleted on death (permadeath).
- **High-score table** persisted across runs.
- **Optional audio** (synthesized SFX via `rodio`), behind the `audio` feature
  (on by default). Build with `--no-default-features` for a headless/CI build.

## Requirements

- Rust stable (edition 2024).
- A terminal that supports the alternate screen buffer (any modern terminal).

## Build & Run

```sh
# Debug build (with audio)
cargo run

# Reproducible run
cargo run -- --seed 12345

# Headless build (no audio) — used by CI
cargo build --no-default-features
```

### CLI

```
deepdelve [OPTIONS]

Options:
  --seed <u64>   Use a fixed seed for a reproducible run
  --no-audio     Disable audio (accepted; build with --no-default-features to truly disable)
  -h, --help     Show this help
```

### Controls

| Action            | Keys                                  |
| ----------------- | ------------------------------------- |
| Move              | Arrows or `h j k l` (diagonals `u y b n`) |
| Wait              | `.` or `space`                        |
| Stairs down / up  | `>` / `<`                             |
| Pickup            | `,` or `g`                            |
| Inventory         | `i`                                   |
| Help              | `?`                                   |
| Save & quit       | `S`                                   |
| Abort             | `Q`                                   |

## The Game

- **Races:** Human, Elf, Dwarf, Halfling — each with distinct attribute
  bonuses, AC, darkvision, stealth, and crit traits.
- **Classes:** Warrior, Thief, Ranger, Mage, Cleric — each with a starting kit
  and base HP/EP.
- **Zones** (by depth):
  | Depth | Zone |
  | ----- | ---- |
  | 1–5   | The Barrow Halls |
  | 6–10  | The Fungal Grottos |
  | 11–15 | The Drowned Vaults |
  | 16–20 | The Ember Works |
  | 21+   | The Abyss |
- Descend to **D25** and beyond to enter endless mode.

## Architecture

```
src/
  core/        # UI-agnostic game state: Game, Action, GameEvent, Rng, Pos, score, message
  map/         # Level, Tile, generation, FOV (DDA line-of-sight), A* pathfinding
  entities/    # Player, Monster, NPC, AI
  items/       # ItemDef, catalog, inventory, equipment, loot
  data/        # Static data: classes, races, monsters, themes, quests
  combat.rs    # Melee/ranged resolution
  magic.rs     # Spells
  quest.rs     # Quest log
  shop.rs      # Shop
  status.rs    # Status effects
  save.rs      # Versioned JSON save/load
  hiscore.rs   # High-score table
  audio/       # Optional synthesized SFX (feature "audio")
  ui/          # ratatui + crossterm terminal UI (App, render, menu, palette)
  main.rs      # CLI entry point
```

Key invariants:

- `Game::do_turn` is the only mutation path.
- `GameEvent` is the only side-channel (audio + UI effects).
- The grid is a flat `Vec<Tile>` indexed `y * WIDTH + x` (`WIDTH = 80`,
  `HEIGHT = 25`).
- No `unwrap`/`expect` on player-reachable game paths; errors use `thiserror`.

## Testing

The core is fully testable headlessly. There are 149 unit tests plus 24
integration tests (generation invariants, combat, inventory, save round-trip,
and a long headless simulation).

```sh
cargo test                      # with audio feature
cargo test --no-default-features  # headless (CI)
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

## CI

`.github/workflows/ci.yml` runs on push/PR across Ubuntu, Windows, and macOS:
build + test (no audio), clippy (`-D warnings`), and `cargo fmt --check`.

## License

MIT
