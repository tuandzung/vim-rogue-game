# Spec: Split App into Domain Aggregates

4 incremental steps. Each step passes all 393 tests before proceeding.

## Step 1: Create Aggregate Structs, Move Fields

Mechanical field migration. No behavior changes. All `pub` fields remain `pub`.

### types.rs — new structs

```rust
pub struct World {
    pub map: Map,
    pub visibility: VisibilityMap,
    pub enemies: Vec<Enemy>,
    pub activated_torchlights: HashSet<Position>,
}

pub struct PlayerState {
    pub inner: Player,
    pub hp: i32,
    pub trail: VecDeque<Position>,
    pub motion_count: usize,
    pub discovered_motions: HashSet<VimMotion>,
    pub level: usize,
    pub last_checkpoint: Option<Position>,
    pub pending_respawn: Option<Position>,
}

pub struct InputState {
    pub input_queue: Vec<(VirtualKeyCode, bool)>,
    pub pending_input: Option<PendingInput>,
}

pub struct Session {
    pub game_state: GameState,
    pub pause_selection: PauseOption,
    pub started: bool,
    pub status_message: String,
    pub start_time: Instant,
    pub elapsed: Duration,
    pub final_time: Option<Duration>,
}
```

### types.rs — new App

```rust
pub struct App {
    pub world: World,
    pub player: PlayerState,
    pub input: InputState,
    pub session: Session,
    // Animation stays flat (candidate 2 extracts these)
    pub player_animation: Option<AnimationState>,
    pub enemy_animations: Vec<(usize, AnimationState)>,
    pub attack_effects: Vec<AttackEffect>,
    pub audio: AudioManager,
    #[cfg(debug_assertions)]
    pub cheat_buf: CheatBuffer,
    #[cfg(debug_assertions)]
    pub cheat_god_mode: bool,
}
```

9 fields (down from 29). Animation (3 fields) + audio + debug (2) = remaining flat fields.

### Constructor methods

Each aggregate gets a `new(...)` that sets sensible defaults:

- `World::new_for_level(level: usize) -> Self` — builds Map::level(n), VisibilityMap::new, spawns enemies, empty torchlights
- `PlayerState::new(position: Position) -> Self` — Player::new(pos), MAX_HP, empty trail, level 1
- `InputState::new() -> Self` — empty queue, no pending
- `Session::new() -> Self` — Playing, Resume, started=false, zero timing

`App::new()` becomes:
```rust
pub fn new() -> Self {
    let world = World::new_for_level(1);
    let start = world.map.start;
    Self {
        world,
        player: PlayerState::new(start),
        input: InputState::new(),
        session: Session::new(),
        player_animation: None,
        enemy_animations: Vec::new(),
        attack_effects: Vec::new(),
        audio: AudioManager::new(),
        #[cfg(debug_assertions)]
        cheat_buf: CheatBuffer::new(),
        #[cfg(debug_assertions)]
        cheat_god_mode: false,
    }
}
```

### Accessor migration (find-replace across codebase)

Every file that touches App fields changes:

| Before | After |
|--------|-------|
| `app.map` | `app.world.map` |
| `app.visibility` | `app.world.visibility` |
| `app.enemies` | `app.world.enemies` |
| `app.activated_torchlights` | `app.world.activated_torchlights` |
| `app.player.position` | `app.player.inner.position` |
| `app.player.handle_motion(...)` | `app.player.inner.handle_motion(...)` |
| `app.hp` | `app.player.hp` |
| `app.trail` | `app.player.trail` |
| `app.motion_count` | `app.player.motion_count` |
| `app.discovered_motions` | `app.player.discovered_motions` |
| `app.level` | `app.player.level` |
| `app.last_checkpoint` | `app.player.last_checkpoint` |
| `app.pending_respawn` | `app.player.pending_respawn` |
| `app.input_queue` | `app.input.input_queue` |
| `app.pending_input` | `app.input.pending_input` |
| `app.game_state` | `app.session.game_state` |
| `app.pause_selection` | `app.session.pause_selection` |
| `app.started` | `app.session.started` |
| `app.status_message` | `app.session.status_message` |
| `app.start_time` | `app.session.start_time` |
| `app.elapsed` | `app.session.elapsed` |
| `app.final_time` | `app.session.final_time` |

### Files changed

- `src/types.rs` — new structs + modified App
- `src/game.rs` — all accessor changes (largest diff)
- `src/renderer.rs` — all accessor changes
- `src/main.rs` — App::new() call (if it constructs App)
- `tests/common/mod.rs` — test_app(), started_app_with_map()
- `tests/game.rs` — accessor changes
- `tests/renderer.rs` — accessor changes
- `tests/map.rs` — minor (tests Map directly, not App)

### Helper methods to add to App for common compound accessors

These reduce churn in step 2+:

```rust
impl App {
    // Convenience: current map start position
    pub fn map_start(&self) -> Position { self.world.map.start }

    // Convenience: update visibility for current player position
    pub fn update_visibility(&mut self) {
        let pos = self.player.inner.position;
        let torchlights = &self.world.activated_torchlights;
        self.world.visibility.demote_visible_to_explored();
        self.world.visibility.compute_fov(pos, FOV_RADIUS, |p| {
            self.world.map.get_tile(p.x, p.y).is_transparent()
        });
        for tp in torchlights {
            if self.world.visibility.is_visible(*tp) {
                self.world.visibility.compute_fov(*tp, TORCHLIGHT_FOV_RADIUS, |p| {
                    self.world.map.get_tile(p.x, p.y).is_transparent()
                });
            }
        }
    }
}
```

### Verification

- `cargo fmt --check` clean
- `cargo clippy` zero warnings
- `cargo test` — all 393 pass

---

## Step 2: Move Reset Logic into Aggregate Methods

Extract duplicated reset sequences from `advance_level` and `retry_level`.

### World methods

```rust
impl World {
    /// Reset terrain, visibility, enemies for a new level
    pub fn reset_for_level(&mut self, level: usize) {
        self.map = Map::level(level);
        if self.visibility.width() != self.map.width
            || self.visibility.height() != self.map.height
        {
            self.visibility = VisibilityMap::new(self.map.width, self.map.height);
        }
        self.visibility.reset();
        self.enemies = Vec::new();
        self.activated_torchlights.clear();
    }

    /// Spawn enemies appropriate for current map
    pub fn spawn_enemies(&mut self) {
        self.enemies = self.map.enemy_spawns.iter()
            .map(|&pos| Enemy::new(pos))
            .collect();
        // Level 4: assign patrol areas and HP
        // ... (move logic from App::spawn_enemies_for_current_level)
    }
}
```

### PlayerState methods

```rust
impl PlayerState {
    /// Advance to next level, reset position
    pub fn advance_level(&mut self, level: usize, start: Position) {
        self.level = level;
        self.inner.position = start;
        self.trail.clear();
        self.last_checkpoint = None;
        self.pending_respawn = None;
    }

    /// Retry current level, full heal
    pub fn retry_level(&mut self, start: Position) {
        self.inner.position = start;
        self.hp = MAX_HP;
        self.trail.clear();
        self.last_checkpoint = None;
        self.pending_respawn = None;
    }
}
```

### App::advance_level becomes

```rust
pub fn advance_level(&mut self) {
    let next_level = self.player.level + 1;
    self.player.advance_level(next_level, self.world.map.start);
    self.world.reset_for_level(next_level);
    self.world.spawn_enemies();
    self.world.update_visibility(self.player.inner.position);
    self.player_animation = None;
    self.enemy_animations.clear();
    self.attack_effects.clear();
    self.input.clear();
    self.session.status_message = format!("Level {} — The dungeon shifts around you...", next_level);
}
```

### App::retry_level becomes

```rust
pub fn retry_level(&mut self) {
    self.player.retry_level(self.world.map.start);
    self.world.reset_for_level(self.player.level);
    self.world.spawn_enemies();
    self.world.update_visibility(self.player.inner.position);
    self.player_animation = None;
    self.enemy_animations.clear();
    self.attack_effects.clear();
    self.input.clear();
    self.session.game_state = GameState::Playing;
    self.session.status_message = format!("Level {} — Try again!", self.player.level);
}
```

DRY violation eliminated — each aggregate owns its reset.

### Verification

- `cargo test` — all 393 pass
- `advance_level` and `retry_level` in game.rs are now ~10 lines each (down from ~25)

---

## Step 3: Move Game Logic into World

Move `update_visibility`, `spawn_enemies_for_current_level`, `enemies_step` from game.rs into World.

### World::update_visibility

```rust
impl World {
    pub fn update_visibility(&mut self, player_pos: Position) {
        self.visibility.demote_visible_to_explored();
        self.visibility.compute_fov(player_pos, FOV_RADIUS, |p| {
            self.map.get_tile(p.x, p.y).is_transparent()
        });
        for &tp in &self.activated_torchlights {
            if self.visibility.is_visible(tp) {
                self.visibility.compute_fov(tp, TORCHLIGHT_FOV_RADIUS, |p| {
                    self.map.get_tile(p.x, p.y).is_transparent()
                });
            }
        }
    }
}
```

App::update_visibility becomes `self.world.update_visibility(self.player.inner.position)`.

### World::step_enemies

Move the core loop from `game.rs::enemies_step`. Returns collision info so App can coordinate damage:

```rust
pub struct EnemyCollision {
    pub enemy_index: usize,
    pub enemy_pos: Position,
}

impl World {
    /// Move all enemies. Returns collisions with player_pos.
    pub fn step_enemies(&mut self, player_pos: Position) -> Vec<EnemyCollision> {
        let mut collisions = Vec::new();
        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            let moved = if enemy.has_line_of_sight(player_pos, &self.map) {
                enemy.step_toward_player(player_pos, &self.map)
            } else {
                enemy.patrol_step(&self.map)
            };
            if moved && enemy.position == player_pos {
                collisions.push(EnemyCollision { enemy_index: i, enemy_pos: enemy.position });
            }
        }
        collisions
    }
}
```

App coordinates the cross-aggregate flow:
```rust
fn enemies_step(&mut self) {
    let collisions = self.world.step_enemies(self.player.inner.position);
    for col in collisions {
        // damage
        // animation
        // status message
        // death/respawn check
    }
}
```

### spawn_enemies_for_current_level → World::spawn_enemies

Already covered in step 2. Move the level-specific enemy configuration logic.

### Verification

- `cargo test` — all 393 pass
- game.rs shrinks by ~80 lines (enemies_step, update_visibility, spawn logic moved)

---

## Step 4: Extract status_message on PlayerState

### PlayerState::status_message

```rust
impl PlayerState {
    /// Status message for the last player action, if any
    pub fn status_message(&self) -> Option<String> {
        // Motion discovery messages
        // Damage/death messages
        // Checkpoint messages
        // Respawn messages
    }
}
```

Note: Not all status messages come from PlayerState. Level transition messages come from App coordination. Session still owns the `status_message` field — PlayerState just provides content through its method.

The pattern becomes:
```rust
// In game.rs, after a player action:
if let Some(msg) = self.player.status_message() {
    self.session.status_message = msg;
}
// For coordination-level messages:
self.session.status_message = format!("Level {}...", self.player.level);
```

### Verification

- `cargo test` — all 393 pass
- `status_message` assignments in game.rs reduced from 24 to ~15 direct sets + method calls

---

## Test Impact Summary

| Test file | Step 1 | Step 2 | Step 3 | Step 4 |
|-----------|--------|--------|--------|--------|
| common/mod.rs | Heavy (constructors rewrite) | Minor (use new methods) | None | Minor |
| game.rs (140 tests) | Heavy (accessors) | Minor | Minor | Minor |
| renderer.rs (53 tests) | Medium (accessors) | None | None | None |
| player.rs (29 tests) | Medium (accessors) | None | None | Minor |
| enemy.rs (21 tests) | Light | None | Light (World API) | None |
| map.rs (46 tests) | None | None | None | None |
| visibility.rs (29 tests) | Light | None | Light (World API) | None |
| animation.rs (34 tests) | Light (accessors) | None | None | None |
| types.rs (25 tests) | Medium (struct changes) | None | None | None |
| audio.rs (16 tests) | Light | None | None | None |

Step 1 is the big mechanical migration. Steps 2-4 are surgical.
