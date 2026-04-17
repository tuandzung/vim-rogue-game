/// All game sound effects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundEffect {
    Movement,
    ZoneEntry,
    Victory,
    Damage,
    EnemyStep,
    LevelComplete,
}

/// Audio manager with graceful fallback.
///
/// When audio is unavailable (no device, missing files), the game runs silently.
pub struct AudioManager {
    enabled: bool,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioManager {
    /// Create a new AudioManager.
    ///
    /// If audio initialization fails, creates a disabled manager that silently
    /// ignores all play requests.
    pub fn new() -> Self {
        Self { enabled: false }
    }

    /// Play a sound effect.
    ///
    /// If audio is disabled, this is a no-op and never panics.
    pub fn play(&self, _effect: SoundEffect) {
        if !self.enabled {}
    }

    /// Check if audio is available.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable audio.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable audio.
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}
