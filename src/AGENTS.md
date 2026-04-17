<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-17 | Updated: 2026-04-17 -->

# src

All application source code and inline tests for vim-quake.

## Key Files
| File | Lines | Role |
|------|-------|------|
| `main.rs` | 32 | Binary entry — bracket-lib setup, event loop, delegates to game/renderer |
| `lib.rs` | 9 | Library root — `pub mod` re-exports all modules |
| `game.rs` | 1470 | `App` state, `handle_key`/`tick`, `parse_motion`, `execute_motion`, `enemies_step`, win/loss/retry, trail, audio dispatch |
| `player.rs` | 521 | `Player` struct + 13 motion impls (h/j/k/l/w/b/0/$/G/gg/f/t/dd) |
| `map.rs` | 714 | `Map` struct, 80×40 grid, 5 zones, 3 levels (`carve_level`, `build_level_2/3`), enemy spawn points |
| `renderer.rs` | 1266 | bracket-lib rendering — title/gameplay/win/lost screens, viewport, sidebar, minimap, zone colors |
| `types.rs` | 504 | Position, Tile, Zone, VimMotion, Direction, Enemy, GameState, App, RenderGrid, ViewModel, ScreenModel |
| `animation.rs` | 491 | `GameClock` trait, `RealClock`/`TestClock`, `AnimationState`, `AnimationTimer`, `Interpolator` |
| `visibility.rs` | 519 | `VisibilityMap` with `compute_fov`, `VisibilityState` (Hidden/Explored/Visible) |
| `enemy.rs` | 244 | `Enemy` struct with BFS `step_toward_player` |
| `audio.rs` | 256 | `AudioManager` + `SoundEffect` enum, graceful no-op fallback |

## Where To Look
| Task | File | What to change |
|------|------|----------------|
| Add Vim motion | `player.rs` + `types.rs` | VimMotion enum, handle_motion match arm, game.rs parse_motion |
| Change dungeon | `map.rs` | carve_level, build_level_2/3, assign_zones |
| Change UI | `renderer.rs` | Pure display only — never mutates state |
| Change game flow | `game.rs` | handle_key, tick, pending_input two-phase for f/t/dd/gg |
| Add shared type | `types.rs` | All modules import via `crate::types::*` |
| Change enemy AI | `enemy.rs` | step_toward_player (BFS), called via game.rs enemies_step |
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
- `grid[y][x]` row-major indexing — always bounds-check before access.
- Event handling: single-key motions execute immediately; f/t/dd/gg set `pending_input` for next keypress.
- `Tile` has `glyph()` for char + `Display` for string. `VimMotion` has `key_label()`, `display_name()`, `description()`.
- Input queue: `input_queue` in App buffers keypresses during animation; dequeued after animation completes.
- `GameClock` trait: `RealClock` in production, `TestClock` (deterministic) in tests.
- Animation durations: `PLAYER_MOVE_MS` (150ms), `ENEMY_MOVE_MS` (200ms), `EFFECT_MS` in animation.rs.
- FOV: `compute_fov` uses ray-casting with `FOV_RADIUS`; `demote_visible_to_explored` called before each recomputation.
- Enemy BFS: `step_toward_player` finds shortest path; won't walk through walls or stay on player tile.
- Audio: disabled by default; `play()` no-ops when disabled; `SoundEffect` enum in audio.rs.

## Tests
275 inline tests across 9 files (`#[cfg(test)] mod tests` at bottom):
| File | Tests | Coverage |
|------|-------|----------|
| `game.rs` | 72 | Motions, pending input, animations, input queue, level transitions, enemies, audio, trail, visibility, win/loss/retry |
| `renderer.rs` | 44 | Zone colors, wall glyphs, duration formatting, phases, exit glow, trail colors, minimap, fog, centering |
| `map.rs` | 33 | Dimensions, tiles, passability, zones, corridors, obstacles, 3 levels, reachability, enemy spawns |
| `animation.rs` | 29 | Timer progress, interpolation, easing, AnimationState, TestClock determinism |
| `visibility.rs` | 25 | FOV center, wall blocking, radius, explored persistence, reset, corridors, symmetry, edge cases |
| `player.rs` | 25 | All 13 motions + boundaries + motion recording |
| `types.rs` | 21 | Tile glyphs, motion labels/names/descriptions, zone titles, direction deltas, RenderGrid, ViewModel, Enemy |
| `audio.rs` | 16 | Manager lifecycle, play no-op, enable/disable, rapid play, sound variants |
| `enemy.rs` | 10 | BFS movement, diagonal, walls, adjacency, corridors, shortest path |
| `main.rs` | 0 | No tests (thin wrapper) |
| `lib.rs` | 0 | No tests (re-exports only) |

Per-file test helpers: `test_map(w,h)`, `started_app_with_map(map,pos)`, `key_event(code)`, `assert_approx_eq()`, `tick_timer()`, `tick_state()`.

## Notes
- Map defaults: start (2,2), exit (76,36). Zones: 16 columns each.
- Zone 5 has obstacles (only area with `Tile::Obstacle`).
- Level 2: inverted maze, obstacles in earlier zones, different start/exit.
- Level 3: zigzag corridors, enemy spawn points from `map.enemy_spawns`.
- `Enemy` glyph: `⚡` (default). Stored in `Enemy` struct field.
- `RenderGrid`/`RenderCell`/`ViewModel`/`ScreenModel` are renderer-specific types in types.rs.
- `examples/spike.rs`: bracket-lib proof-of-concept, not part of main build.

<!-- MANUAL: Any manually added notes below this line are preserved on regeneration -->
