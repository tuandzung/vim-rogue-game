mod common;

use common::test_map;
use vim_rogue::map::Map;
use vim_rogue::types::{PlayerState, Position, Tile, VimMotion, Zone};

#[test]
fn player_new_has_starting_position() {
    let player = PlayerState::new(Position { x: 2, y: 3 });

    assert_eq!(player.position, Position { x: 2, y: 3 });
}

#[test]
fn player_new_has_empty_used_motions() {
    let player = PlayerState::new(Position { x: 2, y: 3 });

    assert!(player.used_motions.is_empty());
}

#[test]
fn player_step_right() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 1, y: 1 });

    assert!(player.handle_motion(VimMotion::L, None, &mut map));
    assert_eq!(player.position, Position { x: 2, y: 1 });
}

#[test]
fn player_step_left() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 2, y: 1 });

    assert!(player.handle_motion(VimMotion::H, None, &mut map));
    assert_eq!(player.position, Position { x: 1, y: 1 });
}

#[test]
fn player_step_down() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 1, y: 1 });

    assert!(player.handle_motion(VimMotion::J, None, &mut map));
    assert_eq!(player.position, Position { x: 1, y: 2 });
}

#[test]
fn player_step_up() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 1, y: 2 });

    assert!(player.handle_motion(VimMotion::K, None, &mut map));
    assert_eq!(player.position, Position { x: 1, y: 1 });
}

#[test]
fn player_step_into_wall_fails() {
    let mut map = test_map(5, 5);
    map.set_tile(2, 1, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 1, y: 1 });

    assert!(!player.handle_motion(VimMotion::L, None, &mut map));
    assert_eq!(player.position, Position { x: 1, y: 1 });
}

#[test]
fn player_step_at_boundary_fails() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 0, y: 1 });

    assert!(!player.handle_motion(VimMotion::H, None, &mut map));
    assert_eq!(player.position, Position { x: 0, y: 1 });
}

#[test]
fn player_step_records_motion() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 1, y: 1 });

    player.handle_motion(VimMotion::L, None, &mut map);

    assert!(player.used_motions.contains(&VimMotion::L));
}

#[test]
fn player_w_jumps_forward() {
    let mut map = test_map(6, 1);
    let mut player = PlayerState::new(Position { x: 1, y: 0 });

    assert!(player.handle_motion(VimMotion::W, None, &mut map));
    assert_eq!(player.position, Position { x: 5, y: 0 });
}

#[test]
fn player_w_stops_at_wall() {
    let mut map = test_map(6, 1);
    map.set_tile(2, 0, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 1, y: 0 });

    assert!(!player.handle_motion(VimMotion::W, None, &mut map));
    assert_eq!(player.position, Position { x: 1, y: 0 });
}

#[test]
fn player_b_jumps_backward() {
    let mut map = test_map(6, 1);
    let mut player = PlayerState::new(Position { x: 4, y: 0 });

    assert!(player.handle_motion(VimMotion::B, None, &mut map));
    assert_eq!(player.position, Position { x: 0, y: 0 });
}

#[test]
fn player_b_stops_at_wall() {
    let mut map = test_map(6, 1);
    map.set_tile(3, 0, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 4, y: 0 });

    assert!(!player.handle_motion(VimMotion::B, None, &mut map));
    assert_eq!(player.position, Position { x: 4, y: 0 });
}

#[test]
fn player_zero_jumps_to_row_start() {
    let mut map = test_map(6, 1);
    map.set_tile(0, 0, Tile::Wall);
    map.set_tile(1, 0, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 4, y: 0 });

    assert!(player.handle_motion(VimMotion::Zero, None, &mut map));
    assert_eq!(player.position, Position { x: 2, y: 0 });
}

#[test]
fn player_dollar_jumps_to_row_end() {
    let mut map = test_map(6, 1);
    map.set_tile(5, 0, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 1, y: 0 });

    assert!(player.handle_motion(VimMotion::Dollar, None, &mut map));
    assert_eq!(player.position, Position { x: 4, y: 0 });
}

#[test]
fn player_find_char_finds_target() {
    let mut map = test_map(6, 1);
    map.set_tile(4, 0, Tile::Exit);
    let mut player = PlayerState::new(Position { x: 1, y: 0 });

    assert!(player.handle_motion(VimMotion::Find, Some('>'), &mut map));
    assert_eq!(player.position, Position { x: 4, y: 0 });
}

#[test]
fn player_find_char_not_found_fails() {
    let mut map = test_map(6, 1);
    let mut player = PlayerState::new(Position { x: 1, y: 0 });

    assert!(!player.handle_motion(VimMotion::Find, Some('>'), &mut map));
    assert_eq!(player.position, Position { x: 1, y: 0 });
}

#[test]
fn player_till_char_stops_before_target() {
    let mut map = test_map(6, 1);
    map.set_tile(4, 0, Tile::Exit);
    let mut player = PlayerState::new(Position { x: 1, y: 0 });

    assert!(player.handle_motion(VimMotion::Till, Some('>'), &mut map));
    assert_eq!(player.position, Position { x: 3, y: 0 });
}

#[test]
fn player_dd_destroys_obstacle() {
    let mut map = test_map(6, 1);
    map.set_tile(3, 0, Tile::Obstacle);
    let mut player = PlayerState::new(Position { x: 1, y: 0 });

    assert!(player.handle_motion(VimMotion::DeleteLine, None, &mut map));
    assert_eq!(map.get_tile(3, 0), Tile::Floor);
}

#[test]
fn player_dd_no_obstacle_fails() {
    let mut map = test_map(6, 1);
    let mut player = PlayerState::new(Position { x: 1, y: 0 });

    assert!(!player.handle_motion(VimMotion::DeleteLine, None, &mut map));
}

#[test]
fn player_g_jumps_to_column_bottom() {
    let mut map = test_map(5, 5);
    map.set_tile(2, 4, Tile::Wall);
    map.set_tile(2, 3, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 2, y: 0 });

    assert!(player.handle_motion(VimMotion::G, None, &mut map));
    assert_eq!(player.position, Position { x: 2, y: 2 });
}

#[test]
fn player_gg_jumps_to_column_top() {
    let mut map = test_map(5, 5);
    map.set_tile(2, 0, Tile::Wall);
    map.set_tile(2, 1, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 2, y: 4 });

    assert!(player.handle_motion(VimMotion::GotoLine, None, &mut map));
    assert_eq!(player.position, Position { x: 2, y: 2 });
}

#[test]
fn player_g_no_passable_rows_fails() {
    let mut map = Map {
        grid: vec![vec![Tile::Wall; 5]; 5],
        zones: vec![vec![Zone::Zone1; 5]; 5],
        width: 5,
        height: 5,
        start: Position { x: 0, y: 0 },
        exit: Position { x: 4, y: 4 },
        enemy_spawns: vec![],
        enemy_patrol_areas: vec![],
    };
    map.set_tile(2, 2, Tile::Floor);
    let mut player = PlayerState::new(Position { x: 2, y: 2 });

    assert!(!player.handle_motion(VimMotion::G, None, &mut map));
    assert_eq!(player.position, Position { x: 2, y: 2 });
}

#[test]
fn player_gg_no_passable_in_column_fails() {
    let mut map = Map {
        grid: vec![vec![Tile::Wall; 5]; 5],
        zones: vec![vec![Zone::Zone1; 5]; 5],
        width: 5,
        height: 5,
        start: Position { x: 0, y: 0 },
        exit: Position { x: 4, y: 4 },
        enemy_spawns: vec![],
        enemy_patrol_areas: vec![],
    };
    map.set_tile(2, 2, Tile::Floor);
    let mut player = PlayerState::new(Position { x: 2, y: 2 });

    assert!(!player.handle_motion(VimMotion::GotoLine, None, &mut map));
    assert_eq!(player.position, Position { x: 2, y: 2 });
}

#[test]
fn player_g_stops_at_wall() {
    let mut map = test_map(5, 5);
    map.set_tile(2, 2, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 2, y: 1 });

    assert!(!player.handle_motion(VimMotion::G, None, &mut map));
    assert_eq!(player.position, Position { x: 2, y: 1 });
}

#[test]
fn player_gg_stops_at_wall() {
    let mut map = test_map(5, 5);
    map.set_tile(2, 3, Tile::Wall);
    let mut player = PlayerState::new(Position { x: 2, y: 4 });

    assert!(!player.handle_motion(VimMotion::GotoLine, None, &mut map));
    assert_eq!(player.position, Position { x: 2, y: 4 });
}

#[test]
fn player_g_already_on_target_row_returns_false() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 0, y: 4 });

    assert!(!player.handle_motion(VimMotion::G, None, &mut map));
    assert_eq!(player.position, Position { x: 0, y: 4 });
}

#[test]
fn player_gg_already_at_column_top_returns_false() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 0, y: 0 });

    assert!(!player.handle_motion(VimMotion::GotoLine, None, &mut map));
    assert_eq!(player.position, Position { x: 0, y: 0 });
}

#[test]
fn player_handle_motion_records_in_used_motions() {
    let motions = [
        VimMotion::H,
        VimMotion::J,
        VimMotion::K,
        VimMotion::L,
        VimMotion::W,
        VimMotion::B,
        VimMotion::Zero,
        VimMotion::Dollar,
        VimMotion::Find,
        VimMotion::Till,
        VimMotion::DeleteLine,
        VimMotion::G,
        VimMotion::GotoLine,
    ];

    for motion in motions {
        let mut map = test_map(8, 2);
        map.set_tile(2, 0, Tile::Wall);
        map.set_tile(6, 0, Tile::Exit);
        map.set_tile(5, 0, Tile::Obstacle);
        let mut player = PlayerState::new(Position { x: 4, y: 1 });
        let target = match motion {
            VimMotion::Find | VimMotion::Till => Some('>'),
            _ => None,
        };

        player.handle_motion(motion, target, &mut map);

        assert!(player.used_motions.contains(&motion));
    }
}

#[test]
fn handle_motion_increments_motion_count_on_success_and_failure() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 0, y: 0 });

    assert_eq!(player.motion_count, 0);

    // Failed motion still increments count
    assert!(!player.handle_motion(VimMotion::GotoLine, None, &mut map));
    assert_eq!(player.motion_count, 1);

    // Successful motion increments count
    assert!(player.handle_motion(VimMotion::L, None, &mut map));
    assert_eq!(player.motion_count, 2);
}

#[test]
fn handle_motion_discovered_motions_no_duplicates() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 0, y: 0 });

    assert!(player.discovered_motions.is_empty());

    // First call registers motion
    player.handle_motion(VimMotion::L, None, &mut map);
    assert!(player.discovered_motions.contains(&VimMotion::L));
    assert_eq!(player.discovered_motions.len(), 1);

    // Repeated call does not add duplicate
    player.handle_motion(VimMotion::L, None, &mut map);
    assert_eq!(player.discovered_motions.len(), 1);

    // Different motion adds to set
    player.handle_motion(VimMotion::J, None, &mut map);
    assert_eq!(player.discovered_motions.len(), 2);
}

#[test]
fn handle_motion_discovered_motions_includes_failed_attempts() {
    let mut map = test_map(5, 5);
    let mut player = PlayerState::new(Position { x: 0, y: 0 });

    // GotoLine fails at y=0 but still registers
    assert!(!player.handle_motion(VimMotion::GotoLine, None, &mut map));
    assert!(player.discovered_motions.contains(&VimMotion::GotoLine));
}
