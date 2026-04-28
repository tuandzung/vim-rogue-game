<!-- Generated: 2026-04-17 | Updated: 2026-04-22 -->
<!-- Commit: f0954bc | Branch: feat/secret-cheat-codes -->

# vim-rogue

Terminal-based roguelike dungeon game (Rust + bracket-lib) teaching Vim motions through gameplay. 80×40 dungeon, 4 levels, 5 zone-gated areas, 13 Vim keybindings, FOV-aware enemy AI with patrol, fog-of-war visibility.

## Structure
```
vim-rogue/
├── src/          # Application source code (see src/AGENTS.md)
├── tests/        # Integration tests — 393 tests across 9 files (see tests/AGENTS.md)
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
game.rs       → App state, event handling, motion dispatch, FOV-gated enemy turns, win/loss, trail, audio (814 lines)
player.rs     → Player + 13 motion implementations (247 lines)
map.rs        → 80×40 grid, 5 zones, 4 dungeon levels, corridor carving, enemy spawns + patrol areas (471 lines)
renderer.rs   → bracket-lib rendering: title, viewport, sidebar, minimap, win/loss screens, ASCII art (899 lines)
types.rs      → Position, Tile, Zone, VimMotion, Direction, Enemy, PatrolArea, App, RenderGrid, ViewModel (355 lines)
animation.rs  → GameClock, AnimationState, AnimationTimer, Interpolator (easing) (182 lines)
visibility.rs → VisibilityMap with FOV (explored/visible/hidden states) (124 lines)
enemy.rs      → Enemy struct with FOV-aware BFS chase + room patrol (180 lines)
audio.rs      → AudioManager with SoundEffect enum, graceful fallback (55 lines)
lib.rs        → Re-exports all modules (9 lines)
```

## Where To Look
| Task | Location | Notes |
|------|----------|-------|
| Add a new Vim motion | `src/player.rs` + `src/types.rs` (VimMotion enum) | Also update game.rs parse_motion |
| Change dungeon layout | `src/map.rs` (carve_level, build_level_2/3/4) | grid[y][x] row-major; 4 levels |
| Add UI elements | `src/renderer.rs` | Pure display — never mutates state |
| Change game flow | `src/game.rs` (handle_key, tick, execute_motion) | Two-phase input for f/t/dd/gg; pause menu on ESC/q |
| Change pause menu | `src/game.rs` + `src/renderer.rs` + `src/types.rs` | GameState::Paused, PauseOption, render_pause_overlay |
| Add new types | `src/types.rs` | All modules import via `crate::types::*` |
| Change enemy AI | `src/enemy.rs` (step_toward_player, has_line_of_sight, patrol_step) | FOV-gated BFS chase + patrol, called from game.rs enemies_step |
| Change FOV/visibility | `src/visibility.rs` (compute_fov) | VisibilityMap with Hidden/Explored/Visible states |
| Add animations | `src/animation.rs` | AnimationState + Interpolator; clock via GameClock trait |
| Add sound effects | `src/audio.rs` (SoundEffect enum + AudioManager) | Audio disabled by default |
| Fix a bug | Check tests in `tests/` directory (393 integration tests across 9 files) | main.rs and lib.rs have no tests |

## Conventions
- Rust edition 2024. Formatting configured via `rustfmt.toml` (`use_small_heuristics = "Max"`, `edition = "2024"`).
- Integration tests in `tests/` directory (393 tests across 9 files). Shared helpers in `tests/common/mod.rs`.
- Test helpers: `test_map()`, `started_app_with_map()`, `test_app()`, `assert_approx_eq()`, `approx_eq()`, `tick_timer()`, `tick_state()`.
- `renderer.rs` internals are `pub` for integration test access (e.g., `screen_meets_minimum_size`, `phase_definitions`, `exit_glow`, etc.).
- `lib.rs` re-exports all modules. `main.rs` is thin (~32 lines).
- `is_passable` = `Tile::Floor`, `Tile::Exit`, or `Tile::Torchlight`. `Tile::Obstacle` is not passable but can be destroyed by `dd`.
- w/b motions scan horizontally along clear paths, stopping at non-passable tiles (walls/obstacles).
- G/gg motions scan vertically from current position, stopping at non-passable tiles (walls/obstacles).
- `renderer.rs` is read-only — never mutates App state.
- `Player::handle_motion` takes `&mut Map` (dd deletes obstacles).
- `GameClock` trait for time — `RealClock` in production, `TestClock` in tests.
- Animation durations: player 150ms (`PLAYER_MOVE_MS`), enemy 200ms (`ENEMY_MOVE_MS`).
- FOV radius: `FOV_RADIUS` constant in types.rs. Enemy FOV: `ENEMY_FOV_RADIUS = 8`.
- Enemies have FOV-aware AI: chase via BFS when player visible (`has_line_of_sight`), patrol within `PatrolArea` otherwise.
- Level 4 enemies have room-based patrol areas; no-torchlight rooms have ≥2 enemies.
- Audio disabled by default; `AudioManager::enable()` to activate.
- Error handling: `BError` from bracket-lib in main.rs. `anyhow` listed in Cargo.toml but unused in source.
- `unwrap()`/`expect()` present in non-test source code.

## Commands
```bash
cargo fmt              # Format code (uses rustfmt.toml)
cargo fmt --check      # Check formatting without writing
cargo clippy           # Lint
cargo test             # Run 393 integration tests
cargo build            # Compile
cargo run              # Launch game in terminal
```

## Verification Checklist
After any code change, run ALL of these before considering work complete:
1. `cargo fmt --check` — ensure formatting is clean
2. `cargo clippy` — zero warnings
3. `cargo test` — all 393 tests pass
4. Update `CHANGELOG.md` — add entry under `[Unreleased]` or new version section

## Dependencies
| Crate | Version | Used In |
|-------|---------|---------|
| anyhow | 1.0 | Listed but unused in source |
| bracket-lib | 0.8.7 | main.rs (terminal + rendering), renderer.rs, audio.rs |

## Notes
- CI/CD: GitHub Actions for lint, test, build, and cross-platform release (`.github/workflows/`).
- No Makefile, build.rs, or custom scripts.
- Config files: `rustfmt.toml` (formatting rules). No clippy.toml or .editorconfig.
- Coordinate system: `grid[y][x]` — always bounds-check before access.
- 4 dungeon levels: Level 1 (basic), Level 2 (inverted maze + obstacles), Level 3 (zigzag + BFS enemies), Level 4 (fortress rooms + FOV-aware patrol enemies).
- Lives system: 3 lives; enemy collision costs a life; 0 lives → Lost state → retry current level.
- HP system: Level 4 enemies have HP (`hp: Some(30)`); melee attack (x key) deals 10 damage; 3 hits to kill.
- Torchlight checkpoints: step on torchlight to activate permanent FOV radius 6; death with checkpoint → respawn (HP restore + teleport).
- `examples/spike.rs`: bracket-lib proof-of-concept spike.

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->
