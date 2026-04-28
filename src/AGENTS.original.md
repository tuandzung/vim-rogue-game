<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-17 | Updated: 2026-04-22 -->
# src

All application source code for vim-rogue. Tests are in the `tests/` directory (integration tests).

## Key Files
| File | Lines | Role |
|------|-------|------|
| `main.rs` | 44 | Binary entry — bracket-lib setup, event loop, quit handling via `ctx.quit()`, delegates to game/renderer |
| `lib.rs` | 9 | Library root — `pub mod` re-exports all modules |
| `game.rs` | 814 | `App` state, `handle_key`/`tick`, `parse_motion`, `execute_motion`, `spawn_enemies_for_current_level`, `enemies_step`, win/loss/retry, pause menu, trail, audio dispatch |
| `player.rs` | 247 | `Player` struct + 13 motion impls (h/j/k/l/w/b/0/$/G/gg/f/t/dd) |
| `map.rs` | 471 | `Map` struct, 80×40 grid, 5 zones, 4 levels (`carve_level`, `build_level_2/3/4`), enemy spawn points + patrol areas |
| `renderer.rs` | 899 | bracket-lib rendering — title/gameplay/win/lost/pause screens, viewport, sidebar, minimap, zone colors |
| `types.rs` | 355 | Position, Tile, Zone, VimMotion, Direction, Enemy, PatrolArea, GameState, PauseOption, App, RenderGrid, ViewModel, ScreenModel |
| `animation.rs` | 182 | `GameClock` trait, `RealClock`/`TestClock`, `AnimationState`, `AnimationTimer`, `Interpolator` |
| `visibility.rs` | 124 | `VisibilityMap` with `compute_fov`, `VisibilityState` (Hidden/Explored/Visible) |
| `enemy.rs` | 180 | `Enemy` struct with FOV-aware BFS `step_toward_player`, `has_line_of_sight`, `patrol_step` |
| `audio.rs` | 55 | `AudioManager` + `SoundEffect` enum, graceful no-op fallback |

## Where To Look
| Task | File | What to change |
|------|------|----------------|
| Add Vim motion | `player.rs` + `types.rs` | VimMotion enum, handle_motion match arm, game.rs parse_motion |
| Change dungeon | `map.rs` | carve_level, build_level_2/3/4, assign_zones |
| Change UI | `renderer.rs` | Pure display only — never mutates state |
| Change game flow | `game.rs` | handle_key, tick, pending_input two-phase for f/t/dd/gg; ESC/q opens pause menu |
| Change pause menu | `game.rs` + `renderer.rs` + `types.rs` | GameState::Paused, PauseOption enum, render_pause_overlay |
| Add shared type | `types.rs` | All modules import via `crate::types::*` |
| Change enemy AI | `enemy.rs` | step_toward_player (BFS), has_line_of_sight (Bresenham LOS), patrol_step (room patrol), called via game.rs enemies_step |
| Change visibility | `visibility.rs` | compute_fov, VisibilityMap, demote_visible_to_explored |
| Add animation | `animation.rs` | AnimationState + Interpolator; durations as constants |
| Add sound | `audio.rs` | SoundEffect enum + AudioManager.play() |

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
- Formatting configured via `rustfmt.toml` (`use_small_heuristics = "Max"`, `edition = "2024"`). Run `cargo fmt --check` before committing.
- `grid[y][x]` row-major indexing — always bounds-check before access.
- Event handling: single-key motions execute immediately; f/t/dd/gg set `pending_input` for next keypress.
- Pause menu: ESC or q opens pause overlay; j/k or ↑/↓ navigate options; Enter selects; ESC resumes. `tick()` freezes when paused.
- `Tile` has `glyph()` for char + `Display` for string. `VimMotion` has `key_label()`, `display_name()`, `description()`.
- Input queue: `input_queue` in App buffers keypresses during animation; dequeued after animation completes.
- `GameClock` trait: `RealClock` in production, `TestClock` (deterministic) in tests.
- Animation durations: `PLAYER_MOVE_MS` (150ms), `ENEMY_MOVE_MS` (200ms), `EFFECT_MS` in animation.rs.
- FOV: `compute_fov` uses ray-casting with `FOV_RADIUS`; `demote_visible_to_explored` called before each recomputation.
- Enemy AI: FOV-aware with `ENEMY_FOV_RADIUS=8`. `has_line_of_sight` uses Bresenham LOS. Enemies chase via BFS when player visible, patrol within `PatrolArea` when not. `patrol_area` field on Enemy, `enemy_patrol_areas` on Map. Melee combat gated on `hp.is_some()` (not level number).
- Audio: disabled by default; `play()` no-ops when disabled; `SoundEffect` enum in audio.rs.

## Tests
393 integration tests in `tests/` directory (no inline tests in src/):
| File | Tests | Coverage |
|------|-------|----------|
| `tests/game.rs` | 140 | Motions, pending input, animations, input queue, level transitions, enemies, audio, trail, visibility, win/loss/retry, pause menu, melee combat |
| `tests/renderer.rs` | 53 | Zone colors, wall glyphs, duration formatting, phases, exit glow, trail colors, minimap, fog, centering |
| `tests/map.rs` | 46 | Dimensions, tiles, passability, zones, corridors, obstacles, 4 levels, reachability, enemy spawns, patrol areas, torchlight rooms |
| `tests/animation.rs` | 34 | Timer progress, interpolation, easing, AnimationState, TestClock determinism |
| `tests/visibility.rs` | 29 | FOV center, wall blocking, radius, explored persistence, reset, corridors, symmetry, edge cases |
| `tests/player.rs` | 29 | All 13 motions + boundaries + wall-stopping (w/b/G/gg) + motion recording |
| `tests/enemy.rs` | 21 | BFS movement, diagonal, walls, adjacency, corridors, shortest path, LOS, patrol |
| `tests/types.rs` | 25 | Tile glyphs, motion labels/names/descriptions, zone titles, direction deltas, RenderGrid, ViewModel, Enemy |
| `tests/audio.rs` | 16 | Manager lifecycle, play no-op, enable/disable, rapid play, sound variants |
| `main.rs` | 0 | No tests (thin wrapper) |
| `lib.rs` | 0 | No tests (re-exports only) |

Shared test helpers in `tests/common/mod.rs`: `test_map(w,h)`, `started_app_with_map(map,pos)`, `test_app()`, `assert_approx_eq()`, `approx_eq()`, `tick_timer()`, `tick_state()`, `all_transparent()`, `with_walls()`, `with_transparent_tiles()`.

## Notes
- Map defaults: start (2,2), exit (76,36). Zones: 16 columns each.
- Zone 5 has obstacles (only area with `Tile::Obstacle`).
- Level 2: inverted maze, obstacles in earlier zones, different start/exit.
- Level 3: zigzag corridors, enemy spawn points from `map.enemy_spawns`, torchlights at corridor junctions.
- Level 4: fortress rooms, 9 enemies with room-based patrol areas, no-torchlight rooms have ≥2 enemies, HP-based combat.
- `Enemy` glyph: `⚡` (default). Stored in `Enemy` struct field.
- `RenderGrid`/`RenderCell`/`ViewModel`/`ScreenModel` are renderer-specific types in types.rs.
- `examples/spike.rs`: bracket-lib proof-of-concept, not part of main build.

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->
