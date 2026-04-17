use std::time::Instant;

pub const PLAYER_MOVE_MS: f64 = 150.0;
pub const ENEMY_MOVE_MS: f64 = 200.0;
pub const EFFECT_MS: f64 = 100.0;

/// Clock abstraction for deterministic testing.
pub trait GameClock {
    fn elapsed_ms(&self) -> f64;
    fn tick(&mut self, delta_ms: f64);
}

/// Real clock using actual time for production.
#[derive(Debug)]
pub struct RealClock {
    start: Instant,
    offset_ms: f64,
}

impl RealClock {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            offset_ms: 0.0,
        }
    }
}

impl Default for RealClock {
    fn default() -> Self {
        Self::new()
    }
}

impl GameClock for RealClock {
    fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0 + self.offset_ms
    }

    fn tick(&mut self, delta_ms: f64) {
        self.offset_ms = (self.offset_ms + delta_ms).max(0.0);
    }
}

/// Test clock with manual time control for deterministic tests.
#[derive(Debug, Clone)]
pub struct TestClock {
    elapsed_ms: f64,
}

impl TestClock {
    pub fn new() -> Self {
        Self { elapsed_ms: 0.0 }
    }
}

impl Default for TestClock {
    fn default() -> Self {
        Self::new()
    }
}

impl GameClock for TestClock {
    fn elapsed_ms(&self) -> f64 {
        self.elapsed_ms
    }

    fn tick(&mut self, delta_ms: f64) {
        self.elapsed_ms = (self.elapsed_ms + delta_ms).max(0.0);
    }
}

/// Animation timer tracking progress from 0.0 to 1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationTimer {
    duration_ms: f64,
    elapsed_ms: f64,
}

impl AnimationTimer {
    pub fn new(duration_ms: f64) -> Self {
        Self {
            duration_ms: duration_ms.max(0.0),
            elapsed_ms: 0.0,
        }
    }

    pub fn progress(&self) -> f64 {
        if self.duration_ms <= 0.0 {
            1.0
        } else {
            (self.elapsed_ms / self.duration_ms).clamp(0.0, 1.0)
        }
    }

    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    pub fn update(&mut self, delta_ms: f64) {
        self.elapsed_ms = (self.elapsed_ms + delta_ms).max(0.0);
    }

    pub fn reset(&mut self) {
        self.elapsed_ms = 0.0;
    }
}

/// Easing functions for smooth interpolation.
pub struct Interpolator;

impl Interpolator {
    pub fn linear(t: f64) -> f64 {
        t.clamp(0.0, 1.0)
    }

    pub fn ease_in_out(t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// Per-entity animation state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationState {
    pub timer: AnimationTimer,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
}

impl AnimationState {
    pub fn new(duration_ms: f64, start: (f64, f64), end: (f64, f64)) -> Self {
        Self {
            timer: AnimationTimer::new(duration_ms),
            start_x: start.0,
            start_y: start.1,
            end_x: end.0,
            end_y: end.1,
        }
    }

    pub fn current_position(&self) -> (f64, f64) {
        let t = Interpolator::ease_in_out(self.timer.progress());
        (
            self.start_x + (self.end_x - self.start_x) * t,
            self.start_y + (self.end_y - self.start_y) * t,
        )
    }

    pub fn update(&mut self, delta_ms: f64) {
        self.timer.update(delta_ms);
    }

    pub fn is_complete(&self) -> bool {
        self.timer.is_complete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx_eq(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    fn tick_timer(timer: &mut AnimationTimer, clock: &mut TestClock, delta_ms: f64) {
        let before = clock.elapsed_ms();
        clock.tick(delta_ms);
        timer.update(clock.elapsed_ms() - before);
    }

    fn tick_state(state: &mut AnimationState, clock: &mut TestClock, delta_ms: f64) {
        let before = clock.elapsed_ms();
        clock.tick(delta_ms);
        state.update(clock.elapsed_ms() - before);
    }

    #[test]
    fn timer_at_zero_is_zero_progress() {
        let timer = AnimationTimer::new(100.0);
        assert_eq!(timer.progress(), 0.0);
        assert!(!timer.is_complete());
    }

    #[test]
    fn timer_at_duration_is_complete() {
        let mut timer = AnimationTimer::new(100.0);
        timer.update(100.0);
        assert_eq!(timer.progress(), 1.0);
        assert!(timer.is_complete());
    }

    #[test]
    fn timer_progress_clamps_at_one() {
        let mut timer = AnimationTimer::new(100.0);
        timer.update(250.0);
        assert_eq!(timer.progress(), 1.0);
    }

    #[test]
    fn interpolator_linear_midpoint() {
        assert_eq!(Interpolator::linear(0.5), 0.5);
    }

    #[test]
    fn interpolator_ease_in_out_starts_slow() {
        assert!(Interpolator::ease_in_out(0.25) < 0.25);
    }

    #[test]
    fn interpolator_ease_in_out_ends_fast() {
        assert!(Interpolator::ease_in_out(0.75) > 0.75);
    }

    #[test]
    fn test_clock_deterministic() {
        let mut first = TestClock::new();
        let mut second = TestClock::new();

        for delta in [16.0, 32.0, 8.0] {
            first.tick(delta);
            second.tick(delta);
        }

        assert_eq!(first.elapsed_ms(), second.elapsed_ms());
    }

    #[test]
    fn animation_state_start_position() {
        let state = AnimationState::new(100.0, (2.0, 3.0), (8.0, 9.0));
        assert_eq!(state.current_position(), (2.0, 3.0));
    }

    #[test]
    fn animation_state_end_position() {
        let mut state = AnimationState::new(100.0, (2.0, 3.0), (8.0, 9.0));
        state.update(100.0);
        assert_eq!(state.current_position(), (8.0, 9.0));
    }

    #[test]
    fn animation_state_midpoint_interpolation() {
        let mut state = AnimationState::new(100.0, (2.0, 3.0), (8.0, 9.0));
        state.update(50.0);
        assert_eq!(state.current_position(), (5.0, 6.0));
    }

    #[test]
    fn timer_progresses_with_small_delta_from_test_clock() {
        let mut clock = TestClock::new();
        let mut timer = AnimationTimer::new(100.0);

        tick_timer(&mut timer, &mut clock, 10.0);

        assert_approx_eq(clock.elapsed_ms(), 10.0);
        assert_approx_eq(timer.progress(), 0.1);
        assert!(!timer.is_complete());
    }

    #[test]
    fn timer_progresses_with_medium_delta_from_test_clock() {
        let mut clock = TestClock::new();
        let mut timer = AnimationTimer::new(200.0);

        tick_timer(&mut timer, &mut clock, 100.0);

        assert_approx_eq(clock.elapsed_ms(), 100.0);
        assert_approx_eq(timer.progress(), 0.5);
        assert!(!timer.is_complete());
    }

    #[test]
    fn timer_progresses_with_large_delta_from_test_clock() {
        let mut clock = TestClock::new();
        let mut timer = AnimationTimer::new(120.0);

        tick_timer(&mut timer, &mut clock, 90.0);

        assert_approx_eq(clock.elapsed_ms(), 90.0);
        assert_approx_eq(timer.progress(), 0.75);
        assert!(!timer.is_complete());
    }

    #[test]
    fn timer_exactly_reaches_complete_progress_at_duration_boundary() {
        let mut clock = TestClock::new();
        let mut timer = AnimationTimer::new(100.0);

        tick_timer(&mut timer, &mut clock, 30.0);
        tick_timer(&mut timer, &mut clock, 20.0);
        tick_timer(&mut timer, &mut clock, 50.0);

        assert_approx_eq(clock.elapsed_ms(), 100.0);
        assert_eq!(timer.progress(), 1.0);
        assert!(timer.is_complete());
    }

    #[test]
    fn timer_overshoot_with_multiple_large_deltas_stays_clamped() {
        let mut clock = TestClock::new();
        let mut timer = AnimationTimer::new(100.0);

        tick_timer(&mut timer, &mut clock, 80.0);
        tick_timer(&mut timer, &mut clock, 60.0);
        tick_timer(&mut timer, &mut clock, 40.0);

        assert_approx_eq(clock.elapsed_ms(), 180.0);
        assert_eq!(timer.progress(), 1.0);
        assert!(timer.is_complete());
    }

    #[test]
    fn interpolator_linear_matches_endpoints_and_midpoint() {
        let mut clock = TestClock::new();

        clock.tick(0.0);

        assert_eq!(clock.elapsed_ms(), 0.0);
        assert_eq!(Interpolator::linear(0.0), 0.0);
        assert_eq!(Interpolator::linear(0.5), 0.5);
        assert_eq!(Interpolator::linear(1.0), 1.0);
    }

    #[test]
    fn interpolator_ease_in_out_matches_known_checkpoints() {
        let mut clock = TestClock::new();

        clock.tick(25.0);

        assert_eq!(clock.elapsed_ms(), 25.0);
        assert_eq!(Interpolator::ease_in_out(0.0), 0.0);
        assert_eq!(Interpolator::ease_in_out(0.25), 0.15625);
        assert_eq!(Interpolator::ease_in_out(0.5), 0.5);
        assert_eq!(Interpolator::ease_in_out(0.75), 0.84375);
        assert_eq!(Interpolator::ease_in_out(1.0), 1.0);
    }

    #[test]
    fn interpolator_ease_in_out_is_symmetric_around_midpoint() {
        let mut clock = TestClock::new();

        clock.tick(50.0);

        let early = Interpolator::ease_in_out(0.25);
        let late = Interpolator::ease_in_out(0.75);

        assert_eq!(clock.elapsed_ms(), 50.0);
        assert_approx_eq(early, 1.0 - late);
    }

    #[test]
    fn test_clock_same_fixed_delta_produces_same_elapsed() {
        let mut first = TestClock::new();
        let mut second = TestClock::new();

        for _ in 0..4 {
            first.tick(16.0);
            second.tick(16.0);
        }

        assert_eq!(first.elapsed_ms(), 64.0);
        assert_eq!(first.elapsed_ms(), second.elapsed_ms());
    }

    #[test]
    fn test_clock_cumulative_ticks_match_total_elapsed() {
        let mut clock = TestClock::new();

        clock.tick(5.0);
        clock.tick(15.0);
        clock.tick(30.0);

        assert_eq!(clock.elapsed_ms(), 50.0);
    }

    #[test]
    fn test_clock_zero_delta_keeps_elapsed_at_zero() {
        let mut clock = TestClock::new();

        clock.tick(0.0);

        assert_eq!(clock.elapsed_ms(), 0.0);
    }

    #[test]
    fn animation_state_creation_preserves_start_and_end_positions() {
        let mut clock = TestClock::new();
        let state = AnimationState::new(160.0, (1.0, 2.0), (7.0, 8.0));

        clock.tick(0.0);

        assert_eq!(clock.elapsed_ms(), 0.0);
        assert_eq!(state.current_position(), (1.0, 2.0));
        assert_eq!((state.start_x, state.start_y), (1.0, 2.0));
        assert_eq!((state.end_x, state.end_y), (7.0, 8.0));
    }

    #[test]
    fn animation_state_update_tracks_quarter_progress_position() {
        let mut clock = TestClock::new();
        let mut state = AnimationState::new(100.0, (1.0, 2.0), (5.0, 10.0));

        tick_state(&mut state, &mut clock, 25.0);

        let (x, y) = state.current_position();
        assert_approx_eq(x, 1.625);
        assert_approx_eq(y, 3.25);
    }

    #[test]
    fn animation_state_update_tracks_halfway_position() {
        let mut clock = TestClock::new();
        let mut state = AnimationState::new(100.0, (1.0, 2.0), (5.0, 10.0));

        tick_state(&mut state, &mut clock, 50.0);

        assert_eq!(state.current_position(), (3.0, 6.0));
    }

    #[test]
    fn animation_state_update_tracks_three_quarter_position() {
        let mut clock = TestClock::new();
        let mut state = AnimationState::new(100.0, (1.0, 2.0), (5.0, 10.0));

        tick_state(&mut state, &mut clock, 75.0);

        let (x, y) = state.current_position();
        assert_approx_eq(x, 4.375);
        assert_approx_eq(y, 8.75);
    }

    #[test]
    fn animation_state_is_complete_when_progress_reaches_one() {
        let mut clock = TestClock::new();
        let mut state = AnimationState::new(120.0, (2.0, 4.0), (8.0, 10.0));

        tick_state(&mut state, &mut clock, 120.0);

        assert!(state.is_complete());
        assert_eq!(state.current_position(), (8.0, 10.0));
    }

    #[test]
    fn animation_state_current_position_is_exact_start_at_zero_progress() {
        let mut clock = TestClock::new();
        let state = AnimationState::new(100.0, (3.0, 4.0), (9.0, 12.0));

        clock.tick(0.0);

        assert_eq!(clock.elapsed_ms(), 0.0);
        assert_eq!(state.current_position(), (3.0, 4.0));
    }

    #[test]
    fn animation_state_current_position_is_exact_end_at_full_progress() {
        let mut clock = TestClock::new();
        let mut state = AnimationState::new(100.0, (3.0, 4.0), (9.0, 12.0));

        tick_state(&mut state, &mut clock, 100.0);

        assert_eq!(state.current_position(), (9.0, 12.0));
    }

    #[test]
    fn animation_states_can_run_sequentially_without_sharing_elapsed_time() {
        let mut clock = TestClock::new();
        let mut first = AnimationState::new(100.0, (0.0, 0.0), (4.0, 0.0));

        tick_state(&mut first, &mut clock, 100.0);
        assert!(first.is_complete());
        assert_eq!(first.current_position(), (4.0, 0.0));

        let mut second = AnimationState::new(80.0, (4.0, 0.0), (4.0, 6.0));
        tick_state(&mut second, &mut clock, 40.0);

        assert_eq!(clock.elapsed_ms(), 140.0);
        assert!(!second.is_complete());
        assert_eq!(second.current_position(), (4.0, 3.0));

        tick_state(&mut second, &mut clock, 40.0);

        assert!(second.is_complete());
        assert_eq!(second.current_position(), (4.0, 6.0));
    }
}
