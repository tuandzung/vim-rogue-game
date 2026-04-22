use crate::types::{PatrolArea, Position, Tile, Zone};

pub struct Map {
    pub grid: Vec<Vec<Tile>>,
    pub zones: Vec<Vec<Zone>>,
    pub width: usize,
    pub height: usize,
    pub start: Position,
    pub exit: Position,
    pub enemy_spawns: Vec<Position>,
    pub enemy_patrol_areas: Vec<PatrolArea>,
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
            enemy_patrol_areas: vec![],
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
            4 => Self::build_level_4(),
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
        matches!(self.get_tile(x, y), Tile::Floor | Tile::Exit | Tile::Torchlight)
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
        self.set_tile(17, 20, Tile::Obstacle);
        self.set_tile(18, 20, Tile::Obstacle);
        self.carve_horizontal(20, 19, 21);
        self.set_tile(22, 20, Tile::Obstacle);
        self.set_tile(23, 20, Tile::Obstacle);
        self.carve_horizontal(20, 24, 26);
        self.set_tile(27, 20, Tile::Obstacle);
        self.set_tile(28, 20, Tile::Obstacle);
        self.carve_horizontal(20, 29, 31);

        self.carve_horizontal(22, 19, 21);
        self.set_tile(22, 22, Tile::Obstacle);
        self.set_tile(23, 22, Tile::Obstacle);
        self.carve_horizontal(22, 24, 26);
        self.set_tile(27, 22, Tile::Obstacle);
        self.set_tile(28, 22, Tile::Obstacle);
        self.carve_horizontal(22, 29, 31);

        self.carve_vertical(31, 20, 28);

        self.carve_horizontal(28, 31, 47);
        self.set_tile(38, 28, Tile::Obstacle);
        self.set_tile(39, 28, Tile::Obstacle);

        self.carve_horizontal(25, 31, 33);
        self.carve_horizontal(25, 45, 47);
        self.carve_vertical(47, 25, 34);
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

        self.set_tile(59, 36, Tile::Obstacle);
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
            enemy_patrol_areas: vec![],
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
        self.set_tile(27, 12, Tile::Obstacle);
        self.carve_horizontal(12, 28, 31);
        self.carve_vertical(20, 12, 16);
        self.carve_horizontal(16, 17, 20);
        self.carve_vertical(17, 16, 21);
        self.carve_horizontal(21, 17, 23);
        self.carve_vertical(22, 12, 21);
        self.carve_vertical(23, 12, 25);
        self.carve_horizontal(25, 20, 29);
        self.set_tile(17, 25, Tile::Obstacle);
        self.set_tile(18, 25, Tile::Obstacle);
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
        self.set_tile(33, 12, Tile::Obstacle);
        self.set_tile(34, 12, Tile::Obstacle);
        self.set_tile(35, 12, Tile::Obstacle);

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
            enemy_patrol_areas: vec![],
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

        // Torchlight checkpoints at corridor junctions
        self.set_tile(28, 10, Tile::Torchlight);
        self.set_tile(60, 22, Tile::Torchlight);
    }

    fn build_level_4() -> Self {
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
                // Room 1 (no torchlight): 2 enemies
                Position { x: 7, y: 5 },
                Position { x: 12, y: 7 },
                // Room 2 (torchlight at 56,5): 2 enemies
                Position { x: 55, y: 5 },
                Position { x: 58, y: 7 },
                // Room 3 (no torchlight): 3 enemies
                Position { x: 33, y: 18 },
                Position { x: 38, y: 20 },
                Position { x: 36, y: 22 },
                // Room 5 (no torchlight): 2 enemies
                Position { x: 72, y: 34 },
                Position { x: 68, y: 32 },
            ],
            enemy_patrol_areas: vec![
                // Room 1 patrol areas
                PatrolArea { min_x: 4, min_y: 2, max_x: 15, max_y: 9 },
                PatrolArea { min_x: 4, min_y: 2, max_x: 15, max_y: 9 },
                // Room 2 patrol areas
                PatrolArea { min_x: 50, min_y: 2, max_x: 63, max_y: 9 },
                PatrolArea { min_x: 50, min_y: 2, max_x: 63, max_y: 9 },
                // Room 3 patrol areas
                PatrolArea { min_x: 30, min_y: 16, max_x: 43, max_y: 23 },
                PatrolArea { min_x: 30, min_y: 16, max_x: 43, max_y: 23 },
                PatrolArea { min_x: 30, min_y: 16, max_x: 43, max_y: 23 },
                // Room 5 patrol areas
                PatrolArea { min_x: 60, min_y: 30, max_x: 75, max_y: 37 },
                PatrolArea { min_x: 60, min_y: 30, max_x: 75, max_y: 37 },
            ],
        };

        map.assign_zones();
        map.carve_level_4();
        map.set_tile(map.start.x, map.start.y, Tile::Floor);
        map.set_tile(map.exit.x, map.exit.y, Tile::Exit);

        map
    }

    fn carve_level_4(&mut self) {
        // Room 1: top-left (x:4-15, y:2-9) — safe zone, player start
        for y in 2..=9 {
            self.carve_horizontal(y, 4, 15);
        }

        // Room 2: top-right (x:50-63, y:2-9)
        for y in 2..=9 {
            self.carve_horizontal(y, 50, 63);
        }

        // Room 3: center (x:30-43, y:16-23)
        for y in 16..=23 {
            self.carve_horizontal(y, 30, 43);
        }

        // Room 4: bottom-left (x:8-21, y:30-37)
        for y in 30..=37 {
            self.carve_horizontal(y, 8, 21);
        }

        // Room 5: bottom-right (x:60-75, y:30-37)
        for y in 30..=37 {
            self.carve_horizontal(y, 60, 75);
        }

        // Connect start (2,2) to Room 1
        self.carve_horizontal(2, 2, 4);

        // Corridor: Room 1 → Room 2 (horizontal at y=6, x:15-50)
        self.carve_horizontal(6, 15, 50);

        // Corridor: Room 1 → Room 3 (horizontal y=10, x:9-36; vertical x=36, y=10-16)
        self.carve_horizontal(10, 9, 36);
        self.carve_vertical(36, 10, 16);

        // Corridor: Room 1 → Room 4 (vertical x=10, y:9-30)
        self.carve_vertical(10, 9, 30);

        // Corridor: Room 3 → Room 5 (vertical x=36, y=23-26; horizontal y=26, x:36-65; vertical x=65, y=26-30)
        self.carve_vertical(36, 23, 26);
        self.carve_horizontal(26, 36, 65);
        self.carve_vertical(65, 26, 30);

        // Corridor: Room 2 → Room 5 (vertical x=62, y:9-30)
        self.carve_vertical(62, 9, 30);

        // Corridor: Room 4 → Room 5 (horizontal y=34, x:21-60)
        self.carve_horizontal(34, 21, 60);

        // Corridor: Room 3 → Room 2 (vertical x=42, y=13-16; horizontal y=13, x=42-52; vertical x=52, y=9-13)
        self.carve_vertical(42, 13, 16);
        self.carve_horizontal(13, 42, 52);
        self.carve_vertical(52, 9, 13);

        // Path from Room 5 to exit at (77, 37)
        self.carve_horizontal(37, 75, 77);

        // Torchlights at rest points
        self.set_tile(56, 5, Tile::Torchlight); // Room 2 checkpoint
        self.set_tile(14, 33, Tile::Torchlight); // Room 4 checkpoint
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
