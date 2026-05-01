use crate::audio::AudioManager;
use crate::map::Map;
use crate::types::*;

impl App {
    /// Internal test seam — do not use outside integration tests.
    #[doc(hidden)]
    pub fn for_test(map: Map, position: Position) -> Self {
        let world = World::new(map);
        let mut app = Self {
            world,
            player: PlayerState::new(position),
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
        };
        app.session.started = true;
        app.update_visibility();
        app
    }
}
