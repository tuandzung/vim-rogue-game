use std::collections::VecDeque;
use std::time::Instant;

use bracket_lib::prelude::VirtualKeyCode;

use crate::animation::{AnimationState, AttackEffect, AttackEffectKind, ENEMY_MOVE_MS, PLAYER_MOVE_MS};
use crate::audio::SoundEffect;
use crate::map::Map;
use crate::player::Player;
use crate::types::{App, Enemy, FOV_RADIUS, GameState, MAX_HP, PauseOption, PendingInput, PatrolArea, Tile, TORCHLIGHT_FOV_RADIUS, VimMotion};
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
            attack_effects: Vec::new(),
            pending_respawn: None,
            input_queue: Vec::new(),
            enemies: Vec::new(),
            hp: MAX_HP,
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
            last_checkpoint: None,
            activated_torchlights: Default::default(),
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
                    Tile::Floor | Tile::Exit | Tile::Obstacle | Tile::Torchlight
                )
            });

        let torchlight_sources: Vec<(crate::types::Position, i32)> = self
            .activated_torchlights
            .iter()
            .map(|&pos| (pos, TORCHLIGHT_FOV_RADIUS))
            .collect();

        if !torchlight_sources.is_empty() {
            let map_ref = &self.map;
            self.visibility.compute_multi_fov(&torchlight_sources, |pos| {
                matches!(
                    map_ref.get_tile(pos.x, pos.y),
                    Tile::Floor | Tile::Exit | Tile::Obstacle | Tile::Torchlight
                )
            });
        }
    }

    fn spawn_enemies_for_current_level(&mut self) {
        self.enemies = self
            .map
            .enemy_spawns
            .iter()
            .enumerate()
            .map(|(i, &pos)| {
                let patrol_area = self.map.enemy_patrol_areas.get(i).copied()
                    .unwrap_or_else(|| PatrolArea::point(pos.x, pos.y));
                if self.level == 4 {
                    Enemy { position: pos, glyph: 'e', hp: Some(30), stunned_turns: 0, patrol_area }
                } else {
                    let mut e = Enemy::new(pos);
                    e.patrol_area = patrol_area;
                    e
                }
            })
            .collect();
    }

    pub fn advance_level(&mut self) {
        self.level += 1;
        self.map = Map::level(self.level);
        self.player.position = self.map.start;
        self.player_animation = None;
        self.enemy_animations.clear();
        self.attack_effects.clear();
        self.pending_respawn = None;
        self.input_queue.clear();
        self.spawn_enemies_for_current_level();
        self.trail.clear();
        self.pending_input = None;
        self.last_checkpoint = None;
        self.activated_torchlights.clear();
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
        self.attack_effects.clear();
        self.pending_respawn = None;
        self.input_queue.clear();
        self.spawn_enemies_for_current_level();
        self.hp = MAX_HP;
        self.trail.clear();
        self.pending_input = None;
        self.game_state = GameState::Playing;
        self.last_checkpoint = None;
        self.activated_torchlights.clear();
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

    if app.game_state == GameState::Dying {
        app.attack_effects.retain(|e| !e.is_complete());
        if app.attack_effects.is_empty() {
            let checkpoint = app.pending_respawn.take();
            match checkpoint {
                Some(pos) => {
                    app.enemy_animations.clear();
                    app.hp = MAX_HP;
                    app.player.position = pos;
                    app.player_animation = None;
                    app.game_state = GameState::Playing;
                    push_enemies_off_position(app, pos);
                    app.update_visibility();
                    app.status_message = String::from("Respawned at checkpoint!");
                }
                None => {
                    app.game_state = GameState::Lost;
                    app.status_message = String::from("You were caught! Game over.");
                }
            }
            app.attack_effects.clear();
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

        for effect in &mut app.attack_effects {
            effect.update(delta_ms);
        }
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

    app.attack_effects.retain(|e| !e.is_complete());
    for effect in &mut app.attack_effects {
        effect.update(delta_ms);
    }

    while app.player_animation.is_none() && !app.input_queue.is_empty() {
        let (key, shift) = app.input_queue.remove(0);
        handle_key(app, key, shift);

        if matches!(app.game_state, GameState::Quit | GameState::Won | GameState::Dying) {
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

    if app.game_state == GameState::Dying {
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

    if key == VirtualKeyCode::X && !shift {
        handle_melee_attack(app);
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

fn push_enemies_off_position(app: &mut App, pos: crate::types::Position) {
    use std::collections::{HashSet, VecDeque};
    let directions: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut new_positions: Vec<Option<crate::types::Position>> = vec![None; app.enemies.len()];
    let mut claimed: HashSet<crate::types::Position> = HashSet::new();

    let enemies_on_pos: Vec<usize> = app
        .enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| e.position == pos)
        .map(|(i, _)| i)
        .collect();

    if enemies_on_pos.is_empty() {
        return;
    }

    let mut visited: HashSet<crate::types::Position> = HashSet::new();
    visited.insert(pos);
    let mut bfs: VecDeque<crate::types::Position> = VecDeque::new();
    for (dx, dy) in &directions {
        let nx = (pos.x as isize + dx) as usize;
        let ny = (pos.y as isize + dy) as usize;
        if nx < app.map.width && ny < app.map.height
            && app.map.is_passable(nx, ny)
        {
            let neighbor = crate::types::Position { x: nx, y: ny };
            if visited.insert(neighbor) {
                bfs.push_back(neighbor);
            }
        }
    }

    let mut assigned = 0;
    while assigned < enemies_on_pos.len() {
        let candidate = match bfs.pop_front() {
            Some(c) => c,
            None => break,
        };

        if candidate.x < app.map.width
            && candidate.y < app.map.height
            && app.map.is_passable(candidate.x, candidate.y)
            && !claimed.contains(&candidate)
            && !app.enemies.iter().any(|e| e.position == candidate)
        {
            let idx = enemies_on_pos[assigned];
            new_positions[idx] = Some(candidate);
            claimed.insert(candidate);
            assigned += 1;
        }

        for (dx, dy) in &directions {
            let nx = (candidate.x as isize + dx) as usize;
            let ny = (candidate.y as isize + dy) as usize;
            if nx < app.map.width && ny < app.map.height
                && app.map.is_passable(nx, ny)
            {
                let neighbor = crate::types::Position { x: nx, y: ny };
                if visited.insert(neighbor) {
                    bfs.push_back(neighbor);
                }
            }
        }
    }

    for (i, enemy) in app.enemies.iter_mut().enumerate() {
        if let Some(new_pos) = new_positions[i] {
            enemy.position = new_pos;
        }
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
        if enemy.stunned_turns > 0 {
            enemy.stunned_turns -= 1;
        } else {
            let moved = if enemy.has_line_of_sight(player_pos, &app.map) {
                enemy.step_toward_player(player_pos, &app.map)
            } else {
                enemy.patrol_step(&app.map)
            };
            if moved {
                app.audio.play(SoundEffect::EnemyStep);
            }
        }
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
        if enemy.position == player_pos && enemy.stunned_turns == 0 && app.game_state == GameState::Playing {
            app.audio.play(SoundEffect::Damage);
            app.hp -= 10;
            app.attack_effects.push(AttackEffect::new(
                AttackEffectKind::EnemyHit,
                player_pos.x,
                player_pos.y,
            ));
            if app.hp <= 0 {
                app.hp = 0;
                app.input_queue.clear();
                app.game_state = GameState::Dying;
                app.pending_respawn = app.last_checkpoint;
            } else {
                app.status_message = format!("Hit! {} HP remaining.", app.hp);
            }
            if enemy.hp.is_some() {
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
        == Tile::Torchlight
    {
        let torch_pos = app.player.position;
        if !app.activated_torchlights.contains(&torch_pos) {
            app.activated_torchlights.insert(torch_pos);
            app.last_checkpoint = Some(torch_pos);
            app.status_message = String::from("Checkpoint activated! Torchlight lit.");
        }
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
        app.update_visibility();
        enemies_step(app);
    }
}

fn handle_melee_attack(app: &mut App) {
    let facing = match app.player.last_direction {
        Some(dir) => dir,
        None => {
            app.status_message = String::from("No direction — move first.");
            return;
        }
    };

    let (dx, dy) = facing.delta();
    let target_x = (app.player.position.x as isize + dx) as usize;
    let target_y = (app.player.position.y as isize + dy) as usize;

    let enemy_index = app
        .enemies
        .iter()
        .position(|e| e.position.x == target_x && e.position.y == target_y);

    match enemy_index {
        Some(idx) => {
            let enemy_hp = app.enemies[idx].hp;
            match enemy_hp {
                Some(hp) if hp > 0 => {
                    app.attack_effects.push(AttackEffect::new(
                        AttackEffectKind::PlayerStrike,
                        target_x,
                        target_y,
                    ));
                    let new_hp = hp - 10;
                    if new_hp <= 0 {
                        app.enemies.remove(idx);
                        app.status_message = String::from("Enemy defeated!");
                    } else {
                        app.enemies[idx].hp = Some(new_hp);
                        app.enemies[idx].stunned_turns = 1;
                        app.status_message = format!("Hit! Enemy HP: {}", new_hp);
                    }
                    app.motion_count += 1;
                    app.refresh_time();
                    enemies_step(app);
                }
                _ => {
                    app.status_message = String::from("Can't attack this enemy.");
                }
            }
        }
        None => {
            app.status_message = String::from("Nothing there.");
            app.motion_count += 1;
            app.refresh_time();
            enemies_step(app);
        }
    }
}
