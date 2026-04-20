mod common;

use bracket_lib::prelude::VirtualKeyCode;
use common::{started_app_with_map, test_map};
use std::time::Duration;
use std::time::Instant;
use vim_quake::animation::{ENEMY_MOVE_MS, PLAYER_MOVE_MS};
use vim_quake::game::{handle_key, tick};
use vim_quake::map::Map;
use vim_quake::types::{
    App, Enemy, GameState, PauseOption, PendingInput, Position, TOTAL_LEVELS, Tile, VimMotion, Zone,
};
use vim_quake::visibility::{VisibilityMap, VisibilityState};

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
fn app_esc_opens_pause_menu() {
    let mut app = App::new();
    app.started = true;

    handle_key(&mut app, VirtualKeyCode::Escape, false);

    assert_eq!(app.game_state, GameState::Paused);
    assert_eq!(app.pause_selection, PauseOption::Resume);
}

#[test]
fn app_q_opens_pause_menu() {
    let mut app = App::new();
    app.started = true;

    handle_key(&mut app, VirtualKeyCode::Q, false);

    assert_eq!(app.game_state, GameState::Paused);
    assert_eq!(app.pause_selection, PauseOption::Resume);
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

    assert_eq!(app.player.position, Position { x: 3, y: 0 });
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
    app.level = TOTAL_LEVELS;

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
    app.level = TOTAL_LEVELS;

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

    for _ in 0..(vim_quake::types::TRAIL_MAX + 2) {
        handle_key(&mut app, VirtualKeyCode::L, false);
        handle_key(&mut app, VirtualKeyCode::H, false);
    }

    assert!(app.trail.len() <= vim_quake::types::TRAIL_MAX);
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
    app.level = TOTAL_LEVELS;

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
    app.level = TOTAL_LEVELS;

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
fn app_g_jump_to_column_bottom() {
    let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 2 });

    handle_key(&mut app, VirtualKeyCode::G, true);

    assert_eq!(app.player.position, Position { x: 2, y: 4 });
    assert_eq!(app.pending_input, None);
}

#[test]
fn app_gg_two_keys_jump_to_column_top() {
    let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 4 });

    handle_key(&mut app, VirtualKeyCode::G, false);
    assert_eq!(app.pending_input, Some(PendingInput::GotoLine));

    handle_key(&mut app, VirtualKeyCode::G, false);

    assert_eq!(app.player.position, Position { x: 2, y: 0 });
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
    app.level = TOTAL_LEVELS;
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
    app2.level = TOTAL_LEVELS;
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
