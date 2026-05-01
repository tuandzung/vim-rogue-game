<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-17 | Updated: 2026-05-01 (PR#13) -->
# src

All vim-rogue source. Tests in `tests/` (integration only).

## Key Files
| File | Lines | Role |
|------|-------|------|
| `main.rs` | 44 | Binary entry — bracket-lib setup, event loop, `ctx.quit()`, delegates to game/renderer |
| `lib.rs` | 11 | Library root — `pub mod` re-exports + `test_support` module |
| `game.rs` | 647 | App coordinator — `handle_key`/`tick`, thin `enemies_step` coordinator (collision outcomes, animation, audio) |
| `player.rs` | 260 | `PlayerState` impl — 13 motions + motion tracking (h/j/k/l/w/b/0/$/G/gg/f/t/dd) |
| `map.rs` | 471 | `Map`, 80×40 grid, 5 zones, 4 levels (`carve_level`, `build_level_2/3/4`), enemy spawns + patrol areas |
| `renderer.rs` | 914 | bracket-lib render — title/gameplay/win/lost/pause screens, viewport, sidebar, minimap, zone colors |
| `types.rs` | 691 | Position, Tile, Zone, VimMotion, Direction, Enemy, PatrolArea, EnemyMovement, EnemyTurn, PlayerState, App + 3 aggregates (World, InputState, Session), RenderGrid, ViewModel, ScreenModel |
| `animation.rs` | 182 | `GameClock` trait, `RealClock`/`TestClock`, `AnimationState`, `AnimationTimer`, `Interpolator` |
| `visibility.rs` | 124 | `VisibilityMap` + `compute_fov`, `VisibilityState` (Hidden/Explored/Visible) |
| `enemy.rs` | 180 | `Enemy` + FOV-aware BFS `step_toward_player`, `has_line_of_sight`, `patrol_step` |
| `audio.rs` | 55 | `AudioManager` + `SoundEffect` enum, no-op fallback |
| `test_support.rs` | 26 | `App::for_test` constructor — centralized test seam for integration tests |

## Where To Look
| Task | File | What to change |
|------|------|----------------|
| Add Vim motion | `player.rs` + `types.rs` | VimMotion enum, handle_motion arm on PlayerState, game.rs parse_motion |
| Change dungeon | `map.rs` | carve_level, build_level_2/3/4, assign_zones |
| Change UI | `renderer.rs` | Display only — never mutates state |
| Change game flow | `game.rs` | handle_key, tick, pending_input for f/t/dd/gg; ESC/q opens pause |
| Change pause menu | `game.rs` + `renderer.rs` + `types.rs` | GameState::Paused, PauseOption, render_pause_overlay |
| Add shared type | `types.rs` | All modules `use crate::types::*` |
| Change enemy AI | `enemy.rs` + `types.rs` | step_toward_player (BFS), has_line_of_sight (Bresenham), patrol_step in enemy.rs; `World::step_enemies` orchestrates turn in types.rs |
| Change visibility | `visibility.rs` | compute_fov, VisibilityMap, demote_visible_to_explored |
| Change aggregate logic | `types.rs` + `player.rs` | World (terrain, visibility, enemies), PlayerState (position, motions, HP, trail, progression; impl in player.rs), InputState (key buffering), Session (lifecycle, timing, pause) |
| Add animation | `animation.rs` | AnimationState + Interpolator; durations as constants |
| Add sound | `audio.rs` | SoundEffect enum + AudioManager.play() |
| Add test constructor | `test_support.rs` | `App::for_test(map, position)` — single seam for all integration tests |

## Internal Dependencies
```
types.rs      ← (all modules)
map.rs        ← player.rs, enemy.rs, game.rs
player.rs     ← game.rs
enemy.rs      ← game.rs
visibility.rs ← game.rs, renderer.rs
animation.rs  ← game.rs
audio.rs      ← game.rs
renderer.rs   ← main.rs (reads types for display)
game.rs       ← main.rs
lib.rs        ← main.rs (implicit)
```

## Conventions
- `rustfmt.toml`: `use_small_heuristics = "Max"`, `edition = "2024"`. Run `cargo fmt --check` pre-commit.
- `grid[y][x]` row-major — always bounds-check.
- **Aggregates**: `App` is a thin coordinator delegating to 3 domain aggregates + PlayerState:
  - `World` — terrain, visibility, enemies, torchlights; owns `update_visibility`, `step_enemies`, `push_enemies_off_position`, `reset_for_level`
  - `PlayerState` — flat struct (position, used_motions, last_direction, noclip, HP, trail, motion tracking, level, checkpoint, pending respawn); `impl PlayerState` in player.rs owns all motion logic + tracking (motion_count, discovered_motions)
  - `InputState` — `input_queue` + `pending_input` for two-phase Vim commands (f/t/dd/gg)
  - `Session` — game state, pause selection, timing, status message
- Single-key motions fire immediately; f/t/dd/gg set `pending_input` via InputState for next key.
- Pause: ESC/q opens overlay; j/k or ↑/↓ navigate; Enter selects; ESC resumes. `tick()` freezes when paused.
- `Tile`: `glyph()` (char) + `Display` (string). `VimMotion`: `key_label()`, `display_name()`, `description()`.
- `input_queue` in InputState buffers keys during animation; dequeued after complete.
- `GameClock`: `RealClock` prod, `TestClock` (deterministic) tests.
- Durations: `PLAYER_MOVE_MS` (150ms), `ENEMY_MOVE_MS` (200ms), `EFFECT_MS` in animation.rs.
- FOV: `compute_fov` ray-casts with `FOV_RADIUS`; `demote_visible_to_explored` before each recomputation. `update_visibility` lives on `World`.
- Enemy AI: FOV-aware, `ENEMY_FOV_RADIUS=8`. Bresenham LOS. BFS chase when visible, `PatrolArea` patrol when not. `patrol_area` on Enemy, `enemy_patrol_areas` on Map. Melee gated on `hp.is_some()`.
- Audio: disabled default; `play()` no-ops when off.

## Tests
396 integration tests in `tests/` (no inline tests in src/):
| File | Tests | Coverage |
|------|-------|----------|
| `tests/game.rs` | 140 | Motions, pending input, animations, input queue, level transitions, enemies, audio, trail, visibility, win/loss/retry, pause, melee |
| `tests/renderer.rs` | 53 | Zone colors, wall glyphs, duration formatting, phases, exit glow, trail colors, minimap, fog, centering |
| `tests/map.rs` | 46 | Dimensions, tiles, passability, zones, corridors, obstacles, 4 levels, reachability, spawns, patrol, torchlight |
| `tests/animation.rs` | 34 | Timer progress, interpolation, easing, AnimationState, TestClock |
| `tests/visibility.rs` | 29 | FOV center, wall blocking, radius, explored persistence, reset, corridors, symmetry, edge cases |
| `tests/player.rs` | 32 | All 13 motions + boundaries + wall-stopping (w/b/G/gg) + recording + motion_count + discovered_motions |
| `tests/enemy.rs` | 21 | BFS movement, diagonal, walls, adjacency, corridors, shortest path, LOS, patrol |
| `tests/types.rs` | 25 | Tile glyphs, motion labels/names/descriptions, zone titles, direction deltas, RenderGrid, ViewModel, Enemy |
| `tests/audio.rs` | 16 | Lifecycle, play no-op, enable/disable, rapid play, variants |
| `main.rs` | 0 | No tests (thin wrapper) |
| `lib.rs` | 0 | No tests (re-exports) |

Shared helpers in `tests/common/mod.rs`: `test_map(w,h)`, `started_app_with_map(map,pos)`, `test_app()`, `assert_approx_eq()`, `approx_eq()`, `tick_timer()`, `tick_state()`, `all_transparent()`, `with_walls()`, `with_transparent_tiles()`.

## Notes
- Map: start (2,2), exit (76,36). Zones: 16 cols each.
- Zone 5 = obstacles only (`Tile::Obstacle`).
- Level 2: inverted maze, obstacles in earlier zones, different start/exit.
- Level 3: zigzag corridors, spawns from `map.enemy_spawns`, torchlights at junctions.
- Level 4: fortress rooms, 9 enemies + room patrol, no-torchlight rooms have ≥2 enemies, HP combat.
- Enemy glyph: `⚡` (default), stored in struct field.
- `RenderGrid`/`RenderCell`/`ViewModel`/`ScreenModel` = renderer types in types.rs.
- `examples/spike.rs`: bracket-lib POC, not in main build.

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->