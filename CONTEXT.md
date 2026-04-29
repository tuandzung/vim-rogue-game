# vim-rogue Domain Context

## Aggregates

The game state is organized into domain aggregates, each owning a cohesive slice of state and behavior. `App` is a thin coordinator — it sequences cross-aggregate flows but owns no logic itself.

### World
The dungeon environment: terrain, visibility, enemies, and torchlight state. All spatial queries go through World. Enemy AI (BFS chase, patrol) runs inside World. FOV computation is World's responsibility — it knows the map and who can see what.

- Owns: map, visibility, enemies, activated torchlights
- Key invariant: visibility is always consistent with current map state
- Reset: `new_for_level(n)` replaces terrain, enemies, visibility for a dungeon level

### PlayerState
The player's identity and progression: position, health, trail of visited positions, discovered Vim motions, current level, checkpoint, and pending respawn. PlayerState knows *why* the player needs to respawn, not just where.

- Owns: position, hp, trail, motion tracking, level, checkpoint, pending_respawn
- Key invariant: hp <= MAX_HP, trail length <= TRAIL_MAX
- Produces status messages for actions it owns (motion feedback, damage, respawn)

### InputState
Input buffering for multi-key Vim commands. Tracks pending two-phase input (f/t/dd/gg) and queued keys during animation.

- Owns: input_queue, pending_input
- Key invariant: pending_input is consumed before input_queue is processed

### Session
The meta-layer: game lifecycle state, timing, pause menu selection, and the current status message. Session knows *whether* the game is playing, not *what* the player is doing.

- Owns: game_state, pause_selection, started, status_message, timing
- Key invariant: timing only advances when game_state is Playing

### App (Coordinator)
Sequences cross-aggregate flows: level transitions, enemy collision → damage → death/respawn, pause/resume. No business logic — calls aggregate methods in the right order.

## Domain Rules

- Levels: 4 dungeon levels, each with distinct layout. Level exit → advance_level. Death → retry_level (same layout). 0 hp → game over.
- FOV: player has FOV_RADIUS=10. Torchlight checkpoints grant permanent TORCHLIGHT_FOV_RADIUS=6. Enemy FOV radius is 8.
- Combat: enemy collision = -1 life (or -HP on level 4). Level 4 enemies have HP and can be meleed (3 hits kill).
- Checkpoints: torchlight activation saves position. Death with checkpoint → respawn there instead of level start.
- Motions: 13 Vim keys. Single-key motions fire immediately; f/t/dd/gg use two-phase input via InputState.

## Naming

- "World" not "MapContainer" or "GameState" — it's the dungeon as environment
- "PlayerState" not "Player" — the inner `Player` struct handles motion, PlayerState wraps it with progression
- "Session" not "UIState" — timing and lifecycle aren't UI concerns
