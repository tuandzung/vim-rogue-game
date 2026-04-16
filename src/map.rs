use crate::types::{Position, Tile, Zone};

pub struct Map {
    pub grid: Vec<Vec<Tile>>,
    pub zones: Vec<Vec<Zone>>,
    pub width: usize,
    pub height: usize,
    pub start: Position,
    pub exit: Position,
    pub enemy_spawns: Vec<Position>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Self {
        let width = 80;
        let height = 40;
        let mut map = Self {
            grid: vec![vec![Tile::Wall; width]; height],
            zones: vec![vec![Zone::Zone1; width]; height],
            width,
            height,
            start: Position { x: 2, y: 2 },
            exit: Position { x: 76, y: 36 },
            enemy_spawns: vec![],
        };

        map.assign_zones();
        map.carve_level();
        map.set_tile(map.exit.x, map.exit.y, Tile::Exit);

        map
    }

    pub fn level(level_num: usize) -> Self {
        match level_num {
            2 => Self::build_level_2(),
            3 => Self::build_level_3(),
            _ => Self::new(),
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Tile {
        if x >= self.width || y >= self.height {
            Tile::Wall
        } else {
            self.grid[y][x]
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            self.grid[y][x] = tile;
        }
    }

    pub fn is_passable(&self, x: usize, y: usize) -> bool {
        matches!(self.get_tile(x, y), Tile::Floor | Tile::Exit)
    }

    pub fn zone_at(&self, pos: Position) -> Zone {
        self.zones[pos.y.min(self.height - 1)][pos.x.min(self.width - 1)]
    }

    fn assign_zones(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.zones[y][x] = match x {
                    0..=15 => Zone::Zone1,
                    16..=31 => Zone::Zone2,
                    32..=47 => Zone::Zone3,
                    48..=63 => Zone::Zone4,
                    _ => Zone::Zone5,
                };
            }
        }
    }

    fn carve_level(&mut self) {
        self.carve_horizontal(2, 2, 12);
        self.carve_vertical(12, 2, 8);
        self.carve_horizontal(8, 5, 12);
        self.carve_vertical(5, 8, 16);
        self.carve_horizontal(16, 5, 14);
        self.carve_vertical(14, 16, 20);

        self.carve_horizontal(20, 14, 16);
        self.carve_horizontal(20, 19, 21);
        self.carve_horizontal(20, 24, 26);
        self.carve_horizontal(20, 29, 31);
        self.carve_horizontal(22, 19, 21);
        self.carve_horizontal(22, 24, 26);
        self.carve_horizontal(22, 29, 31);
        self.carve_vertical(31, 20, 25);

        self.carve_horizontal(25, 31, 33);
        self.carve_horizontal(25, 45, 47);
        self.carve_vertical(47, 25, 30);
        self.carve_horizontal(30, 32, 34);
        self.carve_horizontal(30, 45, 47);
        self.carve_vertical(34, 30, 34);
        self.carve_horizontal(34, 32, 34);
        self.carve_horizontal(34, 45, 49);

        self.carve_horizontal(34, 55, 56);
        self.carve_horizontal(34, 61, 62);
        self.carve_vertical(62, 34, 35);
        self.carve_horizontal(35, 49, 62);
        self.carve_vertical(49, 35, 36);
        self.carve_horizontal(36, 49, 58);
        self.carve_horizontal(36, 60, 68);

        self.set_tile(59, 36, Tile::Wall);
        self.set_tile(69, 36, Tile::Obstacle);
        self.set_tile(73, 36, Tile::Obstacle);
        self.carve_horizontal(36, 70, 72);
        self.carve_horizontal(36, 74, 76);

        self.carve_vertical(68, 35, 37);
        self.carve_horizontal(37, 64, 68);
        self.set_tile(66, 37, Tile::Obstacle);
        self.set_tile(67, 37, Tile::Obstacle);

        self.carve_vertical(76, 34, 36);
        self.carve_horizontal(34, 72, 76);
        self.set_tile(74, 34, Tile::Obstacle);

        self.set_tile(self.start.x, self.start.y, Tile::Floor);
    }

    fn build_level_2() -> Self {
        let width = 80;
        let height = 40;
        let mut map = Self {
            grid: vec![vec![Tile::Wall; width]; height],
            zones: vec![vec![Zone::Zone1; width]; height],
            width,
            height,
            start: Position { x: 2, y: 37 },
            exit: Position { x: 76, y: 2 },
            enemy_spawns: vec![],
        };

        map.assign_zones();
        map.carve_level_2();
        map.set_tile(map.start.x, map.start.y, Tile::Floor);
        map.set_tile(map.exit.x, map.exit.y, Tile::Exit);

        map
    }

    fn carve_level_2(&mut self) {
        self.carve_vertical(2, 32, 37);
        self.carve_horizontal(35, 2, 6);
        self.carve_vertical(6, 32, 35);
        self.carve_horizontal(32, 2, 9);
        self.carve_vertical(9, 25, 32);
        self.carve_horizontal(28, 9, 13);
        self.carve_vertical(13, 25, 28);
        self.carve_horizontal(25, 5, 15);
        self.carve_vertical(5, 18, 25);
        self.carve_horizontal(22, 5, 9);
        self.carve_vertical(8, 18, 22);
        self.carve_horizontal(18, 5, 12);
        self.carve_vertical(12, 12, 18);
        self.carve_horizontal(12, 8, 16);

        self.carve_horizontal(12, 16, 20);
        self.carve_horizontal(12, 20, 22);
        self.carve_horizontal(12, 22, 26);
        self.carve_horizontal(12, 28, 31);
        self.carve_vertical(20, 12, 16);
        self.carve_horizontal(16, 17, 20);
        self.carve_vertical(17, 16, 21);
        self.carve_horizontal(21, 17, 23);
        self.carve_vertical(22, 12, 21);
        self.carve_vertical(23, 12, 25);
        self.carve_horizontal(25, 20, 29);
        self.carve_vertical(27, 25, 29);
        self.carve_vertical(29, 12, 29);
        self.carve_horizontal(29, 24, 31);

        self.set_tile(19, 16, Tile::Obstacle);
        self.set_tile(23, 14, Tile::Obstacle);
        self.set_tile(28, 25, Tile::Obstacle);

        self.carve_horizontal(29, 31, 47);
        self.carve_vertical(32, 18, 29);
        self.carve_vertical(47, 24, 29);
        self.carve_horizontal(24, 34, 47);
        self.carve_vertical(34, 18, 24);
        self.carve_vertical(39, 18, 24);
        self.carve_vertical(41, 18, 24);
        self.carve_horizontal(18, 32, 44);
        self.carve_vertical(44, 12, 18);
        self.carve_horizontal(12, 36, 47);

        self.set_tile(40, 18, Tile::Obstacle);
        self.set_tile(38, 24, Tile::Obstacle);
        self.set_tile(45, 29, Tile::Obstacle);

        self.carve_horizontal(12, 48, 63);
        self.carve_vertical(50, 12, 20);
        self.carve_horizontal(20, 50, 58);
        self.carve_vertical(58, 16, 24);
        self.carve_horizontal(16, 54, 63);
        self.carve_vertical(63, 8, 16);
        self.carve_horizontal(8, 57, 63);
        self.carve_vertical(57, 8, 14);
        self.carve_horizontal(14, 52, 57);
        self.carve_vertical(52, 14, 22);
        self.carve_horizontal(22, 52, 60);
        self.carve_vertical(60, 2, 8);
        self.carve_horizontal(2, 60, 63);

        self.set_tile(55, 12, Tile::Obstacle);
        self.set_tile(54, 20, Tile::Obstacle);
        self.set_tile(53, 14, Tile::Obstacle);

        self.carve_horizontal(2, 60, 67);
        self.carve_vertical(67, 2, 4);
        self.carve_horizontal(4, 64, 76);
        self.carve_vertical(69, 2, 4);
        self.carve_vertical(72, 2, 4);
        self.carve_vertical(74, 2, 4);
        self.carve_horizontal(2, 69, 76);
        self.carve_vertical(76, 2, 4);

        self.set_tile(68, 2, Tile::Obstacle);
        self.set_tile(73, 2, Tile::Obstacle);
        self.set_tile(70, 4, Tile::Obstacle);
        self.set_tile(75, 4, Tile::Obstacle);
    }

    fn build_level_3() -> Self {
        let width = 80;
        let height = 40;
        let mut map = Self {
            grid: vec![vec![Tile::Wall; width]; height],
            zones: vec![vec![Zone::Zone1; width]; height],
            width,
            height,
            start: Position { x: 2, y: 2 },
            exit: Position { x: 77, y: 37 },
            enemy_spawns: vec![
                Position { x: 38, y: 16 },
                Position { x: 54, y: 22 },
                Position { x: 48, y: 18 },
                Position { x: 70, y: 28 },
                Position { x: 76, y: 34 },
            ],
        };

        map.assign_zones();
        map.carve_level_3();
        map.set_tile(map.start.x, map.start.y, Tile::Floor);
        map.set_tile(map.exit.x, map.exit.y, Tile::Exit);

        map
    }

    fn carve_level_3(&mut self) {
        self.carve_horizontal(2, 2, 14);
        self.carve_vertical(2, 2, 6);
        self.carve_horizontal(6, 2, 10);
        self.carve_vertical(10, 4, 8);
        self.carve_horizontal(4, 4, 10);
        self.carve_horizontal(8, 10, 14);

        self.carve_vertical(14, 2, 10);

        self.carve_horizontal(10, 14, 28);
        self.carve_vertical(20, 6, 10);
        self.carve_horizontal(6, 14, 20);
        self.carve_horizontal(14, 20, 28);

        self.carve_vertical(28, 10, 16);

        self.carve_horizontal(16, 28, 44);
        self.carve_vertical(36, 12, 16);
        self.carve_horizontal(12, 32, 36);

        self.carve_vertical(44, 16, 22);

        self.carve_horizontal(22, 44, 60);
        self.carve_vertical(52, 18, 22);
        self.carve_horizontal(18, 48, 52);
        self.carve_vertical(55, 20, 22);
        self.carve_horizontal(20, 55, 57);
        self.carve_vertical(57, 20, 22);

        self.carve_vertical(60, 22, 28);

        self.carve_horizontal(28, 60, 76);
        self.carve_vertical(68, 24, 28);
        self.carve_horizontal(24, 64, 68);
        self.carve_vertical(71, 26, 28);
        self.carve_horizontal(26, 71, 73);
        self.carve_vertical(73, 26, 28);

        self.carve_vertical(76, 28, 37);
        self.carve_horizontal(37, 70, 78);
        self.carve_vertical(73, 35, 37);
        self.carve_horizontal(35, 73, 75);
        self.carve_vertical(75, 35, 37);

        self.set_tile(56, 22, Tile::Obstacle);
        self.set_tile(72, 28, Tile::Obstacle);
        self.set_tile(74, 37, Tile::Obstacle);
        self.set_tile(50, 18, Tile::Obstacle);
        self.set_tile(66, 24, Tile::Obstacle);
        self.set_tile(34, 12, Tile::Obstacle);
    }

    fn carve_horizontal(&mut self, y: usize, start_x: usize, end_x: usize) {
        for x in start_x.min(end_x)..=start_x.max(end_x) {
            self.set_tile(x, y, Tile::Floor);
        }
    }

    fn carve_vertical(&mut self, x: usize, start_y: usize, end_y: usize) {
        for y in start_y.min(end_y)..=start_y.max(end_y) {
            self.set_tile(x, y, Tile::Floor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let carved_tiles = map
            .grid
            .iter()
            .flatten()
            .filter(|tile| **tile != Tile::Wall)
            .count();

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
        let carved = map
            .grid
            .iter()
            .flatten()
            .filter(|t| **t != Tile::Wall)
            .count();

        assert!(carved > 50);
    }

    #[test]
    fn map_level_2_has_more_obstacles_than_level_1() {
        let level1 = Map::level(1);
        let level2 = Map::level(2);
        let obs1 = level1
            .grid
            .iter()
            .flatten()
            .filter(|t| **t == Tile::Obstacle)
            .count();
        let obs2 = level2
            .grid
            .iter()
            .flatten()
            .filter(|t| **t == Tile::Obstacle)
            .count();

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

                    if nx < map.width
                        && ny < map.height
                        && !visited[ny][nx]
                        && map.is_passable(nx, ny)
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

                    if nx < map.width
                        && ny < map.height
                        && !visited[ny][nx]
                        && map.is_passable(nx, ny)
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
}
