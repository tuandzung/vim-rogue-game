# vim-quake

A roguelike dungeon game with ASCII aesthetic in a graphical window, teaching Vim motions through gameplay. Navigate three 80×40 dungeon levels using real Vim keybindings, dodge enemies, and reach the exit. Built with [bracket-lib](https://github.com/amethyst/bracket-lib) for roguelike-specific rendering, FOV, and tile-based graphics.

## Features

- **Graphical window** with ASCII/CP437 aesthetic — tile-based rendering via bracket-lib
- **3 dungeon levels** with distinct layouts — Level 2 is an inverted maze, Level 3 is a zigzag descent with enemy patrols
- **5 zone-gated areas** per level with distinct color palettes (gray → cyan → magenta → red → gold)
- **Level progression** — stats carry over, trail resets, new map loads on reaching the exit
- **Enemy encounters** — Level 3 spawns BFS-chasing enemies that step toward you each turn
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

## Motions

| Key | Motion | Zone |
|-----|--------|------|
| `h` `j` `k` `l` | Left / Down / Up / Right | 1 |
| `w` `b` | Word forward / back | 2 |
| `0` `$` `G` `gg` | Line start / end / last row / first row | 3 |
| `f<char>` `t<char>` | Find / till char | 4 |
| `dd` | Delete obstacle | 5 |

The dungeon is divided into 5 zone-gated areas. Each zone unlocks progressively harder motions. Level 1 teaches basic movement, Level 2 adds obstacles, and Level 3 introduces enemies.

## Quick Start

```bash
cargo run
```

Opens a graphical window (80×50 character grid). Requires a display — not a terminal UI.

## Controls

- Move with the Vim motions listed above
- `q` / `Esc` — quit
- Any key — start from title screen

Reach the exit (`>`) on each level. Complete all 3 levels to win. Lose all lives and you can retry the current level with a fresh map.

## Build & Test

```bash
cargo build    # Compile
cargo test     # Run 275 inline tests
cargo run      # Play
```

## Architecture

```
src/main.rs       bracket-lib BTerm setup + GameState event loop
src/game.rs       App state, input handling, enemy turns, win/loss, trail, audio, animation
src/player.rs     Player + 13 motion implementations
src/map.rs        80×40 grid, 5 zones, corridor carving, 3 dungeon levels, enemy spawn points
src/renderer.rs   bracket-lib rendering: title, viewport, sidebar, minimap, win/loss screens, fog of war
src/types.rs      Shared types (Position, Tile, Zone, VimMotion, Enemy, GameState, App, …)
src/animation.rs  Animation timers, ease-in-out interpolation, deterministic TestClock
src/visibility.rs FOV ray-casting, explored tile tracking (Hidden/Explored/Visible)
src/enemy.rs      Enemy struct with BFS pathfinding toward the player
src/audio.rs      AudioManager with graceful silent fallback
src/lib.rs        Module re-exports
```

### Key Design Decisions

- **`renderer.rs` is read-only** — never mutates game state
- **Animation state on `App`** — separate from Player/Enemy structs (presentation concern)
- **Deterministic timing** — `TestClock` for tests, `RealClock` for production (via `GameClock` trait)
- **FOV is visual-only** — fog of war doesn't affect enemy AI behavior
- **Audio disabled by default** — `AudioManager::enable()` to activate; silent when unavailable

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [bracket-lib](https://crates.io/crates/bracket-lib) | 0.8.7 | Graphical window, CP437 rendering, roguelike utilities |
| [anyhow](https://crates.io/crates/anyhow) | 1.0 | Error handling |

## License

MIT
