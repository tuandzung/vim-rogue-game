use vim_rogue::animation::{AnimationState, AnimationTimer, GameClock, TestClock};
use vim_rogue::map::Map;
use vim_rogue::types::*;

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
