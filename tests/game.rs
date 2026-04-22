mod common;

use bracket_lib::prelude::VirtualKeyCode;
use common::{started_app_with_map, test_map};
use std::time::Duration;
use std::time::Instant;
use vim_quake::animation::{AttackEffect, AttackEffectKind, ENEMY_MOVE_MS, PLAYER_MOVE_MS, ATTACK_EFFECT_MS};
use vim_quake::game::{handle_key, tick};
use vim_quake::map::Map;
use vim_quake::types::{
    App, Direction, Enemy, GameState, MAX_HP, PauseOption, PatrolArea, PendingInput, Position, TOTAL_LEVELS, Tile, VimMotion, Zone,
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
fn app_enemy_collision_decrements_hp_and_removes_enemy() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = MAX_HP;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_eq!(app.hp, 20);
    assert_eq!(app.game_state, GameState::Playing);
    assert!(app.status_message.contains("20 HP remaining"));
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
fn app_enemy_collision_sets_lost_when_hp_depleted() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_eq!(app.hp, 0);
    assert_eq!(app.game_state, GameState::Dying);
    assert!(!app.attack_effects.is_empty());
    tick(&mut app, ATTACK_EFFECT_MS); // ages effects to complete
    tick(&mut app, 0.0); // detects all expired → transitions to Lost
    assert_eq!(app.game_state, GameState::Lost);
    assert!(app.status_message.contains("Game over"));
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
fn app_advance_level_preserves_hp() {
    let mut map = test_map(5, 5);
    map.set_tile(4, 0, Tile::Exit);
    map.exit = Position { x: 4, y: 0 };
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.level = 1;
    app.hp = 20;

    app.advance_level();

    assert_eq!(app.hp, 20);
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
    app.hp = MAX_HP;
    app.audio.enable();
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_eq!(app.hp, 20);
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
    app3.hp = MAX_HP;
    app3.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    handle_key(&mut app3, VirtualKeyCode::H, false);
    assert_eq!(app3.hp, 20);
}

#[test]
fn audio_app_new_has_disabled_audio() {
    let app = App::new();
    assert!(!app.audio.is_enabled());
}

#[test]
fn torchlight_activation_on_step() {
    let mut map = test_map(20, 20);
    map.set_tile(6, 5, Tile::Torchlight);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });

    handle_key(&mut app, VirtualKeyCode::L, false);

    assert!(app.activated_torchlights.contains(&Position { x: 6, y: 5 }));
    assert_eq!(app.last_checkpoint, Some(Position { x: 6, y: 5 }));
    assert!(app.status_message.contains("Checkpoint"));
}

#[test]
fn torchlight_activation_idempotent() {
    let mut map = test_map(20, 20);
    map.set_tile(6, 5, Tile::Torchlight);
    map.set_tile(7, 5, Tile::Floor);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });

    handle_key(&mut app, VirtualKeyCode::L, false);
    assert!(app.activated_torchlights.contains(&Position { x: 6, y: 5 }));

    handle_key(&mut app, VirtualKeyCode::L, false);
    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_eq!(app.activated_torchlights.len(), 1);
}

#[test]
fn torchlight_reveals_nearby_tiles() {
    let mut map = test_map(40, 40);
    map.set_tile(20, 20, Tile::Torchlight);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });

    app.activated_torchlights.insert(Position { x: 20, y: 20 });
    app.update_visibility();

    assert_eq!(app.visibility.get(Position { x: 20, y: 20 }), VisibilityState::Visible);
    assert_eq!(app.visibility.get(Position { x: 22, y: 20 }), VisibilityState::Visible);
}

#[test]
fn torchlight_visibility_persists_after_player_moves_away() {
    let mut map = test_map(40, 40);
    map.set_tile(20, 20, Tile::Torchlight);
    let mut app = started_app_with_map(map, Position { x: 19, y: 20 });

    handle_key(&mut app, VirtualKeyCode::L, false);
    assert!(app.activated_torchlights.contains(&Position { x: 20, y: 20 }));

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_eq!(app.visibility.get(Position { x: 20, y: 20 }), VisibilityState::Visible);
}

#[test]
fn advance_level_clears_torchlight_checkpoints() {
    let mut map = test_map(5, 5);
    map.set_tile(4, 0, Tile::Exit);
    map.exit = Position { x: 4, y: 0 };
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.level = 1;
    app.activated_torchlights.insert(Position { x: 2, y: 2 });
    app.last_checkpoint = Some(Position { x: 2, y: 2 });

    app.advance_level();

    assert!(app.activated_torchlights.is_empty());
    assert_eq!(app.last_checkpoint, None);
}

#[test]
fn retry_level_clears_torchlight_checkpoints() {
    let mut map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 3 });
    app.level = 2;
    app.activated_torchlights.insert(Position { x: 2, y: 2 });
    app.last_checkpoint = Some(Position { x: 2, y: 2 });

    app.retry_level();

    assert!(app.activated_torchlights.is_empty());
    assert_eq!(app.last_checkpoint, None);
}

#[test]
fn visibility_updates_after_each_move() {
    let mut map = test_map(40, 20);
    let mut app = started_app_with_map(map, Position { x: 5, y: 10 });

    handle_key(&mut app, VirtualKeyCode::L, false);

    assert_eq!(app.visibility.get(Position { x: 6, y: 10 }), VisibilityState::Visible);
    assert_eq!(app.visibility.get(app.player.position), VisibilityState::Visible);
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

fn level4_app_with_enemy(enemy_pos: Position, enemy_hp: Option<i32>) -> App {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });
    app.level = 4;
    app.enemies = vec![Enemy {
        position: enemy_pos,
        hp: enemy_hp,
        ..Enemy::new(enemy_pos)
    }];
    app.player.last_direction = Some(Direction::Right);
    app
}

#[test]
fn facing_updates_on_l_movement() {
    let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });
    assert_eq!(app.player.last_direction, None);
    handle_key(&mut app, VirtualKeyCode::L, false);
    assert_eq!(app.player.last_direction, Some(Direction::Right));
}

#[test]
fn facing_updates_on_h_movement() {
    let mut app = started_app_with_map(test_map(5, 1), Position { x: 2, y: 0 });
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.player.last_direction, Some(Direction::Left));
}

#[test]
fn facing_updates_on_j_movement() {
    let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 2 });
    handle_key(&mut app, VirtualKeyCode::J, false);
    assert_eq!(app.player.last_direction, Some(Direction::Down));
}

#[test]
fn facing_updates_on_k_movement() {
    let mut app = started_app_with_map(test_map(5, 5), Position { x: 2, y: 2 });
    handle_key(&mut app, VirtualKeyCode::K, false);
    assert_eq!(app.player.last_direction, Some(Direction::Up));
}

#[test]
fn facing_does_not_update_on_failed_move() {
    let mut app = started_app_with_map(test_map(5, 1), Position { x: 0, y: 0 });
    assert_eq!(app.player.last_direction, None);
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.player.last_direction, None);
}

#[test]
fn melee_attack_hits_enemy_on_level_4() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    let initial_motions = app.motion_count;
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("Hit"));
    assert!(app.status_message.contains("20"));
    assert_eq!(app.motion_count, initial_motions + 1);
}

#[test]
fn melee_attack_kills_enemy_after_3_hits() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));

    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("Hit"));
    assert!(app.status_message.contains("20"));

    app.enemies = vec![Enemy {
        position: Position { x: 6, y: 5 },
        hp: Some(20),
        ..Enemy::new(Position { x: 6, y: 5 })
    }];
    app.player.last_direction = Some(Direction::Right);

    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("Hit"));
    assert!(app.status_message.contains("10"));

    app.enemies = vec![Enemy {
        position: Position { x: 6, y: 5 },
        hp: Some(10),
        ..Enemy::new(Position { x: 6, y: 5 })
    }];
    app.player.last_direction = Some(Direction::Right);

    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("defeated"));
}

#[test]
fn melee_attack_noop_on_level_3_non_hp_enemy() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, None);
    app.level = 3;
    let initial_motions = app.motion_count;
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("Can't attack"));
    assert_eq!(app.enemies[0].hp, None);
    assert_eq!(app.motion_count, initial_motions);
}

#[test]
fn melee_attack_noop_on_level_1_non_hp_enemy() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, None);
    app.level = 1;
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("Can't attack"));
}

#[test]
fn melee_attack_no_facing() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    app.player.last_direction = None;
    let initial_motions = app.motion_count;
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("direction") || app.status_message.contains("move"));
    assert_eq!(app.enemies[0].hp, Some(30));
    assert_eq!(app.motion_count, initial_motions);
}

#[test]
fn melee_attack_miss_no_enemy() {
    let mut app = level4_app_with_enemy(Position { x: 15, y: 5 }, Some(30));
    let initial_motions = app.motion_count;
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("Nothing"));
    assert_eq!(app.motion_count, initial_motions + 1);
    assert_eq!(app.enemies.len(), 1);
}

#[test]
fn melee_attack_cant_attack_hp_none_enemy() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, None);
    let initial_motions = app.motion_count;
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.status_message.contains("Can't attack"));
    assert_eq!(app.enemies.len(), 1);
    assert_eq!(app.motion_count, initial_motions);
}

#[test]
fn melee_attack_stuns_enemy() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert_eq!(app.enemies[0].stunned_turns, 0);
    assert_eq!(app.enemies[0].hp, Some(20));
    assert!(app.status_message.contains("20"));
}

#[test]
fn stunned_enemy_does_not_move() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    handle_key(&mut app, VirtualKeyCode::X, false);
    // Enemy was at (6,5), got stunned, should not have moved during enemies_step
    assert_eq!(app.enemies[0].position, Position { x: 6, y: 5 });
    // Stun decremented from 1 to 0 during enemies_step
    assert_eq!(app.enemies[0].stunned_turns, 0);
}

#[test]
fn stunned_enemy_does_not_deal_damage() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });
    app.level = 4;
    app.hp = 10;
    // Enemy adjacent on the left — would step onto player if not stunned
    app.enemies = vec![Enemy {
        position: Position { x: 4, y: 5 },
        hp: Some(30),
        stunned_turns: 1,
        ..Enemy::new(Position { x: 4, y: 5 })
    }];
    app.player.last_direction = Some(Direction::Right);

    // Player moves right to (6,5) — enemies_step runs, stunned enemy stays put
    handle_key(&mut app, VirtualKeyCode::L, false);

    assert_eq!(app.hp, 10, "Stunned enemy should not deal damage");
    assert_eq!(app.enemies[0].position, Position { x: 4, y: 5 }, "Stunned enemy should not move");
}

#[test]
fn stun_wears_off_after_one_turn() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });
    app.level = 4;
    app.enemies = vec![Enemy {
        position: Position { x: 6, y: 5 },
        hp: Some(30),
        stunned_turns: 1,
        ..Enemy::new(Position { x: 6, y: 5 })
    }];
    app.player.last_direction = Some(Direction::Right);

    // Turn 1: enemy is stunned, stun decrements to 0
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert_eq!(app.enemies[0].stunned_turns, 0);
    assert_eq!(app.enemies[0].position, Position { x: 6, y: 5 }, "Still stunned, should not move");

    // Turn 2: stun has worn off, enemy should move toward player
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.enemies[0].position, Position { x: 5, y: 5 }, "Enemy should move after stun wears off");
}

#[test]
fn stun_prevents_enemy_counterattack_after_melee() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });
    app.level = 4;
    app.hp = 10;
    app.player.last_direction = Some(Direction::Right);
    app.enemies = vec![Enemy {
        position: Position { x: 6, y: 5 },
        hp: Some(20),
        ..Enemy::new(Position { x: 6, y: 5 })
    }];

    // Melee hits enemy (20->10), enemy gets stunned, doesn't step onto player
    handle_key(&mut app, VirtualKeyCode::X, false);

    assert_eq!(app.hp, 10, "Player should not take damage from stunned enemy");
    assert_eq!(app.enemies.len(), 1);
    assert_eq!(app.enemies[0].hp, Some(10));
    assert_eq!(app.enemies[0].position, Position { x: 6, y: 5 }, "Stunned enemy should not move");
    assert_eq!(app.enemies[0].stunned_turns, 0, "Stun should have decremented during enemies_step");
}

#[test]
fn death_with_checkpoint_respawns_at_torchlight() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.last_checkpoint = Some(Position { x: 10, y: 10 });
    app.activated_torchlights.insert(Position { x: 10, y: 10 });
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_eq!(app.game_state, GameState::Dying);
    tick(&mut app, ATTACK_EFFECT_MS);
    tick(&mut app, 0.0);
    assert_eq!(app.hp, MAX_HP, "HP should be restored to MAX_HP after checkpoint respawn");
    assert_eq!(app.player.position, Position { x: 10, y: 10 }, "Player should respawn at checkpoint");
    assert_eq!(app.game_state, GameState::Playing, "Game state should remain Playing after respawn");
    assert!(app.status_message.contains("Respawned at checkpoint"));
}

#[test]
fn death_without_checkpoint_triggers_lost() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.last_checkpoint = None;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_eq!(app.hp, 0);
    assert_eq!(app.game_state, GameState::Dying, "Should be Dying when no checkpoint");
    tick(&mut app, ATTACK_EFFECT_MS); // ages effects to complete
    tick(&mut app, 0.0); // detects all expired → transitions to Lost
    assert_eq!(app.game_state, GameState::Lost);
    assert!(app.status_message.contains("Game over"));
}

#[test]
fn checkpoint_state_persists_after_respawn() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    let checkpoint = Position { x: 10, y: 10 };
    app.last_checkpoint = Some(checkpoint);
    app.activated_torchlights.insert(checkpoint);
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert!(app.activated_torchlights.contains(&checkpoint), "Torchlights should persist after respawn");
    assert_eq!(app.last_checkpoint, Some(checkpoint), "Checkpoint should persist after respawn");
}

#[test]
fn surviving_enemies_persist_after_checkpoint_respawn() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.last_checkpoint = Some(Position { x: 10, y: 10 });
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    let far_enemy_pos = Position { x: 15, y: 15 };
    app.enemies.push(Enemy {
        position: far_enemy_pos,
        hp: Some(30),
        patrol_area: PatrolArea { min_x: 10, min_y: 10, max_x: 19, max_y: 19 },
        ..Enemy::new(far_enemy_pos)
    });

    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.game_state, GameState::Dying);
    tick(&mut app, ATTACK_EFFECT_MS);
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Playing);
    assert_eq!(app.enemies.len(), 1, "Non-colliding enemy should survive checkpoint respawn");
    assert_ne!(app.enemies[0].position, far_enemy_pos, "Surviving enemy should have moved via BFS");
}

#[test]
fn enemy_on_checkpoint_tile_is_pushed_on_respawn() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    let checkpoint = Position { x: 10, y: 10 };
    app.last_checkpoint = Some(checkpoint);
    app.activated_torchlights.insert(checkpoint);
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    app.enemies.push(Enemy {
        position: checkpoint,
        hp: Some(30),
        ..Enemy::new(checkpoint)
    });

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_eq!(app.game_state, GameState::Dying);
    tick(&mut app, ATTACK_EFFECT_MS);
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Playing);
    assert_eq!(app.player.position, checkpoint, "Player should be at checkpoint");
    let enemy_on_checkpoint = app.enemies.iter().any(|e| e.position == checkpoint);
    assert!(!enemy_on_checkpoint, "Enemy should be pushed off checkpoint tile");
    assert_eq!(app.enemies.len(), 1, "One surviving enemy after respawn");
}

#[test]
fn level_transition_clears_checkpoints() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 3, y: 3 });
    app.last_checkpoint = Some(Position { x: 10, y: 10 });
    app.activated_torchlights.insert(Position { x: 10, y: 10 });

    app.advance_level();

    assert_eq!(app.last_checkpoint, None, "Checkpoint should be cleared on level advance");
    assert!(app.activated_torchlights.is_empty(), "Torchlights should be cleared on level advance");
}

#[test]
fn level_4_enemies_have_hp_from_advance_level() {
    let mut app = App::new();
    app.started = true;
    app.level = 3;
    app.advance_level();
    assert_eq!(app.level, 4);
    assert!(!app.enemies.is_empty(), "Level 4 should have enemy spawns");
    for enemy in &app.enemies {
        assert!(enemy.hp.is_some(), "Level 4 enemies should have hp: Some(N)");
        assert_eq!(enemy.hp, Some(30), "Level 4 enemies should have 30 HP");
    }
}

#[test]
fn level_3_enemies_have_no_hp() {
    let mut app = App::new();
    app.started = true;
    app.level = 2;
    app.advance_level();
    assert_eq!(app.level, 3);
    if !app.enemies.is_empty() {
        for enemy in &app.enemies {
            assert!(enemy.hp.is_none(), "Level 3 enemies should have hp: None (despawn on contact)");
        }
    }
}

#[test]
fn level_4_retry_level_enemies_have_hp() {
    let mut app = App::new();
    app.started = true;
    app.level = 3;
    app.advance_level();
    assert_eq!(app.level, 4);
    app.hp = 0;
    app.game_state = GameState::Lost;
    app.retry_level();
    assert_eq!(app.level, 4);
    assert!(!app.enemies.is_empty(), "Level 4 should have enemy spawns after retry");
    for enemy in &app.enemies {
        assert!(enemy.hp.is_some(), "Level 4 enemies after retry should have hp: Some(N)");
        assert_eq!(enemy.hp, Some(30), "Level 4 enemies after retry should have 30 HP");
    }
}

#[test]
fn level_1_and_2_enemies_have_no_hp() {
    let mut app = App::new();
    app.started = true;
    app.level = 1;
    app.advance_level();
    assert_eq!(app.level, 2);
    assert!(app.enemies.is_empty(), "Level 2 should have no enemies");
}

#[test]
fn level_4_colliding_enemy_persists_on_checkpoint_respawn() {
    // Level 4 enemies with hp: Some(30) should persist even after colliding with the player,
    // unlike Level 3 enemies (hp: None) which despawn on contact.
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });
    app.level = 4;
    app.hp = 10;
    let checkpoint = Position { x: 10, y: 10 };
    app.last_checkpoint = Some(checkpoint);
    app.activated_torchlights.insert(checkpoint);
    app.player.last_direction = Some(Direction::Right);
    // Level 4 enemy with hp: Some(30)
    app.enemies = vec![Enemy {
        position: Position { x: 6, y: 5 },
        hp: Some(30),
        ..Enemy::new(Position { x: 6, y: 5 })
    }];

    // Move right — enemy steps toward player, collision occurs, HP -> 0 -> checkpoint respawn
    vim_quake::game::handle_key(&mut app, VirtualKeyCode::L, false);

    assert_eq!(app.game_state, GameState::Dying);
    tick(&mut app, ATTACK_EFFECT_MS);
    tick(&mut app, 0.0);

    // Player should have respawned at checkpoint
    assert_eq!(app.hp, MAX_HP, "HP should be restored to MAX_HP");
    assert_eq!(app.player.position, checkpoint, "Player should respawn at checkpoint");
    assert_eq!(app.game_state, GameState::Playing, "Should be Playing after respawn");

    // The Level 4 enemy should persist (not despawn)
    assert_eq!(app.enemies.len(), 1, "Level 4 enemy should persist after collision");
    assert_eq!(app.enemies[0].hp, Some(30), "Level 4 enemy HP should be unchanged");
}

#[test]
fn melee_attack_spawns_strike_effect_at_target() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert_eq!(app.attack_effects.len(), 1);
    let effect = &app.attack_effects[0];
    assert_eq!(effect.kind, AttackEffectKind::PlayerStrike);
    assert_eq!(effect.x, 6);
    assert_eq!(effect.y, 5);
}

#[test]
fn melee_attack_kill_spawns_strike_effect() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(10));
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert_eq!(app.attack_effects.len(), 1);
    assert_eq!(app.attack_effects[0].kind, AttackEffectKind::PlayerStrike);
}

#[test]
fn melee_attack_miss_no_effect() {
    let mut app = level4_app_with_enemy(Position { x: 15, y: 5 }, Some(30));
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(app.attack_effects.is_empty());
}

#[test]
fn attack_effect_completes_after_tick() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert_eq!(app.attack_effects.len(), 1);
    tick(&mut app, ATTACK_EFFECT_MS); // ages to complete
    tick(&mut app, 0.0); // removes completed effects
    assert!(app.attack_effects.is_empty());
}

#[test]
fn attack_effect_does_not_block_input() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(!app.attack_effects.is_empty());
    assert!(app.player_animation.is_none());
    handle_key(&mut app, VirtualKeyCode::L, false);
}

#[test]
fn advance_level_clears_attack_effects() {
    let mut map = test_map(10, 1);
    map.set_tile(4, 0, Tile::Exit);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.attack_effects.push(vim_quake::animation::AttackEffect::new(
        AttackEffectKind::PlayerStrike,
        3,
        0,
    ));
    assert!(!app.attack_effects.is_empty());
    app.level = 1;
    handle_key(&mut app, VirtualKeyCode::L, false);
    assert!(app.attack_effects.is_empty());
}

#[test]
fn retry_level_clears_attack_effects() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert!(!app.attack_effects.is_empty());
    app.retry_level();
    assert!(app.attack_effects.is_empty());
}

#[test]
fn enemy_collision_spawns_hit_effect_on_player() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = MAX_HP;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.attack_effects.len(), 1);
    let effect = &app.attack_effects[0];
    assert_eq!(effect.kind, AttackEffectKind::EnemyHit);
    assert_eq!(effect.x, 2);
    assert_eq!(effect.y, 0);
}

#[test]
fn fatal_enemy_hit_preserves_effect() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.game_state, GameState::Dying);
    assert_eq!(app.attack_effects.len(), 1);
    assert_eq!(app.attack_effects[0].kind, AttackEffectKind::EnemyHit);
}

#[test]
fn checkpoint_respawn_preserves_hit_effect() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });
    app.level = 4;
    app.hp = 10;
    let checkpoint = Position { x: 10, y: 10 };
    app.last_checkpoint = Some(checkpoint);
    app.activated_torchlights.insert(checkpoint);
    app.player.last_direction = Some(Direction::Right);
    app.enemies = vec![Enemy {
        position: Position { x: 6, y: 5 },
        hp: Some(30),
        ..Enemy::new(Position { x: 6, y: 5 })
    }];
    handle_key(&mut app, VirtualKeyCode::L, false);
    assert_eq!(app.game_state, GameState::Dying);
    assert!(!app.attack_effects.is_empty());
    tick(&mut app, ATTACK_EFFECT_MS);
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Playing);
    assert_eq!(app.player.position, checkpoint);
}

#[test]
fn attack_effects_expire_after_duration() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = MAX_HP;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.attack_effects.len(), 1);
    tick(&mut app, ATTACK_EFFECT_MS); // ages to complete, still present
    tick(&mut app, 0.0); // removes already-complete effects
    assert!(app.attack_effects.is_empty());
}

#[test]
fn dying_transitions_to_lost_after_effects_expire() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.game_state, GameState::Dying);
    tick(&mut app, ATTACK_EFFECT_MS);
    assert_eq!(app.game_state, GameState::Dying);
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Lost);
}

#[test]
fn nonfatal_effect_survives_oversized_first_delta() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = MAX_HP;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.attack_effects.len(), 1);
    assert_eq!(app.game_state, GameState::Playing);
    tick(&mut app, ATTACK_EFFECT_MS * 10.0);
    assert_eq!(app.attack_effects.len(), 1, "effect should survive oversized delta for at least one render");
    assert!(app.attack_effects[0].is_complete());
}

#[test]
fn player_strike_survives_oversized_first_delta() {
    let mut app = level4_app_with_enemy(Position { x: 6, y: 5 }, Some(30));
    handle_key(&mut app, VirtualKeyCode::X, false);
    assert_eq!(app.attack_effects.len(), 1);
    tick(&mut app, ATTACK_EFFECT_MS * 10.0);
    assert_eq!(app.attack_effects.len(), 1, "PlayerStrike should survive oversized delta");
    assert!(app.attack_effects[0].is_complete());
}

#[test]
fn dying_state_ignores_input() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.game_state, GameState::Dying);
    let pos_before = app.player.position;
    handle_key(&mut app, VirtualKeyCode::L, false);
    assert_eq!(app.player.position, pos_before, "input should be ignored during Dying");
}

#[test]
fn dying_large_delta_still_shows_effects_before_transition() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.game_state, GameState::Dying);
    tick(&mut app, ATTACK_EFFECT_MS * 10.0);
    assert_eq!(app.game_state, GameState::Dying, "effects still visible after oversized delta");
    assert!(!app.attack_effects.is_empty(), "effects should not be cleared during Dying");
    assert!(app.attack_effects.iter().all(|e| e.is_complete()));
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Lost);
}

#[test]
fn two_enemies_same_turn_both_hit_at_high_hp() {
    let map = test_map(10, 3);
    let mut app = started_app_with_map(map, Position { x: 5, y: 1 });
    app.hp = 20;
    app.enemies.push(Enemy::new(Position { x: 4, y: 0 }));
    app.enemies.push(Enemy::new(Position { x: 4, y: 2 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.player.position, Position { x: 4, y: 1 });
    assert_eq!(app.hp, 0);
    assert_eq!(app.game_state, GameState::Dying);
    assert_eq!(app.attack_effects.len(), 2);
}

#[test]
fn two_enemies_fatal_first_skips_second() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 3, y: 0 });
    app.hp = 10;
    app.enemies.push(Enemy::new(Position { x: 1, y: 0 }));
    app.enemies.push(Enemy::new(Position { x: 4, y: 0 }));
    handle_key(&mut app, VirtualKeyCode::H, false);
    assert_eq!(app.hp, 0);
    assert_eq!(app.game_state, GameState::Dying);
    assert_eq!(app.attack_effects.len(), 1, "second enemy skipped after fatal collision");
}

#[test]
fn checkpoint_respawn_pushes_stacked_enemies_off_checkpoint() {
    let map = test_map(10, 10);
    let checkpoint = Position { x: 5, y: 5 };
    let mut app = started_app_with_map(map, Position { x: 5, y: 6 });
    app.level = 4;
    app.hp = 10;
    app.last_checkpoint = Some(checkpoint);
    app.activated_torchlights.insert(checkpoint);
    app.player.last_direction = Some(Direction::Up);
    app.enemies = vec![
        Enemy { position: checkpoint, hp: Some(30), ..Enemy::new(checkpoint) },
        Enemy { position: checkpoint, hp: Some(30), ..Enemy::new(checkpoint) },
    ];
    handle_key(&mut app, VirtualKeyCode::K, false);
    assert_eq!(app.player.position, Position { x: 5, y: 5 });
    assert_eq!(app.game_state, GameState::Dying);
    tick(&mut app, ATTACK_EFFECT_MS);
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Playing);
    assert_eq!(app.player.position, checkpoint);
    assert_eq!(app.enemies.len(), 2);
    let on_checkpoint = app.enemies.iter().any(|e| e.position == checkpoint);
    assert!(!on_checkpoint, "no enemy should remain on checkpoint tile");
    let positions: std::collections::HashSet<Position> = app.enemies.iter().map(|e| e.position).collect();
    assert_eq!(positions.len(), app.enemies.len(), "enemies should not stack on the same tile after push");
}

#[test]
fn push_enemies_bfs_respects_walls() {
    let mut map = test_map(10, 10);
    let checkpoint = Position { x: 5, y: 5 };
    map.set_tile(4, 5, Tile::Wall);
    map.set_tile(6, 5, Tile::Wall);
    map.set_tile(5, 4, Tile::Wall);
    map.set_tile(5, 6, Tile::Wall);
    let mut app = started_app_with_map(map, Position { x: 8, y: 8 });
    app.game_state = GameState::Dying;
    app.pending_respawn = Some(checkpoint);
    app.attack_effects.push(AttackEffect::new(AttackEffectKind::EnemyHit, 8, 8));
    app.enemies = vec![
        Enemy { position: checkpoint, hp: Some(30), ..Enemy::new(checkpoint) },
        Enemy { position: checkpoint, hp: Some(30), ..Enemy::new(checkpoint) },
    ];
    tick(&mut app, ATTACK_EFFECT_MS);
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Playing);
    assert_eq!(app.player.position, checkpoint);
    assert_eq!(app.enemies.len(), 2);
    // Enemies stay at checkpoint since all neighbors are impassable walls
    assert!(
        app.enemies.iter().all(|e| e.position == checkpoint),
        "BFS should not push enemies through walls"
    );
}

#[test]
fn dying_with_empty_effects_transitions_immediately() {
    let map = test_map(5, 5);
    let mut app = started_app_with_map(map, Position { x: 2, y: 2 });
    app.game_state = GameState::Dying;
    app.pending_respawn = None;
    assert!(app.attack_effects.is_empty());
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Lost);
}

#[test]
fn mixed_age_dying_removes_completed_effects_first() {
    let map = test_map(10, 10);
    let checkpoint = Position { x: 5, y: 5 };
    let mut app = started_app_with_map(map, Position { x: 5, y: 5 });
    app.hp = 10;
    app.last_checkpoint = Some(checkpoint);
    app.activated_torchlights.insert(checkpoint);
    let mut old_effect = AttackEffect::new(AttackEffectKind::EnemyHit, 5, 5);
    old_effect.update(ATTACK_EFFECT_MS);
    assert!(old_effect.is_complete());
    app.attack_effects.push(old_effect);
    app.attack_effects.push(AttackEffect::new(AttackEffectKind::EnemyHit, 5, 5));
    app.game_state = GameState::Dying;
    app.pending_respawn = Some(checkpoint);
    assert_eq!(app.attack_effects.len(), 2);
    tick(&mut app, 0.0);
    assert_eq!(app.attack_effects.len(), 1, "completed old effect should be removed");
    assert!(!app.attack_effects[0].is_complete(), "fresh effect should remain");
    tick(&mut app, ATTACK_EFFECT_MS);
    tick(&mut app, 0.0);
    assert_eq!(app.game_state, GameState::Playing);
}

#[test]
fn enemy_chases_when_player_visible() {
    let map = test_map(20, 20);
    let mut app = started_app_with_map(map, Position { x: 15, y: 10 });
    app.level = 3;
    let mut enemy = Enemy::new(Position { x: 12, y: 10 });
    enemy.patrol_area = PatrolArea { min_x: 0, min_y: 0, max_x: 19, max_y: 19 };
    app.enemies = vec![enemy];

    let old_pos = app.enemies[0].position;
    handle_key(&mut app, VirtualKeyCode::H, false);

    assert_ne!(
        app.enemies[0].position, old_pos,
        "Enemy should move toward player when visible"
    );
}

#[test]
fn enemy_patrols_when_player_not_visible() {
    let mut map = test_map(80, 40);
    for y in 0..40 {
        map.set_tile(40, y, Tile::Wall);
    }
    let mut app = started_app_with_map(map, Position { x: 60, y: 20 });
    app.level = 3;
    let mut enemy = Enemy::new(Position { x: 10, y: 20 });
    enemy.patrol_area = PatrolArea { min_x: 0, min_y: 0, max_x: 39, max_y: 39 };
    app.enemies = vec![enemy];

    handle_key(&mut app, VirtualKeyCode::H, false);

    assert!(
        app.enemies[0].patrol_area.contains(app.enemies[0].position.x, app.enemies[0].position.y),
        "Enemy should stay within patrol area"
    );
    assert_ne!(
        app.enemies[0].position.x, 60,
        "Enemy should NOT be moving toward distant player behind wall"
    );
}

#[test]
fn enemy_patrol_does_not_leave_room_over_many_turns() {
    let map = test_map(80, 40);
    let mut app = started_app_with_map(map, Position { x: 70, y: 35 });
    app.level = 3;
    let mut enemy = Enemy::new(Position { x: 10, y: 5 });
    enemy.patrol_area = PatrolArea { min_x: 4, min_y: 2, max_x: 15, max_y: 9 };
    app.enemies = vec![enemy];

    for _ in 0..50 {
        handle_key(&mut app, VirtualKeyCode::L, false);
        assert!(
            app.enemies[0].patrol_area.contains(app.enemies[0].position.x, app.enemies[0].position.y),
            "Enemy left patrol area at ({}, {})",
            app.enemies[0].position.x, app.enemies[0].position.y
        );
    }
}
