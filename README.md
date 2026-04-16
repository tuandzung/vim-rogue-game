# vim-quake

A terminal-based roguelike dungeon game that teaches Vim motions through gameplay. Navigate two 80×40 dungeon levels using real Vim keybindings to reach the exit.

## Features

- **2 dungeon levels** with distinct layouts — Level 2 is an inverted maze with obstacles across all zones
- **5 zone-gated areas** per level with distinct color palettes (gray → cyan → magenta → red → gold)
- **Level progression** — stats carry over, trail resets, new map loads on reaching the exit
- **Figlet-style ASCII art** title screen with motion reference
- **Player trail** — fading green dots show your recent path
- **Animated exit glow** — pulsing `►` beacon guides you to the goal
- **Depth-aware walls** — glyph variation (█▓▒#) based on neighbor analysis
- **Victory screen** — ASCII trophy, zone-by-zone completion breakdown with progress bars, and motion mastery rating

## Motions

| Key | Motion | Zone |
|-----|--------|------|
| `h` `j` `k` `l` | Left / Down / Up / Right | 1 |
| `w` `b` | Word forward / back | 2 |
| `0` `$` | Line start / end | 3 |
| `f<char>` `t<char>` | Find / till char | 4 |
| `dd` | Delete obstacle | 5 |

The dungeon is divided into 5 zone-gated areas. Each zone unlocks progressively harder motions. Complete Level 1 to advance to the harder Level 2 maze.

## Quick Start

```bash
cargo run
```

## Controls

- Move with the Vim motions listed above
- `q` / `Esc` — quit
- Any key — start from title screen

Reach the exit (`>`) on each level. Complete Level 2 to win.

## Build & Test

```bash
cargo build    # Compile
cargo test     # Run 80 inline tests
cargo run      # Play
```

## Architecture

```
src/main.rs     Terminal setup + event loop
src/game.rs     App state, event handling, win condition, trail tracking
src/player.rs   Player + 11 motion implementations
src/map.rs      80×40 grid, 5 zones, corridor carving, 2 dungeon levels
src/renderer.rs ratatui TUI rendering (zone colors, trail, glow, ASCII art)
src/types.rs    Shared types (Position, Tile, Zone, VimMotion, App, …)
src/lib.rs      Module re-exports
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| [ratatui](https://crates.io/crates/ratatui) 0.29 | Terminal UI framework |
| [crossterm](https://crates.io/crates/crossterm) 0.28 | Cross-platform terminal control |
| [anyhow](https://crates.io/crates/anyhow) 1.0 | Error handling |

## License

MIT
