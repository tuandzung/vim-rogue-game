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
