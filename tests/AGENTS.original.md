<!-- Parent: ../AGENTS.md -->

# tests

393 integration tests across 9 files. No inline tests in src/. Shared helpers in `tests/common/mod.rs`.

## Test Files
| File | Tests | Lines | Coverage |
|------|-------|-------|----------|
| `game.rs` | 140 | 1968 | Motions, pending input, animations, input queue, level transitions, enemies, audio, trail, visibility, win/loss/retry, pause menu, melee combat, checkpoint respawn |
| `map.rs` | 46 | 539 | Dimensions, tiles, passability, zones, corridors, obstacles, 4 levels, reachability, enemy spawns, patrol areas, torchlight room presence |
| `renderer.rs` | 53 | 428 | Zone colors, wall glyphs, duration formatting, phases, exit glow, trail colors, minimap, fog, centering |
| `visibility.rs` | 29 | 420 | FOV center, wall blocking, radius, explored persistence, reset, corridors, symmetry, edge cases, multi-source FOV |
| `player.rs` | 29 | 324 | All 13 motions + boundaries + wall-stopping (w/b/G/gg) + motion recording |
| `animation.rs` | 34 | 349 | Timer progress, interpolation, easing, AnimationState, TestClock determinism, attack effects |
| `types.rs` | 25 | 216 | Tile glyphs, motion labels/names/descriptions, zone titles, direction deltas, RenderGrid, ViewModel, Enemy |
| `enemy.rs` | 21 | 265 | BFS movement, diagonal, walls, adjacency, corridors, shortest path, LOS, patrol |
| `audio.rs` | 16 | 191 | Manager lifecycle, play no-op, enable/disable, rapid play, sound variants |

## Shared Helpers (`common/mod.rs`)
| Helper | Signature | Purpose |
|--------|-----------|---------|
| `test_map(w, h)` | `-> Map` | Blank map with all-Floor grid |
| `started_app_with_map(map, pos)` | `-> App` | App at given position on given map, GameState::Playing |
| `test_app()` | `-> App` | App on default Level 1 map |
| `tick_timer(timer, clock, delta_ms)` | | Advance timer with TestClock |
| `tick_state(state, clock, delta_ms)` | | Advance AnimationState with TestClock |
| `tick(app, delta_ms)` | | Convenience: advance App clock + call `tick()` |

## Where To Look
| Task | File | Notes |
|------|------|-------|
| Test a motion | `player.rs` | Use `test_app()` + `handle_key()` |
| Test enemy behavior | `enemy.rs` | Manual enemy construction + `step_toward_player` |
| Test level layout | `map.rs` | `Map::level(N)` + grid assertions |
| Test game state transitions | `game.rs` | `handle_key()`, `tick()`, state assertions |
| Test visibility | `visibility.rs` | `VisibilityMap` + `compute_fov` + `with_walls()` helper |

## Conventions
- Each test file mirrors a src/ module (e.g., `tests/player.rs` ↔ `src/player.rs`).
- `handle_key(app, VirtualKeyCode, shift: bool)` is the main input entry point for game tests.
- `tick(&mut app, delta_ms: f64)` advances the game clock and processes one frame.
- Animation constants: `PLAYER_MOVE_MS = 150.0`, `ENEMY_MOVE_MS = 200.0`, `ATTACK_EFFECT_MS = 200.0` — import from `vim_rogue::animation`.
- `TestClock` provides deterministic timing; always use it in tests (never `RealClock`).
- Enemy construction: `Enemy::new(pos)` for default; override fields with `Enemy { position: pos, hp: Some(30), ..Enemy::new(pos) }` for Level 4 enemies.
- Level 4 helper: `level4_app_with_enemy(pos, hp)` in `tests/game.rs` creates app on Level 4 map with one enemy at given position.
- `renderer.rs` internals are `pub` for test access (e.g., `screen_meets_minimum_size`, `phase_definitions`, `exit_glow`).

## Notes
- Formatting configured via `rustfmt.toml` — run `cargo fmt --check` before committing.
- `tests/common/mod.rs` has some helpers trigger `dead_code` warnings — expected, only used by subset of test files.
- Game test file (`tests/game.rs`) is largest at 1968 lines due to comprehensive state machine coverage.
- No `#[should_panic]` tests — error cases return gracefully.
