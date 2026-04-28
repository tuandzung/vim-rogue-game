use vim_rogue::map::Map;
use vim_rogue::types::{Position, Tile, Zone};

#[test]
fn map_new_creates_80x40_grid() {
    let map = Map::new();

    assert_eq!(map.width, 80);
    assert_eq!(map.height, 40);
    assert_eq!(map.grid.len(), 40);
    assert!(map.grid.iter().all(|row| row.len() == 80));
}

#[test]
fn map_start_is_floor() {
    let map = Map::new();

    assert_eq!(map.get_tile(2, 2), Tile::Floor);
}

#[test]
fn map_exit_is_exit_tile() {
    let map = Map::new();

    assert_eq!(map.get_tile(76, 36), Tile::Exit);
}

#[test]
fn map_default_tiles_are_wall() {
    let map = Map::new();

    assert_eq!(map.get_tile(0, 0), Tile::Wall);
    assert_eq!(map.get_tile(79, 0), Tile::Wall);
    assert_eq!(map.get_tile(0, 39), Tile::Wall);
}

#[test]
fn map_get_tile_out_of_bounds_returns_wall() {
    let map = Map::new();

    assert_eq!(map.get_tile(80, 0), Tile::Wall);
    assert_eq!(map.get_tile(0, 40), Tile::Wall);
    assert_eq!(map.get_tile(80, 40), Tile::Wall);
}

#[test]
fn map_set_and_get_roundtrip() {
    let mut map = Map::new();

    map.set_tile(10, 10, Tile::Obstacle);

    assert_eq!(map.get_tile(10, 10), Tile::Obstacle);
}

#[test]
fn map_is_passable_floor() {
    let map = Map::new();

    assert!(map.is_passable(2, 2));
}

#[test]
fn map_is_passable_exit() {
    let map = Map::new();

    assert!(map.is_passable(76, 36));
}

#[test]
fn map_is_passable_wall() {
    let map = Map::new();

    assert!(!map.is_passable(0, 0));
}

#[test]
fn map_is_passable_obstacle() {
    let map = Map::new();

    assert!(!map.is_passable(69, 36));
}

#[test]
fn map_zone_at_zone1() {
    let map = Map::new();

    assert_eq!(map.zone_at(Position { x: 15, y: 0 }), Zone::Zone1);
}

#[test]
fn map_zone_at_zone2() {
    let map = Map::new();

    assert_eq!(map.zone_at(Position { x: 16, y: 0 }), Zone::Zone2);
    assert_eq!(map.zone_at(Position { x: 31, y: 0 }), Zone::Zone2);
}

#[test]
fn map_zone_at_zone3() {
    let map = Map::new();

    assert_eq!(map.zone_at(Position { x: 32, y: 0 }), Zone::Zone3);
    assert_eq!(map.zone_at(Position { x: 47, y: 0 }), Zone::Zone3);
}

#[test]
fn map_zone_at_zone4() {
    let map = Map::new();

    assert_eq!(map.zone_at(Position { x: 48, y: 0 }), Zone::Zone4);
    assert_eq!(map.zone_at(Position { x: 63, y: 0 }), Zone::Zone4);
}

#[test]
fn map_zone_at_zone5() {
    let map = Map::new();

    assert_eq!(map.zone_at(Position { x: 64, y: 0 }), Zone::Zone5);
    assert_eq!(map.zone_at(Position { x: 79, y: 0 }), Zone::Zone5);
}

#[test]
fn map_has_carved_corridors() {
    let map = Map::new();
    let carved_tiles = map.grid.iter().flatten().filter(|tile| **tile != Tile::Wall).count();

    assert!(carved_tiles > 0);
}

#[test]
fn map_has_obstacles_in_zone5() {
    let map = Map::new();
    let zone5_obstacles = map
        .grid
        .iter()
        .enumerate()
        .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, tile)| (x, y, tile)))
        .filter(|(x, _, tile)| *x >= 64 && **tile == Tile::Obstacle)
        .count();

    assert!(zone5_obstacles > 0);
}

#[test]
fn map_level_1_is_same_as_new() {
    let default_map = Map::new();
    let level_1 = Map::level(1);

    assert_eq!(default_map.start, level_1.start);
    assert_eq!(default_map.exit, level_1.exit);
    assert_eq!(default_map.grid, level_1.grid);
}

#[test]
fn map_level_2_has_different_start_and_exit() {
    let map = Map::level(2);

    assert_ne!(map.start, Map::new().start);
    assert_ne!(map.exit, Map::new().exit);
}

#[test]
fn map_level_2_is_80x40() {
    let map = Map::level(2);

    assert_eq!(map.width, 80);
    assert_eq!(map.height, 40);
}

#[test]
fn map_level_2_start_is_floor() {
    let map = Map::level(2);

    assert_eq!(map.get_tile(map.start.x, map.start.y), Tile::Floor);
}

#[test]
fn map_level_2_exit_is_exit_tile() {
    let map = Map::level(2);

    assert_eq!(map.get_tile(map.exit.x, map.exit.y), Tile::Exit);
}

#[test]
fn map_level_2_has_obstacles_in_earlier_zones() {
    let map = Map::level(2);
    let early_obstacles: usize = map
        .grid
        .iter()
        .enumerate()
        .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, tile)| (x, y, tile)))
        .filter(|(x, _, tile)| *x < 64 && **tile == Tile::Obstacle)
        .count();

    assert!(early_obstacles > 0);
}

#[test]
fn map_level_2_has_carved_corridors() {
    let map = Map::level(2);
    let carved = map.grid.iter().flatten().filter(|t| **t != Tile::Wall).count();

    assert!(carved > 50);
}

#[test]
fn map_level_2_has_more_obstacles_than_level_1() {
    let level1 = Map::level(1);
    let level2 = Map::level(2);
    let obs1 = level1.grid.iter().flatten().filter(|t| **t == Tile::Obstacle).count();
    let obs2 = level2.grid.iter().flatten().filter(|t| **t == Tile::Obstacle).count();

    assert!(obs2 > obs1);
}

#[test]
fn map_level_invalid_falls_back_to_level_1() {
    let level_0 = Map::level(0);
    let level_99 = Map::level(99);
    let level_1 = Map::level(1);

    assert_eq!(level_0.start, level_1.start);
    assert_eq!(level_99.start, level_1.start);
}

#[test]
fn map_level_2_start_to_exit_is_reachable() {
    use std::collections::VecDeque;

    let map = Map::level(2);
    let mut visited = vec![vec![false; map.width]; map.height];
    let mut queue = VecDeque::new();

    queue.push_back(map.start);
    visited[map.start.y][map.start.x] = true;

    let directions: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    while let Some(pos) = queue.pop_front() {
        if pos == map.exit {
            return;
        }

        for (dx, dy) in &directions {
            let nx = pos.x as isize + dx;
            let ny = pos.y as isize + dy;

            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;

                if nx < map.width && ny < map.height && !visited[ny][nx] && map.is_passable(nx, ny)
                {
                    visited[ny][nx] = true;
                    queue.push_back(Position { x: nx, y: ny });
                }
            }
        }
    }

    panic!("Level 2 exit is not reachable from start!");
}

#[test]
fn map_level_3_is_80x40() {
    let map = Map::level(3);

    assert_eq!(map.width, 80);
    assert_eq!(map.height, 40);
    assert_eq!(map.grid.len(), 40);
    assert!(map.grid.iter().all(|row| row.len() == 80));
}

#[test]
fn map_level_3_start_is_floor() {
    let map = Map::level(3);

    assert_eq!(map.get_tile(map.start.x, map.start.y), Tile::Floor);
}

#[test]
fn map_level_3_exit_is_exit_tile() {
    let map = Map::level(3);

    assert_eq!(map.get_tile(map.exit.x, map.exit.y), Tile::Exit);
}

#[test]
fn map_level_3_has_enemy_spawns() {
    let map = Map::level(3);

    assert!(!map.enemy_spawns.is_empty());
    assert!(map.enemy_spawns.len() >= 3);
    assert!(map.enemy_spawns.len() <= 5);

    for spawn in &map.enemy_spawns {
        assert!(map.is_passable(spawn.x, spawn.y));
        let zone = map.zone_at(*spawn);
        assert!(
            zone == Zone::Zone3 || zone == Zone::Zone4 || zone == Zone::Zone5,
            "Enemy spawn at ({}, {}) is in {:?}, expected Zone 3-5",
            spawn.x,
            spawn.y,
            zone
        );
    }
}

#[test]
fn map_level_3_start_to_exit_is_reachable() {
    use std::collections::VecDeque;

    let map = Map::level(3);
    let mut visited = vec![vec![false; map.width]; map.height];
    let mut queue = VecDeque::new();

    queue.push_back(map.start);
    visited[map.start.y][map.start.x] = true;

    let directions: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    while let Some(pos) = queue.pop_front() {
        if pos == map.exit {
            return;
        }

        for (dx, dy) in &directions {
            let nx = pos.x as isize + dx;
            let ny = pos.y as isize + dy;

            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;

                if nx < map.width && ny < map.height && !visited[ny][nx] && map.is_passable(nx, ny)
                {
                    visited[ny][nx] = true;
                    queue.push_back(Position { x: nx, y: ny });
                }
            }
        }
    }

    panic!("Level 3 exit is not reachable from start!");
}

#[test]
fn map_level_1_and_2_have_no_enemy_spawns() {
    let level1 = Map::level(1);
    let level2 = Map::level(2);

    assert!(level1.enemy_spawns.is_empty());
    assert!(level2.enemy_spawns.is_empty());
}

#[test]
fn level_4_dimensions() {
    let map = Map::level(4);
    assert_eq!(map.width, 80);
    assert_eq!(map.height, 40);
    assert_eq!(map.grid.len(), 40);
    assert!(map.grid.iter().all(|row| row.len() == 80));
}

#[test]
fn level_4_has_torchlights() {
    let map = Map::level(4);
    let torchlight_count = map.grid.iter().flatten().filter(|t| **t == Tile::Torchlight).count();
    assert!(
        torchlight_count >= 2,
        "Level 4 should have at least 2 torchlights, found {}",
        torchlight_count
    );
}

#[test]
fn level_4_has_enemy_spawns() {
    let map = Map::level(4);
    assert!(
        map.enemy_spawns.len() >= 5,
        "Level 4 should have at least 5 enemy spawns, found {}",
        map.enemy_spawns.len()
    );

    for spawn in &map.enemy_spawns {
        assert!(
            map.is_passable(spawn.x, spawn.y),
            "Enemy spawn at ({}, {}) should be on a passable tile",
            spawn.x,
            spawn.y
        );
    }
}

#[test]
fn level_4_start_and_exit() {
    let map = Map::level(4);
    assert!(
        map.get_tile(map.start.x, map.start.y) == Tile::Floor
            || map.get_tile(map.start.x, map.start.y) == Tile::Torchlight
    );
    assert_eq!(map.get_tile(map.exit.x, map.exit.y), Tile::Exit);
}

#[test]
fn level_4_reachable() {
    use std::collections::VecDeque;

    let map = Map::level(4);
    let mut visited = vec![vec![false; map.width]; map.height];
    let mut queue = VecDeque::new();

    queue.push_back(map.start);
    visited[map.start.y][map.start.x] = true;

    let directions: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    while let Some(pos) = queue.pop_front() {
        if pos == map.exit {
            return;
        }

        for (dx, dy) in &directions {
            let nx = pos.x as isize + dx;
            let ny = pos.y as isize + dy;

            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;

                if nx < map.width && ny < map.height && !visited[ny][nx] && map.is_passable(nx, ny)
                {
                    visited[ny][nx] = true;
                    queue.push_back(Position { x: nx, y: ny });
                }
            }
        }
    }

    panic!("Level 4 exit is not reachable from start!");
}

#[test]
fn level_4_torchlights_are_passable() {
    let map = Map::level(4);
    for y in 0..map.height {
        for x in 0..map.width {
            if map.get_tile(x, y) == Tile::Torchlight {
                assert!(map.is_passable(x, y), "Torchlight at ({},{}) should be passable", x, y);
            }
        }
    }
}

#[test]
fn level_3_has_torchlights() {
    let map = Map::level(3);
    let torchlight_count = map.grid.iter().flatten().filter(|t| **t == Tile::Torchlight).count();
    assert!(
        torchlight_count >= 1,
        "Level 3 should have at least 1 torchlight, found {}",
        torchlight_count
    );
}

#[test]
fn level_3_torchlights_are_passable() {
    let map = Map::level(3);
    for y in 0..map.height {
        for x in 0..map.width {
            if map.get_tile(x, y) == Tile::Torchlight {
                assert!(map.is_passable(x, y), "Torchlight at ({},{}) should be passable", x, y);
            }
        }
    }
}

#[test]
fn level_4_no_torchlight_rooms_have_at_least_two_enemies() {
    let map = Map::level(4);

    let rooms_without_torchlights: &[((usize, usize), (usize, usize), &str)] = &[
        ((4, 15), (2, 9), "Room 1"),
        ((30, 43), (16, 23), "Room 3"),
        ((60, 75), (30, 37), "Room 5"),
    ];

    for &((x_min, x_max), (y_min, y_max), name) in rooms_without_torchlights {
        let count = map
            .enemy_spawns
            .iter()
            .filter(|pos| pos.x >= x_min && pos.x <= x_max && pos.y >= y_min && pos.y <= y_max)
            .count();
        assert!(count >= 2, "{} should have at least 2 enemies, found {}", name, count);
    }
}

#[test]
fn level_4_patrol_areas_match_spawn_count() {
    let map = Map::level(4);
    assert_eq!(
        map.enemy_spawns.len(),
        map.enemy_patrol_areas.len(),
        "Patrol areas should match spawn count"
    );
}

#[test]
fn level_4_spawns_are_within_their_patrol_areas() {
    let map = Map::level(4);
    for (i, spawn) in map.enemy_spawns.iter().enumerate() {
        let area = map.enemy_patrol_areas[i];
        assert!(
            area.contains(spawn.x, spawn.y),
            "Spawn at ({}, {}) not within its patrol area ({},{})-({},{})",
            spawn.x,
            spawn.y,
            area.min_x,
            area.min_y,
            area.max_x,
            area.max_y
        );
    }
}

#[test]
fn level_4_room_2_has_torchlight() {
    let map = Map::level(4);
    let has_torchlight = (50..=63).any(|x| (2..=9).any(|y| map.get_tile(x, y) == Tile::Torchlight));
    assert!(has_torchlight, "Room 2 (x:50-63, y:2-9) should contain a torchlight checkpoint");
}

#[test]
fn level_4_room_4_has_torchlight() {
    let map = Map::level(4);
    let has_torchlight =
        (8..=21).any(|x| (30..=37).any(|y| map.get_tile(x, y) == Tile::Torchlight));
    assert!(has_torchlight, "Room 4 (x:8-21, y:30-37) should contain a torchlight checkpoint");
}
