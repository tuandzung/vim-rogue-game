use std::collections::VecDeque;

use crate::map::Map;
use crate::types::{ENEMY_FOV_RADIUS, Enemy, PatrolArea, Position, Tile};

impl Enemy {
    pub fn new(pos: Position) -> Self {
        Self {
            position: pos,
            glyph: 'e',
            hp: None,
            stunned_turns: 0,
            patrol_area: PatrolArea::point(pos.x, pos.y),
        }
    }

    pub fn step_toward_player(&mut self, player_pos: Position, map: &Map) -> bool {
        if self.position == player_pos {
            return false;
        }

        let directions: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        for (dx, dy) in &directions {
            let nx = self.position.x as isize + dx;
            let ny = self.position.y as isize + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if nx >= map.width || ny >= map.height {
                continue;
            }
            if (Position { x: nx, y: ny }) == player_pos {
                self.position = player_pos;
                return true;
            }
        }

        let mut visited = vec![vec![false; map.width]; map.height];
        let mut parent: Vec<Vec<Option<Position>>> = vec![vec![None; map.width]; map.height];
        let mut queue = VecDeque::new();

        visited[self.position.y][self.position.x] = true;
        queue.push_back(self.position);

        while let Some(pos) = queue.pop_front() {
            if pos == player_pos {
                break;
            }

            for (dx, dy) in &directions {
                let nx = pos.x as isize + dx;
                let ny = pos.y as isize + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if nx >= map.width || ny >= map.height {
                    continue;
                }
                if visited[ny][nx] {
                    continue;
                }
                let neighbor = Position { x: nx, y: ny };
                if !map.is_passable(nx, ny) && neighbor != player_pos {
                    continue;
                }

                visited[ny][nx] = true;
                parent[ny][nx] = Some(pos);
                queue.push_back(neighbor);
            }
        }

        if !visited[player_pos.y][player_pos.x] {
            return false;
        }

        let mut step = player_pos;
        while let Some(prev) = parent[step.y][step.x] {
            if prev == self.position {
                self.position = step;
                return true;
            }
            step = prev;
        }

        false
    }

    /// Check if this enemy has line-of-sight to a target position using Bresenham's line algorithm.
    /// Returns false if target is beyond ENEMY_FOV_RADIUS or if a Wall tile blocks the line.
    pub fn has_line_of_sight(&self, target: Position, map: &Map) -> bool {
        let dx = target.x as i32 - self.position.x as i32;
        let dy = target.y as i32 - self.position.y as i32;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq > ENEMY_FOV_RADIUS * ENEMY_FOV_RADIUS {
            return false;
        }

        if self.position == target {
            return true;
        }

        // Bresenham's line algorithm
        let adx = dx.abs();
        let ady = dy.abs();
        let sx: i32 = if dx > 0 { 1 } else { -1 };
        let sy: i32 = if dy > 0 { 1 } else { -1 };

        let mut err = adx - ady;
        let mut x = self.position.x as i32;
        let mut y = self.position.y as i32;
        let tx = target.x as i32;
        let ty = target.y as i32;

        loop {
            let e2 = 2 * err;
            if e2 > -ady {
                err -= ady;
                x += sx;
            }
            if e2 < adx {
                err += adx;
                y += sy;
            }

            if x == tx && y == ty {
                return true;
            }

            if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
                return false;
            }

            if map.get_tile(x as usize, y as usize) == Tile::Wall {
                return false;
            }
        }
    }

    /// Move randomly within the enemy's patrol area.
    /// Uses a deterministic direction order based on position to avoid pure randomness.
    /// Returns true if the enemy moved, false if stuck.
    pub fn patrol_step(&mut self, map: &Map) -> bool {
        let directions: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let base_idx = (self.position.x * 7 + self.position.y * 13) % 4;

        for i in 0..4 {
            let (dx, dy) = directions[(base_idx + i) % 4];
            let nx = self.position.x as isize + dx;
            let ny = self.position.y as isize + dy;

            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;

            if nx >= map.width || ny >= map.height {
                continue;
            }
            if !self.patrol_area.contains(nx, ny) {
                continue;
            }
            if !map.is_passable(nx, ny) {
                continue;
            }

            self.position = Position { x: nx, y: ny };
            return true;
        }

        false
    }
}
