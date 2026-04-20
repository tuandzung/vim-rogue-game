use std::collections::VecDeque;

use crate::map::Map;
use crate::types::{Enemy, Position};

impl Enemy {
    pub fn new(pos: Position) -> Self {
        Self {
            position: pos,
            glyph: 'e',
            hp: None,
            stunned_turns: 0,
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
}
