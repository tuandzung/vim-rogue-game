<!-- Generated: 2026-04-17 | Updated: 2026-04-17 -->
<!-- Commit: ce947f2 | Branch: feat/graphics-overhaul-bracket-lib -->

# vim-quake

Terminal-based roguelike dungeon game (Rust + bracket-lib) teaching Vim motions through gameplay. 80×40 dungeon, 3 levels, 5 zone-gated areas, 13 Vim keybindings, BFS enemy AI, FOV visibility.

## Structure
```
vim-quake/
├── src/          # All source + inline tests (see src/AGENTS.md)
├── examples/     # Spike/prototype code (spike.rs)
├── Cargo.toml    # Edition 2024, deps: anyhow (unused), bracket-lib
├── Cargo.lock
├── README.md
└── .gitignore    # target/, *.rs.bk, *.pdb, mutants.out*/, .omc/
```

## Architecture
```
main.rs       → bracket-lib setup + event loop (32 lines)
game.rs       → App state, event handling, motion dispatch, enemy turns, win/loss, trail, audio (1470 lines)
player.rs     → Player + 13 motion implementations (521 lines)
map.rs        → 80×40 grid, 5 zones, 3 dungeon levels, corridor carving, enemy spawns (714 lines)
renderer.rs   → bracket-lib rendering: title, viewport, sidebar, minimap, win/loss screens, ASCII art (1266 lines)
types.rs      → Position, Tile, Zone, VimMotion, Direction, Enemy, App, RenderGrid, ViewModel (504 lines)
animation.rs  → GameClock, AnimationState, AnimationTimer, Interpolator (easing) (491 lines)
visibility.rs → VisibilityMap with FOV (explored/visible/hidden states) (519 lines)
enemy.rs      → Enemy struct with BFS pathfinding toward player (244 lines)
audio.rs      → AudioManager with SoundEffect enum, graceful fallback (256 lines)
lib.rs        → Re-exports all modules (9 lines)
```

## Where To Look
| Task | Location | Notes |
|------|----------|-------|
| Add a new Vim motion | `src/player.rs` + `src/types.rs` (VimMotion enum) | Also update game.rs parse_motion |
| Change dungeon layout | `src/map.rs` (carve_level, build_level_2/3) | grid[y][x] row-major; 3 levels |
| Add UI elements | `src/renderer.rs` | Pure display — never mutates state |
| Change game flow | `src/game.rs` (handle_key, tick, execute_motion) | Two-phase input for f/t/dd/gg |
| Add new types | `src/types.rs` | All modules import via `crate::types::*` |
| Change enemy AI | `src/enemy.rs` (step_toward_player) | BFS pathfinding, called from game.rs enemies_step |
| Change FOV/visibility | `src/visibility.rs` (compute_fov) | VisibilityMap with Hidden/Explored/Visible states |
| Add animations | `src/animation.rs` | AnimationState + Interpolator; clock via GameClock trait |
| Add sound effects | `src/audio.rs` (SoundEffect enum + AudioManager) | Audio disabled by default |
| Fix a bug | Check tests first: 275 inline tests across 9 files | main.rs and lib.rs have no tests |

## Conventions
- Rust edition 2024. No clippy/rustfmt config — defaults apply.
- Inline tests only (`#[cfg(test)] mod tests`). No `tests/` directory, no test frameworks.
- Test helpers per-file: `test_map()`, `started_app_with_map()`, `key_event()`, `assert_approx_eq()`, `tick_timer()`.
- `lib.rs` re-exports all modules. `main.rs` is thin (~32 lines).
- `is_passable` = `Tile::Floor` or `Tile::Exit` only.
- `renderer.rs` is read-only — never mutates App state.
- `Player::handle_motion` takes `&mut Map` (dd deletes obstacles).
- `GameClock` trait for time — `RealClock` in production, `TestClock` in tests.
- Animation durations: player 150ms (`PLAYER_MOVE_MS`), enemy 200ms (`ENEMY_MOVE_MS`).
- FOV radius: `FOV_RADIUS` constant in types.rs.
- Audio disabled by default; `AudioManager::enable()` to activate.
- Error handling: `BError` from bracket-lib in main.rs. `anyhow` listed in Cargo.toml but unused in source.
- `unwrap()`/`expect()` present in non-test source code.

## Commands
```bash
cargo build          # Compile
cargo test           # Run 275 inline tests
cargo run            # Launch game in terminal
```

## Dependencies
| Crate | Version | Used In |
|-------|---------|---------|
| anyhow | 1.0 | Listed but unused in source |
| bracket-lib | 0.8.7 | main.rs (terminal + rendering), renderer.rs, audio.rs |

## Notes
- No CI/CD configured. No Makefile, build.rs, or custom scripts.
- No config files beyond Cargo.toml (no .editorconfig, clippy.toml, rustfmt.toml).
- Coordinate system: `grid[y][x]` — always bounds-check before access.
- 3 dungeon levels: Level 1 (basic), Level 2 (inverted maze + obstacles), Level 3 (zigzag + BFS enemies).
- Lives system: 3 lives; enemy collision costs a life; 0 lives → Lost state → retry current level.
- `examples/spike.rs`: bracket-lib proof-of-concept spike.

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->
