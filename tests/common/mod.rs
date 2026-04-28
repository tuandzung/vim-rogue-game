use std::collections::VecDeque;
use std::time::{Duration, Instant};
use vim_rogue::animation::{AnimationState, AnimationTimer, GameClock, TestClock};
use vim_rogue::audio::AudioManager;
use vim_rogue::map::Map;
use vim_rogue::player::Player;
use vim_rogue::types::*;
use vim_rogue::visibility::VisibilityMap;

#[cfg(debug_assertions)]
use vim_rogue::types::CheatBuffer;

pub fn test_map(width: usize, height: usize) -> Map {
    Map {
        grid: vec![vec![Tile::Floor; width]; height],
        zones: vec![vec![Zone::Zone1; width]; height],
        width,
        height,
        start: Position { x: 0, y: 0 },
        exit: Position { x: width - 1, y: height - 1 },
        enemy_spawns: vec![],
        enemy_patrol_areas: vec![],
    }
}

pub fn started_app_with_map(map: Map, position: Position) -> App {
    let visibility = VisibilityMap::new(map.width, map.height);
    let mut app = App {
        map,
        visibility,
        player: Player::new(position),
        player_animation: None,
        enemy_animations: Vec::new(),
        attack_effects: Vec::new(),
        pending_respawn: None,
        input_queue: Vec::new(),
        enemies: Vec::new(),
        hp: MAX_HP,
        game_state: GameState::Playing,
        pause_selection: PauseOption::Resume,
        started: true,
        pending_input: None,
        start_time: Instant::now(),
        elapsed: Duration::default(),
        final_time: None,
        motion_count: 0,
        status_message: String::new(),
        discovered_motions: Default::default(),
        trail: VecDeque::new(),
        level: 1,
        audio: AudioManager::new(),
        last_checkpoint: None,
        activated_torchlights: Default::default(),
        #[cfg(debug_assertions)]
        cheat_buf: CheatBuffer::new(),
        #[cfg(debug_assertions)]
        cheat_god_mode: false,
    };
    app.update_visibility();
    app
}

pub fn test_app() -> App {
    let map = Map::new();
    App {
        player: Player::new(map.start),
        visibility: VisibilityMap::new(map.width, map.height),
        map,
        player_animation: None,
        enemy_animations: Vec::new(),
        attack_effects: Vec::new(),
        pending_respawn: None,
        input_queue: Vec::new(),
        enemies: Vec::new(),
        hp: MAX_HP,
        game_state: GameState::Playing,
        pause_selection: PauseOption::Resume,
        started: true,
        pending_input: None,
        start_time: Instant::now(),
        elapsed: Duration::default(),
        final_time: None,
        motion_count: 0,
        status_message: String::new(),
        discovered_motions: Default::default(),
        trail: VecDeque::new(),
        level: 1,
        audio: AudioManager::new(),
        last_checkpoint: None,
        activated_torchlights: Default::default(),
        #[cfg(debug_assertions)]
        cheat_buf: CheatBuffer::new(),
        #[cfg(debug_assertions)]
        cheat_god_mode: false,
    }
}

pub fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

pub fn assert_approx_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "expected {expected}, got {actual}");
}

pub fn tick_timer(timer: &mut AnimationTimer, clock: &mut TestClock, delta_ms: f64) {
    let before = clock.elapsed_ms();
    clock.tick(delta_ms);
    timer.update(clock.elapsed_ms() - before);
}

pub fn tick_state(state: &mut AnimationState, clock: &mut TestClock, delta_ms: f64) {
    let before = clock.elapsed_ms();
    clock.tick(delta_ms);
    state.update(clock.elapsed_ms() - before);
}

pub fn all_transparent(_pos: Position) -> bool {
    true
}

pub fn with_walls(walls: &[Position]) -> impl Fn(Position) -> bool + use<'_> {
    move |pos: Position| !walls.contains(&pos)
}

pub fn with_transparent_tiles(
    transparent_tiles: &[Position],
) -> impl Fn(Position) -> bool + use<'_> {
    move |pos: Position| transparent_tiles.contains(&pos)
}
