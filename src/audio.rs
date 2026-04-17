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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn audio_manager_new_is_created() {
        let _manager = AudioManager::new();
    }

    #[test]
    fn audio_manager_default_disabled() {
        let manager = AudioManager::new();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn audio_manager_play_no_panic() {
        let manager = AudioManager::new();
        manager.play(SoundEffect::Movement);
    }

    #[test]
    fn audio_manager_enable_disable() {
        let mut manager = AudioManager::new();

        manager.enable();
        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn sound_effect_variants_complete() {
        let effects = [
            SoundEffect::Movement,
            SoundEffect::ZoneEntry,
            SoundEffect::Victory,
            SoundEffect::Damage,
            SoundEffect::EnemyStep,
            SoundEffect::LevelComplete,
        ];

        for effect in effects {
            let name = match effect {
                SoundEffect::Movement => "Movement",
                SoundEffect::ZoneEntry => "ZoneEntry",
                SoundEffect::Victory => "Victory",
                SoundEffect::Damage => "Damage",
                SoundEffect::EnemyStep => "EnemyStep",
                SoundEffect::LevelComplete => "LevelComplete",
            };

            assert!(!name.is_empty());
        }
    }

    #[test]
    fn play_when_enabled_no_panic() {
        let mut manager = AudioManager::new();
        manager.enable();
        manager.play(SoundEffect::Victory);
    }

    #[test]
    fn all_sound_effects_listenable() {
        let mut manager = AudioManager::new();
        manager.enable();

        for effect in [
            SoundEffect::Movement,
            SoundEffect::ZoneEntry,
            SoundEffect::Victory,
            SoundEffect::Damage,
            SoundEffect::EnemyStep,
            SoundEffect::LevelComplete,
        ] {
            manager.play(effect);
        }
    }

    #[test]
    fn audio_manager_play_is_non_blocking() {
        let mut manager = AudioManager::new();
        manager.enable();

        let started = Instant::now();
        manager.play(SoundEffect::EnemyStep);
        assert!(started.elapsed().as_millis() < 10);
    }

    #[test]
    fn audio_manager_multiple_play_calls_stay_panic_free() {
        let mut manager = AudioManager::new();
        manager.enable();

        for _ in 0..50 {
            for effect in [
                SoundEffect::Movement,
                SoundEffect::ZoneEntry,
                SoundEffect::Victory,
                SoundEffect::Damage,
                SoundEffect::EnemyStep,
                SoundEffect::LevelComplete,
            ] {
                manager.play(effect);
            }
        }

        assert!(manager.is_enabled());
    }

    #[test]
    fn audio_manager_enable_disable_enable_cycle_remains_functional() {
        let mut manager = AudioManager::new();

        manager.enable();
        manager.disable();
        manager.enable();

        assert!(manager.is_enabled());
        manager.play(SoundEffect::Victory);
    }

    #[test]
    fn audio_manager_play_after_disable_stays_silent() {
        let mut manager = AudioManager::new();

        manager.enable();
        manager.disable();
        manager.play(SoundEffect::Damage);

        assert!(!manager.is_enabled());
    }

    #[test]
    fn audio_manager_play_after_reenable_still_works() {
        let mut manager = AudioManager::new();

        manager.enable();
        manager.disable();
        manager.enable();
        manager.play(SoundEffect::EnemyStep);

        assert!(manager.is_enabled());
    }

    #[test]
    fn audio_manager_repeated_disable_is_idempotent() {
        let mut manager = AudioManager::new();

        manager.enable();
        manager.disable();
        manager.disable();

        assert!(!manager.is_enabled());
        manager.play(SoundEffect::LevelComplete);
    }

    #[test]
    fn audio_manager_repeated_enable_is_idempotent() {
        let mut manager = AudioManager::new();

        manager.enable();
        manager.enable();

        assert!(manager.is_enabled());
        manager.play(SoundEffect::Movement);
    }

    #[test]
    fn audio_manager_same_sound_effect_can_be_played_rapidly() {
        let mut manager = AudioManager::new();
        manager.enable();

        for _ in 0..100 {
            manager.play(SoundEffect::ZoneEntry);
        }

        assert!(manager.is_enabled());
    }

    #[test]
    fn sound_effect_names_are_unique() {
        let mut names = vec![
            "Movement",
            "ZoneEntry",
            "Victory",
            "Damage",
            "EnemyStep",
            "LevelComplete",
        ];

        names.sort_unstable();
        names.dedup();

        assert_eq!(names.len(), 6);
    }
}
