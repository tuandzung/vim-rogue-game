# vim-rogue

A roguelike dungeon game with ASCII aesthetic in a graphical window, teaching Vim motions through gameplay. Navigate four 80×40 dungeon levels using real Vim keybindings, dodge enemies, and reach the exit. Built with [bracket-lib](https://github.com/amethyst/bracket-lib) for roguelike-specific rendering, FOV, and tile-based graphics.

## Features

- **Graphical window** with ASCII/CP437 aesthetic — tile-based rendering via bracket-lib
- **4 dungeon levels** with distinct layouts — Level 1 has destroyable obstacles and connecting corridors, Level 2 is an inverted maze with obstacles at corridor gaps, Level 3 is a zigzag descent with enemy patrols, Level 4 is a fortress with FOV-aware patrol enemies and melee combat
- **5 zone-gated areas** per level with distinct color palettes (gray → cyan → magenta → red → gold)
- **Level progression** — stats carry over, trail resets, new map loads on reaching the exit
- **FOV-aware enemy AI** — enemies chase via BFS when they see you (within their FOV radius), patrol their room when you're hidden
- **Enemy encounters** — Level 3+ spawns enemies; Level 4 enemies have HP and patrol rooms
- **HP system** — enemy collisions deal 10 damage (MAX_HP=30); health bar in sidebar with color coding
- **Melee combat** — press `x` to attack adjacent enemies on Level 4 (3 hits to kill)
- **Torchlight checkpoints** — step on torchlights for permanent illumination and respawn points
- **Lives and retry** — you start with 3 lives; enemy collisions cost a life, losing all lives triggers a loss screen, and any key retries the current level
- **Fog of war** — unexplored areas are hidden; explored tiles persist dimly when out of view
- **Minimap** — scaled-down view of explored areas in the sidebar
- **Smooth animations** — ease-in-out interpolation for player and enemy movement (150ms / 200ms)
- **Sound effects** — audio events for movement, zone entry, victory, and combat (graceful silent fallback)
- **Figlet-style ASCII art** title screen with motion reference
- **Player trail** — fading green dots show your recent path
- **Animated exit glow** — pulsing `►` beacon guides you to the goal
- **Depth-aware walls** — glyph variation (█▓▒#) based on neighbor analysis
- **Victory screen** — ASCII trophy, zone-by-zone completion breakdown with progress bars, and motion mastery rating (up to 13 motions)
- **Pause menu** — press `Esc` or `q` to pause; choose Resume, Retry Level, or Quit Game (navigate with `j`/`k` or arrow keys)

## Motions

| Key | Motion | Zone |
|-----|--------|------|
| `h` `j` `k` `l` | Left / Down / Up / Right | 1 |
| `w` `b` | Word forward / back | 2 |
| `0` `$` `G` `gg` | Line start / end / last row / first row | 3 |
| `f<char>` `t<char>` | Find / till char | 4 |
| `dd` | Delete obstacle | 5 |

The dungeon is divided into 5 zone-gated areas. Each zone unlocks progressively harder motions. Level 1 teaches basic movement, Level 2 adds obstacles, Level 3 introduces enemies, and Level 4 adds FOV-aware patrol enemies with melee combat.

## Quick Start

```bash
cargo run
```

Opens a graphical window (80×50 character grid). Requires a display — not a terminal UI.

## Controls

- Move with the Vim motions listed above
- `Esc` / `q` — open pause menu (Resume, Retry Level, Quit Game)
- `j`/`k` or `↑`/`↓` — navigate pause menu
- `Enter` — select pause menu option
- Any key — start from title screen

Reach the exit (`>`) on each level. Complete all 4 levels to win. Lose all lives and you can retry the current level with a fresh map.

## Build & Test

```bash
cargo build            # Compile
cargo test             # Run 385 integration tests
cargo fmt              # Format code (uses rustfmt.toml)
cargo fmt --check      # Check formatting without writing
cargo clippy           # Lint
cargo run              # Play
```

## Architecture

```
src/main.rs       bracket-lib BTerm setup + GameState event loop, quit handling
src/game.rs       App state, input handling, FOV-gated enemy turns, win/loss/retry, pause menu, trail, audio, animation
src/player.rs     Player + 13 motion implementations
src/map.rs        80×40 grid, 5 zones, corridor carving, 4 dungeon levels, enemy spawn points + patrol areas
src/renderer.rs   bracket-lib rendering: title, viewport, sidebar, minimap, win/loss/pause screens, fog of war
src/types.rs      Shared types (Position, Tile, Zone, VimMotion, Enemy, PatrolArea, GameState, PauseOption, App, …)
src/animation.rs  Animation timers, ease-in-out interpolation, deterministic TestClock
src/visibility.rs FOV ray-casting, explored tile tracking (Hidden/Explored/Visible)
src/enemy.rs      Enemy struct with FOV-aware BFS chase and room patrol behavior
src/audio.rs      AudioManager with graceful silent fallback
src/lib.rs        Module re-exports
```

### Key Design Decisions

- **`renderer.rs` is read-only** — never mutates game state
- **Animation state on `App`** — separate from Player/Enemy structs (presentation concern)
- **Deterministic timing** — `TestClock` for tests, `RealClock` for production (via `GameClock` trait)
- **FOV-aware enemy AI** — enemies use Bresenham line-of-sight within their FOV radius to detect the player; they chase via BFS when visible and patrol their room when not
- **Audio disabled by default** — `AudioManager::enable()` to activate; silent when unavailable

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [bracket-lib](https://crates.io/crates/bracket-lib) | 0.8.7 | Graphical window, CP437 rendering, roguelike utilities |
| [anyhow](https://crates.io/crates/anyhow) | 1.0 | Error handling |

## License

MIT
