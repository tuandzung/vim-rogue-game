<!-- Generated: 2026-04-17 | Updated: 2026-05-01 -->
<!-- Commit: HEAD | Branch: refactor/enemy-turn-into-world -->

# vim-rogue

Roguelike dungeon game (Rust + bracket-lib). Teaches Vim motions through gameplay. 80×40 dungeon, 4 levels, 5 zones, 13 Vim keys, FOV-aware enemy AI + patrol, fog-of-war.

## Structure
```
vim-rogue/
├── src/          # Application source code (see src/AGENTS.md)
├── tests/        # Integration tests — 396 tests across 9 files (see tests/AGENTS.md)
├── examples/     # Spike/prototype code (spike.rs)
├── resources/    # CP437 font sprite sheets (PNG) for bracket-lib rendering
├── Cargo.toml    # Edition 2024, deps: anyhow (unused), bracket-lib
├── Cargo.lock
├── README.md
└── .gitignore    # target/, *.rs.bk, *.pdb, mutants.out*/, .omc/
```

## Architecture
```
main.rs       → bracket-lib setup + event loop (44 lines)
game.rs       → App coordinator — thin enemies_step coordinator (collision outcomes, animation, audio), sequences cross-aggregate flows: level transitions, pause/resume (647 lines)
player.rs     → PlayerState impl — 13 motions + motion tracking (260 lines)
map.rs        → 80×40 grid, 5 zones, 4 dungeon levels, corridor carving, enemy spawns + patrol areas (471 lines)
renderer.rs   → bracket-lib rendering: title, viewport, sidebar, minimap, win/loss screens, ASCII art (914 lines)
types.rs      → Position, Tile, Zone, VimMotion, Direction, Enemy, PatrolArea, EnemyMovement, EnemyTurn, PlayerState, App + 3 aggregates (World owns step_enemies + push_enemies_off_position, InputState, Session), RenderGrid, ViewModel (691 lines)
animation.rs  → GameClock, AnimationState, AnimationTimer, Interpolator (easing) (182 lines)
visibility.rs → VisibilityMap with FOV (explored/visible/hidden states) (124 lines)
enemy.rs      → Enemy struct with FOV-aware BFS chase + room patrol (180 lines)
audio.rs      → AudioManager with SoundEffect enum, graceful fallback (55 lines)
lib.rs        → Re-exports all modules (9 lines)
```

## Where To Look
| Task | Location | Notes |
|------|----------|-------|
| Add new Vim motion | `src/player.rs` + `src/types.rs` (VimMotion enum) | handle_motion on PlayerState; Update `game.rs` parse_motion too |
| Change dungeon layout | `src/map.rs` (carve_level, build_level_2/3/4) | `grid[y][x]` row-major; 4 levels |
| Add UI elements | `src/renderer.rs` | Display only — never mutates state |
| Change game flow | `src/game.rs` (handle_key, tick, execute_motion) | Two-phase input for f/t/dd/gg; ESC/q = pause |
| Change pause menu | `src/game.rs` + `src/renderer.rs` + `src/types.rs` | GameState::Paused, PauseOption, render_pause_overlay |
| Add new types | `src/types.rs` | All modules use `crate::types::*` |
| Change enemy AI | `src/enemy.rs` + `src/types.rs` (World::step_enemies) | FOV-gated BFS chase + patrol in enemy.rs; turn orchestration in World |
| Change FOV/visibility | `src/visibility.rs` (compute_fov) | Hidden/Explored/Visible states |
| Change aggregate logic | `src/types.rs` (World, InputState, Session) + `src/player.rs` (PlayerState) | PlayerState flat struct; each aggregate owns its domain; App coordinates |
| Add animations | `src/animation.rs` | AnimationState + Interpolator; GameClock trait |
| Add sound effects | `src/audio.rs` (SoundEffect enum + AudioManager) | Disabled by default |
| Fix bug | `tests/` (396 integration tests, 9 files) | main.rs + lib.rs have no tests |

## Conventions
- Rust edition 2024. `rustfmt.toml`: `use_small_heuristics = "Max"`, `edition = "2024"`.
- 396 integration tests in `tests/` (9 files). Shared helpers in `tests/common/mod.rs`.
- Helpers: `test_map()`, `started_app_with_map()`, `test_app()`, `assert_approx_eq()`, `approx_eq()`, `tick_timer()`, `tick_state()`.
- `renderer.rs` internals `pub` for test access (e.g. `screen_meets_minimum_size`, `phase_definitions`, `exit_glow`).
- `lib.rs` re-exports all. `main.rs` thin (~32 lines).
- `is_passable` = `Tile::Floor`, `Tile::Exit`, `Tile::Torchlight`. `Tile::Obstacle` not passable but `dd` destroys it.
- w/b: horizontal scan, stop at non-passable. G/gg: vertical scan, stop at non-passable.
- `renderer.rs` read-only — never mutates App.
- `PlayerState::handle_motion` takes `&mut Map` (dd deletes obstacles). Owns motion tracking (motion_count, discovered_motions).
- `GameClock` trait: `RealClock` prod, `TestClock` tests.
- Animations: player 150ms (`PLAYER_MOVE_MS`), enemy 200ms (`ENEMY_MOVE_MS`).
- FOV: `FOV_RADIUS` in types.rs. Enemy FOV: `ENEMY_FOV_RADIUS = 8`.
- Enemy AI: BFS chase when player visible (`has_line_of_sight`), patrol in `PatrolArea` otherwise.
- Level 4: room-based patrol; no-torchlight rooms have ≥2 enemies.
- Audio off by default; `AudioManager::enable()` to turn on.
- Errors: `BError` from bracket-lib in main.rs. `anyhow` in Cargo.toml but unused.
- `unwrap()`/`expect()` in non-test source.

## Commands
```bash
cargo fmt              # Format code (uses rustfmt.toml)
cargo fmt --check      # Check formatting without writing
cargo clippy           # Lint
cargo test             # Run 396 integration tests
cargo build            # Compile
cargo run              # Launch game in terminal
```

## Verification Checklist
After any code change, run all:
1. `cargo fmt --check` — formatting clean
2. `cargo clippy` — zero warnings
3. `cargo test` — all 396 pass
4. Update `CHANGELOG.md` — add entry under `[Unreleased]` or new version

## Dependencies
| Crate | Version | Used In |
|-------|---------|---------|
| anyhow | 1.0 | Listed but unused |
| bracket-lib | 0.8.7 | main.rs, renderer.rs, audio.rs |

## Notes
- CI: GitHub Actions (`.github/workflows/`). Lint, test, build, cross-platform release.
- No Makefile, build.rs, or custom scripts.
- Config: `rustfmt.toml` only. No clippy.toml or .editorconfig.
- Coords: `grid[y][x]` — bounds-check before access.
- Levels: 1 (basic), 2 (inverted maze + obstacles), 3 (zigzag + BFS enemies), 4 (fortress rooms + FOV patrol).
- Lives: 3. Enemy collision = -1 life. 0 lives → Lost → retry level.
- HP: Level 4 enemies have `hp: Some(30)`. Melee (x key) = 10 dmg. 3 hits kill.
- Torchlight checkpoints: activate permanent FOV radius 6. Death with checkpoint → respawn (HP + teleport).
- `examples/spike.rs`: bracket-lib PoC spike.

## Agent skills

### Issue tracker

GitHub Issues via `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Five canonical roles with default label names. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context repo. See `docs/agents/domain.md`.

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->