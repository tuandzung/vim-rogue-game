# vim-rogue Onboarding Guide

**Languages:** Rust (2024 edition), TOML, YAML, Markdown
**Frameworks:** bracket-lib 0.8.7 (roguelike rendering), GitHub Actions (CI/CD)
**Quick start:** `cargo run` to play, `cargo test` to run 396 tests

## What Is This Project

vim-rogue is a roguelike dungeon game that teaches Vim motions through gameplay. Players navigate four 80x40 dungeon levels using real Vim keybindings (`hjkl`, `wb`, `G/gg`, `0$`, `ft`, `dd`), dodge FOV-aware enemies, and manage lives/HP while progressing through zones that gate new motion mechanics.

## Architecture Layers

The codebase follows a clean layered design. Read bottom-up (dependencies flow upward):

```
┌─────────────────────────────────────────────────────┐
│  Entry Points     main.rs, lib.rs, test_support.rs  │
├─────────────────────────────────────────────────────┤
│  Application      game.rs (App coordinator)         │
├─────────────────────────────────────────────────────┤
│  Game Logic       map.rs, player.rs, enemy.rs,      │
│                   visibility.rs                      │
├─────────────────────────────────────────────────────┤
│  Domain Core      types.rs (aggregates, enums)      │
├─────────────────────────────────────────────────────┤
│  Presentation     renderer.rs (read-only display)   │
├─────────────────────────────────────────────────────┤
│  System Support   animation.rs, audio.rs            │
├─────────────────────────────────────────────────────┤
│  Tests            tests/ (396 integration tests)     │
└─────────────────────────────────────────────────────┘
```

### Domain Core (`types.rs`)
Core type definitions for the entire game. Everything depends on this file.
- **Primitives:** `Position`, `Tile` (Floor/Wall/Obstacle/Exit/Torchlight), `Zone`, `Direction`
- **VimMotion:** 13-variant enum covering all Vim keys
- **GameState:** State machine (Title/Playing/Paused/Won/Lost)
- **Aggregates:** `World` (terrain, visibility, enemies), `PlayerState` (position, motions, HP, trail), `InputState` (Vim key buffering for f/t/dd/gg), `Session` (lifecycle, timing, pause)
- **App:** Thin coordinator holding all aggregates — no business logic itself

### Game Logic
| File | Role | Key Functions |
|------|------|---------------|
| `map.rs` | 80x40 grid, 4 levels, 5 zones | `carve_level`, `build_level_2/3/4` |
| `player.rs` | 13 Vim motion handlers | `handle_motion` (takes `&mut Map` for dd) |
| `enemy.rs` | FOV-aware BFS chase + patrol | `step_toward_player`, `has_line_of_sight` |
| `visibility.rs` | Ray-casting FOV | `compute_fov`, Hidden/Explored/Visible states |

### Application (`game.rs`)
The App coordinator — highest fan-out in the codebase (calls into player, enemy, visibility, animation, audio). Orchestrates:
- `handle_key` — input processing with pending state for two-key motions (f/t/dd/gg)
- `tick` — animation timer updates
- `execute_motion` — delegates to `PlayerState::handle_motion`
- `enemies_step` — collision outcomes, damage, animation queueing, audio
- Level transitions, pause/resume state machine

### Presentation (`renderer.rs`)
**Read-only** — never mutates `App`. Pure function of game state. Draws:
- Title screen with ASCII art
- Game viewport with zone-colored walls and FOV fog
- Sidebar (HP bar, lives, discovered motions, level)
- Minimap of explored areas
- Win/loss/pause overlay screens

### System Support
- **`animation.rs`:** `GameClock` trait (`RealClock`/`TestClock`), easing interpolators, `PLAYER_MOVE_MS=150`, `ENEMY_MOVE_MS=200`
- **`audio.rs`:** `AudioManager` with `SoundEffect` enum, disabled by default, graceful no-op fallback

## Key Concepts

### Aggregate Pattern (DDD)
`App` is decomposed into domain aggregates — each owns its state and logic:
- **World** owns terrain, visibility, enemies (`step_enemies`, `push_enemies_off_position`)
- **PlayerState** owns position, motions, HP, trail (`handle_motion`)
- **InputState** owns pending Vim key buffering
- **Session** owns lifecycle, timing, pause state
- **App** coordinates cross-aggregate flows (level transitions, collision → damage)

### Deterministic Timing
`GameClock` trait with `RealClock` (production) and `TestClock` (tests). All time-dependent code uses this trait, making animations and timers testable without wall-clock delays.

### Renderer Isolation
`renderer.rs` never mutates `App`. This architectural boundary ensures display code cannot corrupt game state. Renderer internals are `pub` for test access.

### Test Seam
`App::for_test(map, position)` in `test_support.rs` creates an `App` with minimal state. All 396 integration tests use this constructor. Shared helpers in `tests/common/mod.rs`.

### Zone-Gated Motions
5 zones gate progressively harder Vim motions:
| Zone | Motions | Color |
|------|---------|-------|
| 1 | h, j, k, l | Gray |
| 2 | w, b | Cyan |
| 3 | 0, $, G, gg | Magenta |
| 4 | f\<char\>, t\<char\> | Red |
| 5 | dd | Gold |

### Enemy AI
Two behaviors:
1. **Chase:** When player visible (Bresenham LOS within FOV radius 8), BFS shortest path toward player
2. **Patrol:** When player hidden, move within assigned `PatrolArea`

Level 4 enemies have `hp: Some(30)` — 3 melee hits (x key, 10 dmg each) to kill.

### Level Design
| Level | Layout | Enemies | Special |
|-------|--------|---------|---------|
| 1 | Basic corridors + obstacles | None | Zone-gated areas |
| 2 | Inverted maze, obstacles at gaps | None | dd destroys obstacles |
| 3 | Zigzag descent | BFS chase enemies | First enemy encounters |
| 4 | Fortress rooms | FOV patrol enemies w/ HP | Melee combat, torchlight checkpoints |

## Guided Tour

Follow these steps in order to understand the codebase:

### Step 1: Read the README
Start with `README.md` for features, motions reference, and build instructions.

### Step 2: Entry Point (`src/main.rs`)
~44 lines. Initializes bracket-lib `BTerm`, creates `App`, runs event loop. Delegates to lib.rs for actual logic.

### Step 3: Module Exports (`src/lib.rs`)
~12 lines. Barrel file re-exporting all modules with `pub use`. Creates the public API surface.

### Step 4: Domain Types (`src/types.rs`)
691 lines. Read the struct/enum definitions to understand the domain model. Start with `VimMotion`, `GameState`, then the aggregates (`World`, `PlayerState`, `InputState`, `Session`, `App`).

### Step 5: Map Generation (`src/map.rs`)
471 lines. `carve_level` builds the base grid, `build_level_2/3/4` add level-specific features. Note `grid[y][x]` row-major indexing.

### Step 6: Player Motions (`src/player.rs`)
243 lines. `handle_motion` dispatches to motion-specific handlers. Note `&mut Map` parameter — `dd` destroys obstacles.

### Step 7: Enemy AI + FOV (`src/enemy.rs`, `src/visibility.rs`)
`enemy.rs` (180 lines): BFS chase + patrol. `visibility.rs` (124 lines): `compute_fov` with ray-casting.

### Step 8: Game Coordinator (`src/game.rs`)
647 lines. The orchestrator — highest complexity in the codebase. Trace `handle_key` → `execute_motion` → `enemies_step` for the main gameplay loop.

### Step 9: Renderer (`src/renderer.rs`)
914 lines. Largest file but read-only — safe to skim. Look at `render` (entry point), `render_map_viewport`, `render_sidebar`.

### Step 10: Test Infrastructure (`src/test_support.rs`, `tests/common/mod.rs`)
`test_support.rs` (28 lines): `App::for_test` constructor. `tests/common/mod.rs` (50 lines): shared helpers (`test_map`, `tick_timer`, `approx_eq`).

## File Map

### Source Code (`src/`)
| File | Lines | Complexity | Purpose |
|------|-------|------------|---------|
| `main.rs` | 44 | Simple | Binary entry, bracket-lib setup |
| `lib.rs` | 12 | Simple | Module re-exports |
| `types.rs` | 691 | **Complex** | Domain types, aggregates |
| `game.rs` | 647 | **Complex** | App coordinator |
| `renderer.rs` | 914 | **Complex** | Pure rendering |
| `map.rs` | 471 | **Complex** | Map generation |
| `player.rs` | 243 | Moderate | Vim motion handlers |
| `enemy.rs` | 180 | Moderate | Enemy AI (BFS + patrol) |
| `animation.rs` | 182 | Moderate | Timing, easing |
| `visibility.rs` | 124 | Moderate | FOV ray-casting |
| `audio.rs` | 55 | Simple | Sound effects |
| `test_support.rs` | 28 | Simple | Test constructor |

### Tests (`tests/`)
| File | Tests | Purpose |
|------|-------|---------|
| `game.rs` | 140 | Game state machine, all flows |
| `renderer.rs` | 53 | Rendering, colors, viewport |
| `map.rs` | 46 | Map structure, levels, zones |
| `player.rs` | 32 | All 13 Vim motions |
| `animation.rs` | 34 | Timers, interpolation |
| `visibility.rs` | 29 | FOV, explored states |
| `enemy.rs` | 21 | BFS, patrol, LOS |
| `types.rs` | 25 | Enums, structs |
| `audio.rs` | 16 | Audio lifecycle |
| `common/mod.rs` | — | Shared helpers |

### Configuration & CI
| File | Purpose |
|------|---------|
| `Cargo.toml` | Package manifest (edition 2024, bracket-lib 0.8.7) |
| `Cross.toml` | Cross-compilation (Linux x86_64, ARM64) |
| `rustfmt.toml` | Formatter config (max small heuristics) |
| `.github/workflows/ci.yml` | CI: fmt, clippy, test, cross-platform build |
| `.github/workflows/release.yml` | Release: cross-platform binaries + GitHub releases |

## Complexity Hotspots

Approach these files with extra care — they have the highest complexity:

1. **`src/renderer.rs`** (914 lines) — Largest file. Many rendering paths (title, game, sidebar, minimap, win, loss, pause). Changes here should maintain the read-only invariant.
2. **`src/types.rs`** (691 lines) — Central type definitions. Changes ripple through all modules. Maintain aggregate boundaries.
3. **`src/game.rs`** (647 lines) — Highest fan-out. Cross-aggregate coordination. Changes here affect the entire gameplay loop.
4. **`src/map.rs`** (471 lines) — Procedural generation. Level-specific logic in separate build functions. `grid[y][x]` row-major — bounds-check before access.
5. **`tests/game.rs`** (2015 lines) — Largest test file. 140 tests covering the full state machine.

## Development Workflow

```bash
cargo fmt              # Format (uses rustfmt.toml)
cargo fmt --check      # Check formatting
cargo clippy           # Lint
cargo test             # Run 396 integration tests
cargo build            # Compile
cargo run              # Play the game (opens graphical window)
```

**Verification checklist** — run all of these after any change:
1. `cargo fmt --check` — formatting clean
2. `cargo clippy` — zero warnings
3. `cargo test` — all 396 pass
4. Update `CHANGELOG.md` — add entry under `[Unreleased]`

## Conventions

- Rust edition 2024 with `use_small_heuristics = "Max"`
- `grid[y][x]` row-major — always bounds-check before access
- `is_passable` = `Tile::Floor`, `Tile::Exit`, `Tile::Torchlight`
- `renderer.rs` never mutates `App`
- `unwrap()`/`expect()` OK in non-test source
- All tests use `App::for_test(map, pos)` — no other test constructors
- FOV radius: player `FOV_RADIUS` (types.rs), enemies `ENEMY_FOV_RADIUS = 8`
- Animation: player 150ms, enemy 200ms
- Lives: 3. HP: MAX_HP=30. Melee: 10 dmg per hit
