use std::collections::VecDeque;
use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

use crate::map::Map;
use crate::player::Player;
use crate::types::{App, GameState, PendingInput, Tile, VimMotion};

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let map = Map::new();
        let player = Player::new(map.start);
        Self {
            map,
            player,
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
        }
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

    pub fn advance_level(&mut self) {
        self.level += 1;
        self.map = Map::level(self.level);
        self.player.position = self.map.start;
        self.trail.clear();
        self.pending_input = None;
        self.status_message = format!("Level {} — The dungeon shifts around you...", self.level);
    }
}

pub fn handle_event(app: &mut App, event: Event) {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Release {
            return;
        }
        handle_key_event(app, key);
    }
}

fn handle_key_event(app: &mut App, key: KeyEvent) {
    if !app.started {
        app.started = true;
        app.start_time = Instant::now();
        app.elapsed = Default::default();
        app.status_message =
            String::from("Use hjkl to move. Every motion is available from the start.");
        return;
    }

    if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
        app.game_state = GameState::Quit;
        return;
    }

    if app.game_state == GameState::Won {
        return;
    }

    if let Some(pending) = app.pending_input {
        app.pending_input = None;
        match pending {
            PendingInput::Find => {
                if let KeyCode::Char(target) = key.code {
                    execute_motion(app, VimMotion::Find, Some(target));
                }
            }
            PendingInput::Till => {
                if let KeyCode::Char(target) = key.code {
                    execute_motion(app, VimMotion::Till, Some(target));
                }
            }
            PendingInput::Delete => {
                if let KeyCode::Char('d') = key.code {
                    execute_motion(app, VimMotion::DeleteLine, None);
                } else {
                    app.status_message = String::from("dd needs a second d. Command cancelled.");
                }
            }
        }
        return;
    }

    match parse_motion(key) {
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
            };
        }
        None => {}
    }
}

enum ParsedInput {
    Immediate(VimMotion),
    AwaitTarget(PendingInput),
}

fn parse_motion(key: KeyEvent) -> Option<ParsedInput> {
    match key.code {
        KeyCode::Char('h') => Some(ParsedInput::Immediate(VimMotion::H)),
        KeyCode::Char('j') => Some(ParsedInput::Immediate(VimMotion::J)),
        KeyCode::Char('k') => Some(ParsedInput::Immediate(VimMotion::K)),
        KeyCode::Char('l') => Some(ParsedInput::Immediate(VimMotion::L)),
        KeyCode::Char('w') => Some(ParsedInput::Immediate(VimMotion::W)),
        KeyCode::Char('b') => Some(ParsedInput::Immediate(VimMotion::B)),
        KeyCode::Char('0') => Some(ParsedInput::Immediate(VimMotion::Zero)),
        KeyCode::Char('$') => Some(ParsedInput::Immediate(VimMotion::Dollar)),
        KeyCode::Char('f') => Some(ParsedInput::AwaitTarget(PendingInput::Find)),
        KeyCode::Char('t') => Some(ParsedInput::AwaitTarget(PendingInput::Till)),
        KeyCode::Char('d') => Some(ParsedInput::AwaitTarget(PendingInput::Delete)),
        _ => None,
    }
}

fn execute_motion(app: &mut App, motion: VimMotion, target: Option<char>) {
    let old_pos = app.player.position;

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
        app.trail.push_front(old_pos);
        if app.trail.len() > crate::types::TRAIL_MAX {
            app.trail.pop_back();
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
            app.advance_level();
        } else {
            app.game_state = GameState::Won;
            let final_time = app.start_time.elapsed();
            app.final_time = Some(final_time);
            app.elapsed = final_time;
            app.status_message = String::from("You conquered all levels of the dungeon!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Position, Zone};
    use crossterm::event::KeyModifiers;
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
        }
    }

    fn started_app_with_map(map: Map, position: Position) -> App {
        App {
            map,
            player: Player::new(position),
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
        }
    }

    fn key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
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
    fn app_handle_key_starts_game() {
        let mut app = App::new();

        handle_event(&mut app, key_event(KeyCode::Char('h')));

        assert!(app.started);
    }

    #[test]
    fn app_esc_quits() {
        let mut app = App::new();
        app.started = true;

        handle_event(&mut app, key_event(KeyCode::Esc));

        assert_eq!(app.game_state, GameState::Quit);
    }

    #[test]
    fn app_q_quits() {
        let mut app = App::new();
        app.started = true;

        handle_event(&mut app, key_event(KeyCode::Char('q')));

        assert_eq!(app.game_state, GameState::Quit);
    }

    #[test]
    fn app_h_motion_moves_player() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });

        handle_event(&mut app, key_event(KeyCode::Char('h')));

        assert_eq!(app.player.position, Position { x: 1, y: 0 });
    }

    #[test]
    fn app_trail_records_successful_motion() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });

        handle_event(&mut app, key_event(KeyCode::Char('l')));

        assert_eq!(app.trail.len(), 1);
        assert_eq!(app.trail[0], Position { x: 2, y: 0 });
    }

    #[test]
    fn app_trail_does_not_record_failed_motion() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 0, y: 0 });

        handle_event(&mut app, key_event(KeyCode::Char('h')));

        assert!(app.trail.is_empty());
    }

    #[test]
    fn app_trail_caps_at_max() {
        let mut app = started_app_with_map(test_map(20, 1), Position { x: 1, y: 0 });

        for _ in 0..(crate::types::TRAIL_MAX + 2) {
            handle_event(&mut app, key_event(KeyCode::Char('l')));
            handle_event(&mut app, key_event(KeyCode::Char('h')));
        }

        assert!(app.trail.len() <= crate::types::TRAIL_MAX);
    }

    #[test]
    fn app_d_then_d_deletes_obstacle() {
        let mut map = test_map(6, 1);
        map.set_tile(3, 0, Tile::Obstacle);
        let mut app = started_app_with_map(map, Position { x: 1, y: 0 });

        handle_event(&mut app, key_event(KeyCode::Char('d')));
        handle_event(&mut app, key_event(KeyCode::Char('d')));

        assert_eq!(app.map.get_tile(3, 0), Tile::Floor);
        assert_eq!(app.pending_input, None);
    }

    #[test]
    fn app_d_then_other_cancels() {
        let mut app = started_app_with_map(test_map(6, 1), Position { x: 1, y: 0 });

        handle_event(&mut app, key_event(KeyCode::Char('d')));
        handle_event(&mut app, key_event(KeyCode::Char('x')));

        assert_eq!(app.pending_input, None);
        assert!(app.status_message.contains("cancelled"));
    }

    #[test]
    fn app_f_then_char_finds() {
        let mut map = test_map(6, 1);
        map.set_tile(4, 0, Tile::Exit);
        let mut app = started_app_with_map(map, Position { x: 1, y: 0 });
        app.level = crate::types::TOTAL_LEVELS;

        handle_event(&mut app, key_event(KeyCode::Char('f')));
        handle_event(&mut app, key_event(KeyCode::Char('>')));

        assert_eq!(app.player.position, Position { x: 4, y: 0 });
    }

    #[test]
    fn app_win_condition_on_exit_tile() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = crate::types::TOTAL_LEVELS;

        handle_event(&mut app, key_event(KeyCode::Char('l')));

        assert_eq!(app.game_state, GameState::Won);
    }

    #[test]
    fn app_motion_count_increments() {
        let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });

        handle_event(&mut app, key_event(KeyCode::Char('l')));

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

        handle_event(&mut app, key_event(KeyCode::Char('l')));

        assert_eq!(app.level, 2);
        assert_eq!(app.game_state, GameState::Playing);
    }

    #[test]
    fn app_exit_on_level_2_triggers_win() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 2;

        handle_event(&mut app, key_event(KeyCode::Char('l')));

        assert_eq!(app.level, 2);
        assert_eq!(app.game_state, GameState::Won);
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

        handle_event(&mut app, key_event(KeyCode::Char('l')));

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

        handle_event(&mut app, key_event(KeyCode::Char('l')));

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

        handle_event(&mut app, key_event(KeyCode::Char('l')));

        assert_eq!(app.player.position, app.map.start);
    }

    #[test]
    fn app_level_transition_loads_new_map() {
        let mut map = test_map(5, 1);
        map.set_tile(4, 0, Tile::Exit);
        map.exit = Position { x: 4, y: 0 };
        let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
        app.level = 1;

        handle_event(&mut app, key_event(KeyCode::Char('l')));

        assert_eq!(app.level, 2);
        assert_eq!(app.player.position, app.map.start);
    }
}
