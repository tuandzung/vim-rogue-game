# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Move enemy turn into World aggregate** — `enemies_step` AI dispatch (LOS check, chase, patrol, stun decrement) moved from game.rs into `World::step_enemies`, returning structured `EnemyTurn` result; `push_enemies_off_position` (BFS respawn displacement) moved into `World::push_enemies_off_position`; game.rs `enemies_step` reduced to thin coordinator handling collision outcomes, animation, and audio from returned data

### Added

- `EnemyMovement` and `EnemyTurn` structs — structured result types for `World::step_enemies`

## [0.3.0] - 2026-04-29

### Changed

- **Decompose `App` god object into 4 domain aggregates** — 29-field `App` struct split into `World`, `PlayerState`, `InputState`, `Session`; `App` reduced to 8-field thin coordinator
- **Move reset logic into aggregate methods** — each aggregate owns its own constructors and reset behavior
- **Move `update_visibility` into `World`** — FOV computation belongs to the aggregate that owns the map
- **Extract motion/damage feedback into `PlayerState`** — status message generation lives with the aggregate that produces it
- **Merge `Player` into `PlayerState`** — eliminate shallow `inner: Player` wrapper; PlayerState now holds position, used_motions, last_direction, noclip directly. `handle_motion` owns motion tracking (motion_count, discovered_motions)

### Tests

- 396 integration tests (+3 covering motion_count and discovered_motions behavior)

## [0.2.2] - 2026-04-22

### Added

- **Cheat codes for testing** — secret two-key combos (`iv` skip level, `im` god mode, `ie` kill enemies, `ip` noclip) gated behind `cfg(debug_assertions)` so they are excluded from release builds

### Changed

- Renamed project from `vim-quake` to `vim-rogue`
- Replaced `String`-based cheat buffer with fixed-size `[Option<char>; 2]` struct (`CheatBuffer`) for O(1) push operations (no `remove(0)` shifting)
- Guarded all cheat-related fields, enums, functions, and tests behind `#[cfg(debug_assertions)]` — compiled out of production builds entirely
- Added `can_pass_to()` helper on `Player` and `is_invincible()` helper on `App` for clean conditional access to debug-only behavior

### Tests

- 393 integration tests (8 cheat tests now gated with `#[cfg(debug_assertions)]`)

## [0.2.1] - 2026-04-22

### Changed

- Release workflow now extracts the matching version section from CHANGELOG.md and attaches it to the GitHub release body instead of auto-generated notes

## [0.2.0] - 2026-04-22

### Added

- **Pause menu** — press `Esc` or `q` to pause; navigate with `j`/`k` or arrow keys; choose Resume, Retry Level, or Quit Game
- **Pause overlay rendering** — dimmed backdrop with centered menu panel
- **HP system** — enemies on Level 4 have HP (30); player melee attack (`x` key) deals 10 damage; 3 hits to kill
- **Melee combat** — press `x` to attack adjacent Level 4 enemies; stunned enemies skip their next turn
- **Torchlight checkpoints** — step on torchlight tiles for permanent FOV radius 6 illumination; respawn at last checkpoint on death (HP restored, enemies pushed off tile)
- **Death and retry flow** — `Dying` state with visual effects before loss screen; retry current level with fresh map on death
- **Attack effects** — `PlayerStrike` and `EnemyHit` visual effects with animation lifecycle
- **FOV-aware enemy AI** — enemies chase via BFS when player is visible (within FOV radius 8), patrol within room bounds otherwise
- **Enemy patrol system** — `PatrolArea` type with room-based patrol areas; enemies stay within their assigned room
- **Enemy stun mechanic** — melee hits stun Level 4 enemies for 1 turn, preventing counterattack
- **Facing direction tracking** — player facing updates on h/j/k/l movement for melee targeting
- **Level 4 Fortress** — new dungeon level with room-based layout, 9 enemies, torchlight checkpoints, and melee combat
- **Multi-source FOV** — torchlight checkpoints provide persistent visibility from their location
- **HP bar rendering** — sidebar health bar with color-coded display
- **Destroyable obstacles** — `dd` motion destroys `Tile::Obstacle` in Zone 5
- **Clear-path motions** — `w`/`b`/`G`/`gg` restricted to clear paths, stopping at walls instead of jumping over them

### Changed

- `w`/`b`/`G`/`gg` motions now scan along clear paths and stop at non-passable tiles (walls/obstacles) instead of skipping over them
- Level 3 updated with torchlights at corridor junctions and enemy spawn points from map data
- Enemy AI uses FOV-gated BFS chase with Bresenham line-of-sight instead of simple direct movement

### Fixed

- Replaced manual `Default` impl for `PatrolArea` with `#[derive(Default)]` (clippy warning)

### Tests

- 385 integration tests across 9 test files covering all game systems
- Added pause menu tests, stun mechanic tests, attack effect lifecycle tests, enemy FOV/patrol tests, and Level 4 placement tests

## [0.1.0] - 2026-04-17

### Added

- Initial release
- Graphical window with ASCII/CP437 aesthetic via bracket-lib
- 3 dungeon levels with distinct layouts (basic, inverted maze, zigzag)
- 5 zone-gated areas with color palettes (gray, cyan, magenta, red, gold)
- 13 Vim keybindings: `h` `j` `k` `l` `w` `b` `0` `$` `G` `gg` `f` `t` `dd`
- Level progression with stat carry-over
- Enemy encounters with lives system (3 lives)
- Fog of war with explored tile persistence
- Minimap showing explored areas
- Smooth animations with ease-in-out interpolation
- Sound effects with graceful silent fallback
- Figlet-style ASCII art title screen
- Player trail with fading dots
- Animated exit glow beacon
- Depth-aware wall glyphs
- Victory screen with zone completion breakdown and motion mastery rating
- Zilk 16x16 and Kjammer font support
- CI pipeline for lint, test, build, and cross-platform release

[0.3.0]: https://github.com/tuandzung/vim-rogue-game/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/tuandzung/vim-rogue-game/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/tuandzung/vim-rogue-game/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tuandzung/vim-rogue-game/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/tuandzung/vim-rogue-game/releases/tag/v0.1.0
