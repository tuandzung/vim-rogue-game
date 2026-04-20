use std::collections::VecDeque;
use std::time::Instant;

use bracket_lib::prelude::VirtualKeyCode;

use crate::animation::{AnimationState, ENEMY_MOVE_MS, PLAYER_MOVE_MS};
use crate::audio::SoundEffect;
use crate::map::Map;
use crate::player::Player;
use crate::types::{App, Enemy, FOV_RADIUS, GameState, PauseOption, PendingInput, Tile, VimMotion};
use crate::visibility::VisibilityMap;

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let map = Map::new();
        let player = Player::new(map.start);
        let mut app = Self {
            map,
            visibility: VisibilityMap::new(80, 40),
            player,
            player_animation: None,
            enemy_animations: Vec::new(),
            input_queue: Vec::new(),
            enemies: Vec::new(),
            lives: 3,
            game_state: GameState::Playing,
            pause_selection: PauseOption::Resume,
            started: false,
            pending_input: None,
            start_time: Instant::now(),
            elapsed: Default::default(),
            final_time: None,
            motion_count: 0,
            status_message: String::from(
                "Explore the dungeon and practice the highlighted motions.",
            ),
            discovered_motions: Default::default(),
            trail: VecDeque::new(),
            level: 1,
            audio: crate::audio::AudioManager::new(),
        };
        app.update_visibility();
        app
    }

    pub fn refresh_time(&mut self) {
        if self.started && self.game_state == GameState::Playing {
            self.elapsed = self.start_time.elapsed();
        }
    }

    pub fn current_zone(&self) -> crate::types::Zone {
        self.map.zone_at(self.player.position)
    }

    pub fn unique_motions(&self) -> usize {
        self.discovered_motions.len()
    }

    pub fn update_visibility(&mut self) {
        if self.visibility.width() != self.map.width || self.visibility.height() != self.map.height
        {
            self.visibility = VisibilityMap::new(self.map.width, self.map.height);
        }

        let player_position = self.player.position;
        let map = &self.map;

        self.visibility.demote_visible_to_explored();
        self.visibility
            .compute_fov(player_position, FOV_RADIUS, |pos| {
                matches!(
                    map.get_tile(pos.x, pos.y),
                    Tile::Floor | Tile::Exit | Tile::Obstacle
                )
            });
    }

    pub fn advance_level(&mut self) {
        self.level += 1;
        self.map = Map::level(self.level);
        self.player.position = self.map.start;
        self.player_animation = None;
        self.enemy_animations.clear();
        self.input_queue.clear();
        self.enemies = self
            .map
            .enemy_spawns
            .iter()
            .map(|&pos| Enemy::new(pos))
            .collect();
        self.trail.clear();
        self.pending_input = None;
        if self.visibility.width() != self.map.width || self.visibility.height() != self.map.height
        {
            self.visibility = VisibilityMap::new(self.map.width, self.map.height);
        }
        self.visibility.reset();
        self.update_visibility();
        self.status_message = format!("Level {} — The dungeon shifts around you...", self.level);
    }

    pub fn retry_level(&mut self) {
        self.map = Map::level(self.level);
        self.player.position = self.map.start;
        self.player_animation = None;
        self.enemy_animations.clear();
        self.input_queue.clear();
        self.enemies = self
            .map
            .enemy_spawns
            .iter()
            .map(|&pos| Enemy::new(pos))
            .collect();
        self.lives = 3;
        self.trail.clear();
        self.pending_input = None;
        self.game_state = GameState::Playing;
        if self.visibility.width() != self.map.width || self.visibility.height() != self.map.height
        {
            self.visibility = VisibilityMap::new(self.map.width, self.map.height);
        }
        self.visibility.reset();
        self.update_visibility();
        self.status_message = format!("Level {} — Try again!", self.level);
    }
}

pub fn tick(app: &mut App, delta_ms: f64) {
    if app.game_state == GameState::Paused {
        return;
    }

    if let Some(animation) = app.player_animation.as_mut() {
        animation.update(delta_ms);
        if animation.is_complete() {
            app.player_animation = None;
        }
    }

    for (_, animation) in &mut app.enemy_animations {
        animation.update(delta_ms);
    }
    app.enemy_animations
        .retain(|(_, animation)| !animation.is_complete());

    while app.player_animation.is_none() && !app.input_queue.is_empty() {
        let (key, shift) = app.input_queue.remove(0);
        handle_key(app, key, shift);

        if matches!(app.game_state, GameState::Quit | GameState::Won) {
            app.input_queue.clear();
            break;
        }
    }
}

fn vkey_to_char(key: VirtualKeyCode, shift: bool) -> Option<char> {
    match (key, shift) {
        (VirtualKeyCode::Period, false) => Some('.'),
        (VirtualKeyCode::Period, true) => Some('>'),
        (VirtualKeyCode::Key3, true) => Some('#'),
        (VirtualKeyCode::A, false) => Some('a'),
        (VirtualKeyCode::B, false) => Some('b'),
        (VirtualKeyCode::C, false) => Some('c'),
        (VirtualKeyCode::D, false) => Some('d'),
        (VirtualKeyCode::E, false) => Some('e'),
        (VirtualKeyCode::F, false) => Some('f'),
        (VirtualKeyCode::G, false) => Some('g'),
        (VirtualKeyCode::H, false) => Some('h'),
        (VirtualKeyCode::I, false) => Some('i'),
        (VirtualKeyCode::J, false) => Some('j'),
        (VirtualKeyCode::K, false) => Some('k'),
        (VirtualKeyCode::L, false) => Some('l'),
        (VirtualKeyCode::M, false) => Some('m'),
        (VirtualKeyCode::N, false) => Some('n'),
        (VirtualKeyCode::O, false) => Some('o'),
        (VirtualKeyCode::P, false) => Some('p'),
        (VirtualKeyCode::Q, false) => Some('q'),
        (VirtualKeyCode::R, false) => Some('r'),
        (VirtualKeyCode::S, false) => Some('s'),
        (VirtualKeyCode::T, false) => Some('t'),
        (VirtualKeyCode::U, false) => Some('u'),
        (VirtualKeyCode::V, false) => Some('v'),
        (VirtualKeyCode::W, false) => Some('w'),
        (VirtualKeyCode::X, false) => Some('x'),
        (VirtualKeyCode::Y, false) => Some('y'),
        (VirtualKeyCode::Z, false) => Some('z'),
        (VirtualKeyCode::Key0, false) => Some('0'),
        (VirtualKeyCode::Key1, false) => Some('1'),
        (VirtualKeyCode::Key2, false) => Some('2'),
        (VirtualKeyCode::Key3, false) => Some('3'),
        (VirtualKeyCode::Key4, false) => Some('4'),
        (VirtualKeyCode::Key5, false) => Some('5'),
        (VirtualKeyCode::Key6, false) => Some('6'),
        (VirtualKeyCode::Key7, false) => Some('7'),
        (VirtualKeyCode::Key8, false) => Some('8'),
        (VirtualKeyCode::Key9, false) => Some('9'),
        _ => None,
    }
}

pub fn handle_key(app: &mut App, key: VirtualKeyCode, shift: bool) {
    if !app.started {
        app.started = true;
        app.start_time = Instant::now();
        app.elapsed = Default::default();
        app.status_message =
            String::from("Use hjkl to move. Every motion is available from the start.");
        return;
    }

    if app.player_animation.is_some() {
        app.input_queue.push((key, shift));
        return;
    }

    if app.game_state == GameState::Paused {
        match key {
            VirtualKeyCode::Escape => {
                app.game_state = GameState::Playing;
            }
            VirtualKeyCode::Up | VirtualKeyCode::K if !shift => {
                app.pause_selection = app.pause_selection.prev();
            }
            VirtualKeyCode::Down | VirtualKeyCode::J if !shift => {
                app.pause_selection = app.pause_selection.next();
            }
            VirtualKeyCode::Return => match app.pause_selection {
                PauseOption::Resume => app.game_state = GameState::Playing,
                PauseOption::RetryLevel => app.retry_level(),
                PauseOption::QuitGame => app.game_state = GameState::Quit,
            },
            _ => {}
        }
        return;
    }

    if matches!(key, VirtualKeyCode::Escape) || (key == VirtualKeyCode::Q && !shift) {
        app.input_queue.clear();
        app.pending_input = None;
        app.game_state = GameState::Paused;
        app.pause_selection = PauseOption::Resume;
        return;
    }

    if app.game_state == GameState::Won {
        return;
    }

    if app.game_state == GameState::Lost {
        app.retry_level();
        return;
    }

    if let Some(pending) = app.pending_input {
        app.pending_input = None;
        match pending {
            PendingInput::Find => {
                if let Some(target) = vkey_to_char(key, shift) {
                    execute_motion(app, VimMotion::Find, Some(target));
                }
            }
            PendingInput::Till => {
                if let Some(target) = vkey_to_char(key, shift) {
                    execute_motion(app, VimMotion::Till, Some(target));
                }
            }
            PendingInput::Delete => {
                if key == VirtualKeyCode::D && !shift {
                    execute_motion(app, VimMotion::DeleteLine, None);
                } else {
                    app.status_message = String::from("dd needs a second d. Command cancelled.");
                }
            }
            PendingInput::GotoLine => {
                if key == VirtualKeyCode::G && !shift {
                    execute_motion(app, VimMotion::GotoLine, None);
                } else {
                    app.status_message = String::from("gg needs a second g. Command cancelled.");
                }
            }
        }
        return;
    }

    match parse_motion(key, shift) {
        Some(ParsedInput::Immediate(motion)) => execute_motion(app, motion, None),
        Some(ParsedInput::AwaitTarget(pending)) => {
            app.pending_input = Some(pending);
            app.status_message = match pending {
                PendingInput::Find => {
                    String::from("Find: type the target tile character (., #, >, ▒).")
                }
                PendingInput::Till => {
                    String::from("Till: type the target tile character to stop one tile before it.")
                }
                PendingInput::Delete => {
                    String::from("Press d again to break the nearest obstacle on this row.")
                }
                PendingInput::GotoLine => String::from("Press g again to jump to the first row."),
            };
        }
        None => {}
    }
}

enum ParsedInput {
    Immediate(VimMotion),
    AwaitTarget(PendingInput),
}

fn parse_motion(key: VirtualKeyCode, shift: bool) -> Option<ParsedInput> {
    match key {
        VirtualKeyCode::H => Some(ParsedInput::Immediate(VimMotion::H)),
        VirtualKeyCode::J => Some(ParsedInput::Immediate(VimMotion::J)),
        VirtualKeyCode::K => Some(ParsedInput::Immediate(VimMotion::K)),
        VirtualKeyCode::L => Some(ParsedInput::Immediate(VimMotion::L)),
        VirtualKeyCode::W => Some(ParsedInput::Immediate(VimMotion::W)),
        VirtualKeyCode::B => Some(ParsedInput::Immediate(VimMotion::B)),
        VirtualKeyCode::Key0 if !shift => Some(ParsedInput::Immediate(VimMotion::Zero)),
        VirtualKeyCode::Key4 if shift => Some(ParsedInput::Immediate(VimMotion::Dollar)),
        VirtualKeyCode::F if !shift => Some(ParsedInput::AwaitTarget(PendingInput::Find)),
        VirtualKeyCode::T if !shift => Some(ParsedInput::AwaitTarget(PendingInput::Till)),
        VirtualKeyCode::D if !shift => Some(ParsedInput::AwaitTarget(PendingInput::Delete)),
        VirtualKeyCode::G if shift => Some(ParsedInput::Immediate(VimMotion::G)),
        VirtualKeyCode::G if !shift => Some(ParsedInput::AwaitTarget(PendingInput::GotoLine)),
        _ => None,
    }
}

fn enemies_step(app: &mut App) {
    let player_pos = app.player.position;
    let mut prior_animations = vec![None; app.enemies.len()];
    for (enemy_index, animation) in std::mem::take(&mut app.enemy_animations) {
        if enemy_index < prior_animations.len() {
            prior_animations[enemy_index] = Some(animation);
        }
    }
    let old_positions: Vec<crate::types::Position> =
        app.enemies.iter().map(|enemy| enemy.position).collect();
    let old_visual_positions: Vec<(f64, f64)> = app
        .enemies
        .iter()
        .enumerate()
        .map(|(index, enemy)| {
            prior_animations[index]
                .map(|animation| animation.current_position())
                .unwrap_or((enemy.position.x as f64, enemy.position.y as f64))
        })
        .collect();

    for enemy in &mut app.enemies {
        enemy.step_toward_player(player_pos, &app.map);
        app.audio.play(SoundEffect::EnemyStep);
    }

    let player_pos = app.player.position;
    let mut remaining_enemies = Vec::with_capacity(app.enemies.len());
    let mut next_animations = Vec::new();
    for (old_index, ((old_position, old_visual_position), enemy)) in old_positions
        .into_iter()
        .zip(old_visual_positions)
        .zip(app.enemies.drain(..))
        .enumerate()
    {
        if enemy.position == player_pos {
            app.audio.play(SoundEffect::Damage);
            if app.lives > 1 {
                app.lives -= 1;
                app.status_message = format!("Hit! {} lives remaining.", app.lives);
            } else {
                app.lives = 0;
                app.game_state = GameState::Lost;
                app.input_queue.clear();
                app.enemy_animations.clear();
                app.status_message = String::from("You were caught! Game over.");
                return;
            }
        } else {
            let new_index = remaining_enemies.len();
            if old_position != enemy.position {
                next_animations.push((
                    new_index,
                    AnimationState::new(
                        ENEMY_MOVE_MS,
                        old_visual_position,
                        (enemy.position.x as f64, enemy.position.y as f64),
                    ),
                ));
            } else if let Some(animation) = prior_animations[old_index] {
                next_animations.push((new_index, animation));
            }
            remaining_enemies.push(enemy);
        }
    }

    app.enemies = remaining_enemies;
    app.enemy_animations = next_animations;
}

fn execute_motion(app: &mut App, motion: VimMotion, target: Option<char>) {
    let old_pos = app.player.position;
    let old_zone = app.map.zone_at(old_pos);

    let activated = match motion {
        VimMotion::DeleteLine => {
            app.status_message = String::from("dd clears the nearest obstacle on your row.");
            app.player.handle_motion(motion, target, &mut app.map)
        }
        VimMotion::Find => {
            let message = target
                .map(|ch| format!("f{ch} searches forward for the next matching tile."))
                .unwrap_or_else(|| String::from("Find motion ready."));
            app.status_message = message;
            app.player.handle_motion(motion, target, &mut app.map)
        }
        VimMotion::Till => {
            let message = target
                .map(|ch| format!("t{ch} stops one tile before the next match."))
                .unwrap_or_else(|| String::from("Till motion ready."));
            app.status_message = message;
            app.player.handle_motion(motion, target, &mut app.map)
        }
        _ => {
            app.status_message = format!("{} — {}", motion.key_label(), motion.description());
            app.player.handle_motion(motion, target, &mut app.map)
        }
    };

    app.motion_count += 1;
    app.discovered_motions.insert(motion);
    app.refresh_time();

    if activated && old_pos != app.player.position {
        app.player_animation = Some(AnimationState::new(
            PLAYER_MOVE_MS,
            (old_pos.x as f64, old_pos.y as f64),
            (app.player.position.x as f64, app.player.position.y as f64),
        ));
        app.trail.push_front(old_pos);
        if app.trail.len() > crate::types::TRAIL_MAX {
            app.trail.pop_back();
        }
        app.audio.play(SoundEffect::Movement);
        let new_zone = app.map.zone_at(app.player.position);
        if new_zone != old_zone {
            app.audio.play(SoundEffect::ZoneEntry);
        }
    }

    if !activated {
        app.status_message
            .push_str(" No valid destination from here.");
    }

    if app
        .map
        .get_tile(app.player.position.x, app.player.position.y)
        == Tile::Exit
    {
        if app.level < crate::types::TOTAL_LEVELS {
            app.audio.play(SoundEffect::LevelComplete);
            app.advance_level();
        } else {
            app.audio.play(SoundEffect::Victory);
            app.game_state = GameState::Won;
            let final_time = app.start_time.elapsed();
            app.final_time = Some(final_time);
            app.elapsed = final_time;
            app.status_message = String::from("You conquered all levels of the dungeon!");
        }
        return;
    }

    if activated && old_pos != app.player.position {
        enemies_step(app);
    }
}
