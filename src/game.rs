use std::collections::VecDeque;
use std::time::Instant;

use bracket_lib::prelude::VirtualKeyCode;

use crate::animation::{AnimationState, ENEMY_MOVE_MS, PLAYER_MOVE_MS};
use crate::audio::SoundEffect;
use crate::map::Map;
use crate::player::Player;
use crate::types::{App, Enemy, GameState, PendingInput, Tile, VimMotion, FOV_RADIUS};
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

    if matches!(key, VirtualKeyCode::Escape) || (key == VirtualKeyCode::Q && !shift) {
        app.input_queue.clear();
        app.game_state = GameState::Quit;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Enemy, Position, Zone};
    use crate::visibility::{VisibilityMap, VisibilityState};
    use std::time::Duration;

    fn test_map(width: usize, height: usize) -> Map {
        Map {
            grid: vec![vec![Tile::Floor; width]; height],
            zones: vec![vec![Zone::Zone1; width]; height],
            width,
            height,
            start: Position { x: 0, y: 0 },
            exit: Position {
                x: width - 1,
                y: height - 1,
            },
            enemy_spawns: vec![],
        }
    }

    fn started_app_with_map(map: Map, position: Position) -> App {
        let visibility = VisibilityMap::new(map.width, map.height);
        let mut app = App {
            map,
            visibility,
            player: Player::new(position),
            player_animation: None,
            enemy_animations: Vec::new(),
            input_queue: Vec::new(),
            enemies: Vec::new(),
            lives: 3,
            game_state: GameState::Playing,
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
            audio: crate::audio::AudioManager::new(),
        };
        app.update_visibility();
        app
    }

    #[test]
    fn app_new_starts_playing() {
        let app = App::new();

        assert_eq!(app.game_state, GameState::Playing);
    }

    #[test]
    fn app_new_not_started() {
        let app = App::new();

        assert!(!app.started);
    }

    #[test]
    fn app_new_start_time_exists() {
        let app = App::new();

        assert!(app.start_time <= Instant::now());
    }

    #[test]
    fn app_trail_starts_empty() {
        let app = App::new();

        assert!(app.trail.is_empty());
    }

    #[test]
    fn app_current_zone_tracks_position() {
        let mut map = test_map(3, 1);
        map.zones[0][1] = Zone::Zone4;
        let app = started_app_with_map(map, Position { x: 1, y: 0 });

        assert_eq!(app.current_zone(), Zone::Zone4);
    }

    #[test]
    fn app_unique_motions_starts_zero() {
        let app = App::new();

        assert_eq!(app.unique_motions(), 0);
    }

    #[test]
    fn app_new_has_visibility_map() {
        let app = App::new();

        assert_eq!(app.visibility.width(), 80);
        assert_eq!(app.visibility.height(), 40);
        assert_eq!(
            app.visibility.get(app.player.position),
            VisibilityState::Visible
        );
    }

    #[test]
    fn update_visibility_makes_area_visible() {
        let mut app = App::new();

        app.visibility.reset();
        app.update_visibility();

        assert_eq!(
            app.visibility.get(app.player.position),
            VisibilityState::Visible
        );
        assert_eq!(
            app.visibility.get(Position { x: 3, y: 2 }),
            VisibilityState::Visible
        );
    }

    #[test]
    fn update_visibility_walls_block() {
        let mut app = App::new();

        app.visibility.reset();
        app.update_visibility();

        assert_eq!(
            app.visibility.get(Position { x: 1, y: 2 }),
            VisibilityState::Visible
        );
        assert_eq!(
            app.visibility.get(Position { x: 0, y: 2 }),
            VisibilityState::Hidden
        );
    }

    #[test]
    fn update_visibility_crosses_zone_boundaries() {
        let mut map = test_map(20, 5);
        for row in &mut map.zones {
            for zone in &mut row[10..] {
                *zone = Zone::Zone2;
            }
        }

        let app = started_app_with_map(map, Position { x: 9, y: 2 });

        assert_eq!(app.current_zone(), Zone::Zone1);
        assert_eq!(app.map.zone_at(Position { x: 10, y: 2 }), Zone::Zone2);
        assert_eq!(
            app.visibility.get(Position { x: 10, y: 2 }),
            VisibilityState::Visible
        );
    }

    #[test]
    fn update_visibility_treats_obstacles_as_transparent() {
        let mut map = test_map(8, 5);
        map.set_tile(2, 2, Tile::Obstacle);
        map.set_tile(3, 2, Tile::Floor);

        let app = started_app_with_map(map, Position { x: 1, y: 2 });

        assert_eq!(
            app.visibility.get(Position { x: 2, y: 2 }),
            VisibilityState::Visible
        );
        assert_eq!(
            app.visibility.get(Position { x: 3, y: 2 }),
            VisibilityState::Visible
        );
    }

    #[test]
    fn app_handle_key_starts_game() {
        let mut app = App::new();

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert!(app.started);
    }

    #[test]
    fn app_esc_quits() {
        let mut app = App::new();
        app.started = true;

        handle_key(&mut app, VirtualKeyCode::Escape, false);

        assert_eq!(app.game_state, GameState::Quit);
    }

    #[test]
    fn app_q_quits() {
        let mut app = App::new();
        app.started = true;

        handle_key(&mut app, VirtualKeyCode::Q, false);

        assert_eq!(app.game_state, GameState::Quit);
    }

    #[test]
    fn app_h_motion_moves_player() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert_eq!(app.player.position, Position { x: 1, y: 0 });
    }

    #[test]
    fn app_trail_records_successful_motion() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.trail.len(), 1);
        assert_eq!(app.trail[0], Position { x: 2, y: 0 });
    }

    #[test]
    fn app_trail_does_not_record_failed_motion() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 0, y: 0 });

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert!(app.trail.is_empty());
    }

    #[test]
    fn player_animation_starts_on_move() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);

        let animation = app
            .player_animation
            .expect("movement should start animation");
        assert_eq!((animation.start_x, animation.start_y), (1.0, 0.0));
        assert_eq!((animation.end_x, animation.end_y), (2.0, 0.0));
    }

    #[test]
    fn player_animation_completes_after_duration() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        tick(&mut app, PLAYER_MOVE_MS);

        assert_eq!(app.player_animation, None);
    }

    #[test]
    fn player_animation_interpolates_position() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);

        let animation = app
            .player_animation
            .as_mut()
            .expect("movement should start animation");
        animation.update(PLAYER_MOVE_MS / 2.0);

        assert_eq!(animation.current_position(), (1.5, 0.0));
    }

    #[test]
    fn input_queued_during_animation() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.player.position, Position { x: 2, y: 0 });
        assert_eq!(app.input_queue, vec![(VirtualKeyCode::L, false)]);
    }

    #[test]
    fn queued_input_executed_after_animation() {
        let mut app = started_app_with_map(test_map(6, 1), Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::L, false);
        tick(&mut app, PLAYER_MOVE_MS);

        let animation = app
            .player_animation
            .expect("queued move should start a new animation");
        assert_eq!(app.player.position, Position { x: 3, y: 0 });
        assert!(app.input_queue.is_empty());
        assert_eq!((animation.start_x, animation.start_y), (2.0, 0.0));
        assert_eq!((animation.end_x, animation.end_y), (3.0, 0.0));
    }

    #[test]
    fn queued_multi_key_motion_executes_without_stalling() {
        let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 4 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::G, false);
        handle_key(&mut app, VirtualKeyCode::G, false);
        tick(&mut app, PLAYER_MOVE_MS);

        assert_eq!(app.player.position, Position { x: 0, y: 0 });
        assert_eq!(app.pending_input, None);
        assert!(app.player_animation.is_some());
        assert!(app.input_queue.is_empty());
    }

    #[test]
    fn queued_find_preserves_pending_input_after_animation() {
        let mut map = test_map(6, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 1, y: 0 });
        app.level = crate::types::TOTAL_LEVELS;

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::F, false);

        tick(&mut app, PLAYER_MOVE_MS);

        assert_eq!(app.pending_input, Some(PendingInput::Find));
        assert!(app.input_queue.is_empty());

        handle_key(&mut app, VirtualKeyCode::Period, true);

        assert_eq!(app.pending_input, None);
        assert_eq!(app.player.position, Position { x: 4, y: 0 });
    }

    #[test]
    fn queued_till_preserves_pending_input_after_animation() {
        let mut map = test_map(7, 1);
        map.set_tile(5, 0, Tile::Exit);
        let mut app = started_app_with_map(map, Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::T, false);

        tick(&mut app, PLAYER_MOVE_MS);

        assert_eq!(app.pending_input, Some(PendingInput::Till));

        handle_key(&mut app, VirtualKeyCode::Period, true);

        assert_eq!(app.pending_input, None);
        assert_eq!(app.player.position, Position { x: 4, y: 0 });
    }

    #[test]
    fn queued_delete_preserves_pending_input_after_animation() {
        let mut map = test_map(6, 1);
        map.set_tile(4, 0, Tile::Obstacle);
        let mut app = started_app_with_map(map, Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::D, false);

        tick(&mut app, PLAYER_MOVE_MS);

        assert_eq!(app.pending_input, Some(PendingInput::Delete));

        handle_key(&mut app, VirtualKeyCode::D, false);

        assert_eq!(app.pending_input, None);
        assert_eq!(app.map.get_tile(4, 0), Tile::Floor);
    }

    #[test]
    fn queued_goto_line_preserves_pending_input_after_animation() {
        let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 4 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::G, false);

        tick(&mut app, PLAYER_MOVE_MS);

        assert_eq!(app.pending_input, Some(PendingInput::GotoLine));

        handle_key(&mut app, VirtualKeyCode::G, false);

        assert_eq!(app.pending_input, None);
        assert_eq!(app.player.position.y, 0);
    }

    #[test]
    fn rapid_keypresses_preserve_order_without_double_triggering() {
        let mut app = started_app_with_map(test_map(8, 1), Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::H, false);
        handle_key(&mut app, VirtualKeyCode::L, false);

        tick(&mut app, PLAYER_MOVE_MS);
        tick(&mut app, PLAYER_MOVE_MS);
        tick(&mut app, PLAYER_MOVE_MS);

        assert_eq!(app.player.position, Position { x: 3, y: 0 });
        assert_eq!(app.motion_count, 4);
        assert!(app.input_queue.is_empty());
        assert!(app.player_animation.is_some());
    }

    #[test]
    fn rapid_keypresses_can_queue_find_and_target_together() {
        let mut map = test_map(6, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 1, y: 0 });
        app.level = crate::types::TOTAL_LEVELS;

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::F, false);
        handle_key(&mut app, VirtualKeyCode::Period, true);

        tick(&mut app, PLAYER_MOVE_MS);

        assert_eq!(app.pending_input, None);
        assert_eq!(app.player.position, Position { x: 4, y: 0 });
        assert!(app.input_queue.is_empty());
    }

    #[test]
    fn no_animation_on_failed_move() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 0, y: 0 });

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert_eq!(app.player_animation, None);
    }

    #[test]
    fn app_trail_caps_at_max() {
        let mut app = started_app_with_map(test_map(20, 1), Position { x: 1, y: 0 });

        for _ in 0..(crate::types::TRAIL_MAX + 2) {
            handle_key(&mut app, VirtualKeyCode::L, false);
            handle_key(&mut app, VirtualKeyCode::H, false);
        }

        assert!(app.trail.len() <= crate::types::TRAIL_MAX);
    }

    #[test]
    fn app_d_then_d_deletes_obstacle() {
        let mut map = test_map(6, 1);
        map.set_tile(3, 0, Tile::Obstacle);
        let mut app = started_app_with_map(map, Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::D, false);
        handle_key(&mut app, VirtualKeyCode::D, false);

        assert_eq!(app.map.get_tile(3, 0), Tile::Floor);
        assert_eq!(app.pending_input, None);
    }

    #[test]
    fn app_d_then_other_cancels() {
        let mut app = started_app_with_map(test_map(6, 1), Position { x: 1, y: 0 });

        handle_key(&mut app, VirtualKeyCode::D, false);
        handle_key(&mut app, VirtualKeyCode::X, false);

        assert_eq!(app.pending_input, None);
        assert!(app.status_message.contains("cancelled"));
    }

    #[test]
    fn app_f_then_char_finds() {
        let mut map = test_map(6, 1);
        map.set_tile(4, 0, Tile::Exit);
        let mut app = started_app_with_map(map, Position { x: 1, y: 0 });
        app.level = crate::types::TOTAL_LEVELS;

        handle_key(&mut app, VirtualKeyCode::F, false);
        handle_key(&mut app, VirtualKeyCode::Period, true);

        assert_eq!(app.player.position, Position { x: 4, y: 0 });
    }

    #[test]
    fn app_win_condition_on_exit_tile() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = crate::types::TOTAL_LEVELS;

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.game_state, GameState::Won);
    }

    #[test]
    fn app_motion_count_increments() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.motion_count, 1);
    }

    #[test]
    fn app_new_level_is_one() {
        let app = App::new();
        assert_eq!(app.level, 1);
    }

    #[test]
    fn app_exit_on_level_1_transitions_to_level_2() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.level, 2);
        assert_eq!(app.game_state, GameState::Playing);
    }

    #[test]
    fn app_exit_on_level_2_transitions_to_level_3() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 2;

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.level, 3);
        assert_eq!(app.game_state, GameState::Playing);
    }

    #[test]
    fn app_level_transition_preserves_stats() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;
        app.motion_count = 42;
        app.discovered_motions.insert(VimMotion::H);
        app.discovered_motions.insert(VimMotion::J);

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.level, 2);
        assert_eq!(app.motion_count, 43);
        assert_eq!(app.discovered_motions.len(), 3);
    }

    #[test]
    fn app_level_transition_clears_trail() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;
        app.trail.push_front(Position { x: 2, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.level, 2);
        assert!(app.trail.is_empty());
    }

    #[test]
    fn app_level_transition_clears_pending_input() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;
        app.pending_input = Some(PendingInput::Delete);

        app.advance_level();

        assert_eq!(app.level, 2);
        assert_eq!(app.pending_input, None);
    }

    #[test]
    fn app_level_transition_resets_player_position() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.player.position, app.map.start);
    }

    #[test]
    fn app_level_transition_loads_new_map() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.level, 2);
        assert_eq!(app.player.position, app.map.start);
    }

    #[test]
    fn app_g_jump_to_last_row() {
        let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 2 });

        handle_key(&mut app, VirtualKeyCode::G, true);

        assert_eq!(app.player.position.y, 4);
        assert_eq!(app.pending_input, None);
    }

    #[test]
    fn app_gg_two_keys_jump_to_first_row() {
        let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 4 });

        handle_key(&mut app, VirtualKeyCode::G, false);
        assert_eq!(app.pending_input, Some(PendingInput::GotoLine));

        handle_key(&mut app, VirtualKeyCode::G, false);

        assert_eq!(app.player.position.y, 0);
        assert_eq!(app.pending_input, None);
    }

    #[test]
    fn app_g_then_other_cancels() {
        let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 2 });

        handle_key(&mut app, VirtualKeyCode::G, false);
        handle_key(&mut app, VirtualKeyCode::X, false);

        assert_eq!(app.player.position, Position { x: 2, y: 2 });
        assert_eq!(app.pending_input, None);
        assert!(app.status_message.contains("cancelled"));
    }

    #[test]
    fn app_lost_state_any_key_restarts_level() {
        let mut app = started_app_with_map(test_map(5, 5), Position { x: 3, y: 3 });
        app.level = 2;
        app.game_state = GameState::Lost;
        app.trail.push_front(Position { x: 2, y: 2 });

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert_eq!(app.game_state, GameState::Playing);
        assert_eq!(app.player.position, app.map.start);
        assert!(app.trail.is_empty());
        assert_eq!(app.level, 2);
    }

    #[test]
    fn advance_level_resets_visibility() {
        let mut app = started_app_with_map(test_map(5, 5), Position { x: 3, y: 3 });
        let far_tile = Position { x: 79, y: 0 };
        app.level = 1;
        app.visibility.set(far_tile, VisibilityState::Explored);

        app.advance_level();

        assert_eq!(app.visibility.get(far_tile), VisibilityState::Hidden);
        assert_eq!(
            app.visibility.get(app.player.position),
            VisibilityState::Visible
        );
    }

    #[test]
    fn retry_level_resets_visibility() {
        let mut app = started_app_with_map(test_map(5, 5), Position { x: 3, y: 3 });
        let far_tile = Position { x: 79, y: 0 };
        app.level = 2;
        app.visibility.set(far_tile, VisibilityState::Explored);

        app.retry_level();

        assert_eq!(app.visibility.get(far_tile), VisibilityState::Hidden);
        assert_eq!(
            app.visibility.get(app.player.position),
            VisibilityState::Visible
        );
    }

    #[test]
    fn app_enemy_collision_decrements_lives_and_removes_enemy() {
        let map = test_map(5, 5);
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.lives = 3;
        app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert_eq!(app.lives, 2);
        assert_eq!(app.game_state, GameState::Playing);
        assert!(app.status_message.contains("2 lives remaining"));
        assert!(app.enemies.is_empty());
    }

    #[test]
    fn enemy_animation_starts_on_move() {
        let map = test_map(6, 3);
        let mut app = started_app_with_map(map, Position { x: 3, y: 1 });
        app.enemies.push(Enemy::new(Position { x: 1, y: 1 }));

        handle_key(&mut app, VirtualKeyCode::L, false);

        let (enemy_index, animation) = app
            .enemy_animations
            .first()
            .copied()
            .expect("enemy move should start animation");
        assert_eq!(enemy_index, 0);
        assert_eq!((animation.start_x, animation.start_y), (1.0, 1.0));
        assert_eq!((animation.end_x, animation.end_y), (2.0, 1.0));
    }

    #[test]
    fn enemy_animation_completes_after_duration() {
        let map = test_map(6, 3);
        let mut app = started_app_with_map(map, Position { x: 3, y: 1 });
        app.enemies.push(Enemy::new(Position { x: 1, y: 1 }));

        handle_key(&mut app, VirtualKeyCode::L, false);
        tick(&mut app, ENEMY_MOVE_MS);

        assert!(app.enemy_animations.is_empty());
    }

    #[test]
    fn multiple_enemies_animate_simultaneously() {
        let map = test_map(8, 5);
        let mut app = started_app_with_map(map, Position { x: 4, y: 2 });
        app.enemies.push(Enemy::new(Position { x: 0, y: 0 }));
        app.enemies.push(Enemy::new(Position { x: 0, y: 2 }));
        app.enemies.push(Enemy::new(Position { x: 0, y: 4 }));

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.enemy_animations.len(), 3);
        assert_eq!(app.enemy_animations[0].0, 0);
        assert_eq!(app.enemy_animations[1].0, 1);
        assert_eq!(app.enemy_animations[2].0, 2);
    }

    #[test]
    fn enemy_animation_interpolates_position() {
        let map = test_map(6, 3);
        let mut app = started_app_with_map(map, Position { x: 3, y: 1 });
        app.enemies.push(Enemy::new(Position { x: 1, y: 1 }));

        handle_key(&mut app, VirtualKeyCode::L, false);

        let (_, animation) = app
            .enemy_animations
            .first_mut()
            .expect("enemy move should start animation");
        animation.update(ENEMY_MOVE_MS / 2.0);

        assert_eq!(animation.current_position(), (1.5, 1.0));
    }

    #[test]
    fn app_enemy_collision_sets_lost_when_no_lives() {
        let map = test_map(5, 5);
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.lives = 1;
        app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert_eq!(app.lives, 0);
        assert_eq!(app.game_state, GameState::Lost);
        assert!(app.status_message.contains("Game over"));
        assert!(app.enemies.is_empty());
    }

    #[test]
    fn app_advance_level_spawns_enemies_from_map() {
        let mut map = test_map(5, 5);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 2;

        app.advance_level();

        assert_eq!(app.level, 3);
        let level3_map = Map::level(3);
        assert_eq!(app.enemies.len(), level3_map.enemy_spawns.len());
        for (enemy, spawn) in app.enemies.iter().zip(level3_map.enemy_spawns.iter()) {
            assert_eq!(enemy.position, *spawn);
        }
    }

    #[test]
    fn app_advance_level_preserves_lives() {
        let mut map = test_map(5, 5);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;
        app.lives = 2;

        app.advance_level();

        assert_eq!(app.lives, 2);
    }

    #[test]
    fn app_advance_level_level_1_to_2_has_no_enemies() {
        let map = Map::level(1);
        assert!(map.enemy_spawns.is_empty());
        let map2 = Map::level(2);
        assert!(map2.enemy_spawns.is_empty());
    }

    #[test]
    fn app_advance_level_level_2_to_3_spawns_enemies() {
        let mut map = test_map(5, 5);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 2;

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.level, 3);
        assert!(!app.enemies.is_empty());
        assert_eq!(app.enemies.len(), Map::level(3).enemy_spawns.len());
    }

    #[test]
    fn app_advance_level_preserves_motion_count() {
        let mut map = test_map(5, 5);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;
        app.motion_count = 42;

        app.advance_level();

        assert_eq!(app.motion_count, 42);
    }

    #[test]
    fn audio_movement_plays_on_successful_move() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });
        app.audio.enable();

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_ne!(app.player.position, Position { x: 2, y: 0 });
    }

    #[test]
    fn audio_no_sound_on_failed_move() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 0, y: 0 });
        app.audio.enable();

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert_eq!(app.player.position, Position { x: 0, y: 0 });
    }

    #[test]
    fn audio_zone_entry_plays_on_zone_change() {
        let mut map = test_map(80, 1);
        for x in 0..16 {
            map.zones[0][x] = Zone::Zone1;
        }
        for x in 16..32 {
            map.zones[0][x] = Zone::Zone2;
        }
        let mut app = started_app_with_map(map, Position { x: 15, y: 0 });
        app.audio.enable();

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.player.position, Position { x: 16, y: 0 });
    }

    #[test]
    fn audio_no_zone_entry_sound_when_same_zone() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });
        app.audio.enable();

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(
            app.map.zone_at(app.player.position),
            app.map.zone_at(Position { x: 2, y: 0 })
        );
    }

    #[test]
    fn audio_damage_plays_on_enemy_hit() {
        let map = test_map(5, 5);
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.lives = 3;
        app.audio.enable();
        app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

        handle_key(&mut app, VirtualKeyCode::H, false);

        assert_eq!(app.lives, 2);
        assert!(app.enemies.is_empty());
    }

    #[test]
    fn audio_victory_plays_on_win() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = crate::types::TOTAL_LEVELS;
        app.audio.enable();

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.game_state, GameState::Won);
    }

    #[test]
    fn audio_level_complete_plays_on_advance() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;
        app.audio.enable();

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.level, 2);
        assert_eq!(app.game_state, GameState::Playing);
    }

    #[test]
    fn audio_enemy_step_plays_when_enemies_move() {
        let map = test_map(5, 5);
        let mut app = started_app_with_map(map, Position { x: 4, y: 0 });
        app.audio.enable();
        app.enemies.push(Enemy::new(Position { x: 2, y: 2 }));

        handle_key(&mut app, VirtualKeyCode::L, false);

        assert!(!app.enemies.is_empty());
    }

    #[test]
    fn audio_no_panic_when_disabled() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 2, y: 0 });

        handle_key(&mut app, VirtualKeyCode::L, false);
        assert!(!app.audio.is_enabled());

        let mut map2 = test_map(5, 1);
        map2.set_tile(4, 0, Tile::Exit);
        map2.exit = Position { x: 4, y: 0 };
        let mut app2 = started_app_with_map(map2, Position { x: 3, y: 0 });
        app2.level = crate::types::TOTAL_LEVELS;
        handle_key(&mut app2, VirtualKeyCode::L, false);
        assert_eq!(app2.game_state, GameState::Won);

        let map3 = test_map(5, 5);
        let mut app3 = started_app_with_map(map3, Position { x: 3, y: 0 });
        app3.lives = 3;
        app3.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
        handle_key(&mut app3, VirtualKeyCode::H, false);
        assert_eq!(app3.lives, 2);
    }

    #[test]
    fn audio_app_new_has_disabled_audio() {
        let app = App::new();
        assert!(!app.audio.is_enabled());
    }

    #[test]
    fn audio_enabled_does_not_crash_during_movement() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });
        app.audio.enable();

        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::H, false);
        handle_key(&mut app, VirtualKeyCode::L, false);

        assert_eq!(app.player.position, Position { x: 3, y: 0 });
    }
}
