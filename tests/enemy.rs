mod common;

use common::test_map;
use vim_rogue::types::{Enemy, PatrolArea, Position, Tile, Zone};

#[test]
fn enemy_new_has_default_glyph() {
    let enemy = Enemy::new(Position { x: 3, y: 5 });

    assert_eq!(enemy.position, Position { x: 3, y: 5 });
    assert_eq!(enemy.glyph, 'e');
}

#[test]
fn enemy_steps_toward_player() {
    let map = test_map(5, 5);
    let mut enemy = Enemy::new(Position { x: 0, y: 0 });
    let player_pos = Position { x: 3, y: 0 };

    let moved = enemy.step_toward_player(player_pos, &map);

    assert!(moved);
    assert_eq!(enemy.position, Position { x: 1, y: 0 });
}

#[test]
fn enemy_steps_toward_player_diagonal() {
    let map = test_map(5, 5);
    let mut enemy = Enemy::new(Position { x: 0, y: 0 });
    let player_pos = Position { x: 3, y: 3 };

    let moved = enemy.step_toward_player(player_pos, &map);

    assert!(moved);
    assert_eq!(enemy.position, Position { x: 1, y: 0 });
}

#[test]
fn enemy_does_not_walk_through_walls() {
    let mut map = test_map(5, 1);
    map.set_tile(1, 0, Tile::Wall);
    map.set_tile(2, 0, Tile::Wall);
    let mut enemy = Enemy::new(Position { x: 0, y: 0 });
    let player_pos = Position { x: 4, y: 0 };

    let moved = enemy.step_toward_player(player_pos, &map);

    assert!(!moved);
    assert_eq!(enemy.position, Position { x: 0, y: 0 });
}

#[test]
fn enemy_adjacent_moves_onto_player_tile() {
    let map = test_map(5, 5);
    let mut enemy = Enemy::new(Position { x: 1, y: 0 });
    let player_pos = Position { x: 2, y: 0 };

    let moved = enemy.step_toward_player(player_pos, &map);

    assert!(moved);
    assert_eq!(enemy.position, player_pos);
}

#[test]
fn enemy_adjacent_vertical_moves_onto_player_tile() {
    let map = test_map(5, 5);
    let mut enemy = Enemy::new(Position { x: 2, y: 1 });
    let player_pos = Position { x: 2, y: 2 };

    let moved = enemy.step_toward_player(player_pos, &map);

    assert!(moved);
    assert_eq!(enemy.position, player_pos);
}

#[test]
fn enemy_no_path_stays_put() {
    let mut map = test_map(5, 5);
    for x in 0..5 {
        map.set_tile(x, 2, Tile::Wall);
    }
    let mut enemy = Enemy::new(Position { x: 2, y: 0 });
    let player_pos = Position { x: 2, y: 4 };

    let moved = enemy.step_toward_player(player_pos, &map);

    assert!(!moved);
    assert_eq!(enemy.position, Position { x: 2, y: 0 });
}

#[test]
fn enemy_already_on_player_does_not_move() {
    let map = test_map(5, 5);
    let pos = Position { x: 2, y: 2 };
    let mut enemy = Enemy::new(pos);

    let moved = enemy.step_toward_player(pos, &map);

    assert!(!moved);
    assert_eq!(enemy.position, pos);
}

#[test]
fn enemy_follows_corridor_path() {
    let mut map = test_map(5, 5);
    for y in 0..5 {
        for x in 0..5 {
            map.set_tile(x, y, Tile::Wall);
        }
    }
    map.set_tile(0, 0, Tile::Floor);
    map.set_tile(0, 1, Tile::Floor);
    map.set_tile(0, 2, Tile::Floor);
    map.set_tile(1, 2, Tile::Floor);
    map.set_tile(2, 2, Tile::Floor);

    let mut enemy = Enemy::new(Position { x: 0, y: 0 });
    let player_pos = Position { x: 2, y: 2 };

    let moved = enemy.step_toward_player(player_pos, &map);

    assert!(moved);
    assert_eq!(enemy.position, Position { x: 0, y: 1 });
}

#[test]
fn enemy_takes_shortest_path() {
    let map = test_map(3, 3);
    let mut enemy = Enemy::new(Position { x: 0, y: 0 });
    let player_pos = Position { x: 2, y: 2 };

    let moved = enemy.step_toward_player(player_pos, &map);

    assert!(moved);
    let valid_steps = [Position { x: 1, y: 0 }, Position { x: 0, y: 1 }];
    assert!(valid_steps.contains(&enemy.position));
}

#[test]
fn enemy_los_clear_path() {
    let map = test_map(20, 20);
    let enemy = Enemy::new(Position { x: 5, y: 5 });
    assert!(enemy.has_line_of_sight(Position { x: 10, y: 5 }, &map));
}

#[test]
fn enemy_los_blocked_by_wall() {
    let mut map = test_map(20, 20);
    map.set_tile(7, 5, Tile::Wall);
    let enemy = Enemy::new(Position { x: 5, y: 5 });
    assert!(!enemy.has_line_of_sight(Position { x: 10, y: 5 }, &map));
}

#[test]
fn enemy_los_out_of_range() {
    let map = test_map(100, 100);
    let enemy = Enemy::new(Position { x: 0, y: 0 });
    assert!(!enemy.has_line_of_sight(Position { x: 50, y: 50 }, &map));
}

#[test]
fn enemy_los_same_position() {
    let map = test_map(10, 10);
    let pos = Position { x: 5, y: 5 };
    let enemy = Enemy::new(pos);
    assert!(enemy.has_line_of_sight(pos, &map));
}

#[test]
fn enemy_los_diagonal_clear() {
    let map = test_map(20, 20);
    let enemy = Enemy::new(Position { x: 2, y: 2 });
    assert!(enemy.has_line_of_sight(Position { x: 6, y: 6 }, &map));
}

#[test]
fn enemy_los_adjacent() {
    let map = test_map(10, 10);
    let enemy = Enemy::new(Position { x: 5, y: 5 });
    assert!(enemy.has_line_of_sight(Position { x: 6, y: 5 }, &map));
    assert!(enemy.has_line_of_sight(Position { x: 5, y: 6 }, &map));
    assert!(enemy.has_line_of_sight(Position { x: 4, y: 5 }, &map));
    assert!(enemy.has_line_of_sight(Position { x: 5, y: 4 }, &map));
}

#[test]
fn enemy_patrol_moves_within_area() {
    let map = test_map(20, 20);
    let mut enemy = Enemy {
        position: Position { x: 5, y: 5 },
        patrol_area: PatrolArea { min_x: 0, min_y: 0, max_x: 10, max_y: 10 },
        ..Enemy::new(Position { x: 5, y: 5 })
    };
    let moved = enemy.patrol_step(&map);
    assert!(moved);
    assert!(
        enemy.patrol_area.contains(enemy.position.x, enemy.position.y),
        "Enemy moved to ({}, {}) outside patrol area",
        enemy.position.x,
        enemy.position.y
    );
}

#[test]
fn enemy_patrol_stays_in_bounding_box() {
    let map = test_map(20, 20);
    let area = PatrolArea { min_x: 5, min_y: 5, max_x: 5, max_y: 5 };
    let mut enemy = Enemy {
        position: Position { x: 5, y: 5 },
        patrol_area: area,
        ..Enemy::new(Position { x: 5, y: 5 })
    };
    let moved = enemy.patrol_step(&map);
    assert!(!moved, "1x1 patrol area should have no valid moves");
    assert_eq!(enemy.position, Position { x: 5, y: 5 });
}

#[test]
fn enemy_patrol_does_not_cross_walls() {
    let mut map = test_map(10, 10);
    map.set_tile(4, 5, Tile::Wall);
    map.set_tile(6, 5, Tile::Wall);
    map.set_tile(5, 4, Tile::Wall);
    let mut enemy = Enemy {
        position: Position { x: 5, y: 5 },
        patrol_area: PatrolArea { min_x: 0, min_y: 0, max_x: 9, max_y: 9 },
        ..Enemy::new(Position { x: 5, y: 5 })
    };
    enemy.patrol_step(&map);
    assert_ne!(enemy.position, Position { x: 4, y: 5 });
    assert_ne!(enemy.position, Position { x: 6, y: 5 });
    assert_ne!(enemy.position, Position { x: 5, y: 4 });
}

#[test]
fn enemy_patrol_small_area_no_valid_move() {
    let mut map = test_map(10, 10);
    map.set_tile(4, 5, Tile::Wall);
    let mut enemy = Enemy {
        position: Position { x: 5, y: 5 },
        patrol_area: PatrolArea { min_x: 4, min_y: 5, max_x: 5, max_y: 5 },
        ..Enemy::new(Position { x: 5, y: 5 })
    };
    let moved = enemy.patrol_step(&map);
    assert!(!moved);
}

#[test]
fn enemy_patrol_returns_false_when_stuck() {
    let mut map = test_map(5, 5);
    for y in 0..5 {
        for x in 0..5 {
            map.set_tile(x, y, Tile::Wall);
        }
    }
    map.set_tile(2, 2, Tile::Floor);
    let mut enemy = Enemy {
        position: Position { x: 2, y: 2 },
        patrol_area: PatrolArea { min_x: 0, min_y: 0, max_x: 4, max_y: 4 },
        ..Enemy::new(Position { x: 2, y: 2 })
    };
    let moved = enemy.patrol_step(&map);
    assert!(!moved);
    assert_eq!(enemy.position, Position { x: 2, y: 2 });
}
