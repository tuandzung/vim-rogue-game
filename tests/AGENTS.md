<!-- Parent: ../AGENTS.md -->

# tests

393 integration tests, 9 files. No inline tests in src/. Shared helpers in `tests/common/mod.rs`.

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
| `test_map(w, h)` | `-> Map` | Blank map, all-Floor grid |
| `started_app_with_map(map, pos)` | `-> App` | App at pos on map, GameState::Playing |
| `test_app()` | `-> App` | App on default Level 1 map |
| `tick_timer(timer, clock, delta_ms)` | | Advance timer with TestClock |
| `tick_state(state, clock, delta_ms)` | | Advance AnimationState with TestClock |
| `tick(app, delta_ms)` | | Advance App clock + call `tick()` |

## Where To Look
| Task | File | Notes |
|------|------|-------|
| Test motion | `player.rs` | `test_app()` + `handle_key()` |
| Test enemy | `enemy.rs` | Manual enemy + `step_toward_player` |
| Test level layout | `map.rs` | `Map::level(N)` + grid assertions |
| Test state transitions | `game.rs` | `handle_key()`, `tick()`, state assertions |
| Test visibility | `visibility.rs` | `VisibilityMap` + `compute_fov` + `with_walls()` |

## Conventions
- Test file mirrors src/ module (`tests/player.rs` ↔ `src/player.rs`).
- `handle_key(app, VirtualKeyCode, shift: bool)` = main input entry for game tests.
- `tick(&mut app, delta_ms: f64)` advances clock, processes one frame.
- Animation constants: `PLAYER_MOVE_MS = 150.0`, `ENEMY_MOVE_MS = 200.0`, `ATTACK_EFFECT_MS = 200.0` — import from `vim_rogue::animation`.
- `TestClock` = deterministic timing. Always use in tests, never `RealClock`.
- Enemy: `Enemy::new(pos)` default; override with `Enemy { position: pos, hp: Some(30), ..Enemy::new(pos) }` for Level 4.
- Level 4 helper: `level4_app_with_enemy(pos, hp)` in `tests/game.rs` — app on Level 4 map, one enemy at pos.
- `renderer.rs` internals `pub` for test access (`screen_meets_minimum_size`, `phase_definitions`, `exit_glow`).

## Notes
- `rustfmt.toml` config. Run `cargo fmt --check` before commit.
- `tests/common/mod.rs` helpers trigger `dead_code` warnings — expected, used by subset of test files.
- `tests/game.rs` largest at 1968 lines — comprehensive state machine coverage.
- No `#[should_panic]` tests — error cases return gracefully.