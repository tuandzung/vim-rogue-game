use std::collections::HashSet;

use crate::map::Map;
use crate::types::{Direction, Position, Tile, VimMotion};

pub struct Player {
    pub position: Position,
    pub used_motions: HashSet<VimMotion>,
    pub last_direction: Option<Direction>,
}

impl Player {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            used_motions: HashSet::new(),
            last_direction: None,
        }
    }

    pub fn handle_motion(
        &mut self,
        motion: VimMotion,
        target: Option<char>,
        map: &mut Map,
    ) -> bool {
        self.used_motions.insert(motion);
        let old_pos = self.position;

        let activated = match motion {
            VimMotion::H => self.step(Direction::Left, map),
            VimMotion::J => self.step(Direction::Down, map),
            VimMotion::K => self.step(Direction::Up, map),
            VimMotion::L => self.step(Direction::Right, map),
            VimMotion::W => self.jump_word_forward(map),
            VimMotion::B => self.jump_word_backward(map),
            VimMotion::Zero => self.jump_row_start(map),
            VimMotion::Dollar => self.jump_row_end(map),
            VimMotion::Find => target.is_some_and(|ch| self.find_char(ch, map)),
            VimMotion::Till => target.is_some_and(|ch| self.till_char(ch, map)),
            VimMotion::DeleteLine => self.delete_obstacle_on_row(map),
            VimMotion::G => self.jump_to_column_bottom(map),
            VimMotion::GotoLine => self.jump_to_column_top(map),
        };

        // Update facing based on net displacement
        if activated && self.position != old_pos {
            let dx = self.position.x as isize - old_pos.x as isize;
            let dy = self.position.y as isize - old_pos.y as isize;
            if dx > 0 {
                self.last_direction = Some(Direction::Right);
            } else if dx < 0 {
                self.last_direction = Some(Direction::Left);
            } else if dy > 0 {
                self.last_direction = Some(Direction::Down);
            } else if dy < 0 {
                self.last_direction = Some(Direction::Up);
            }
        }

        activated
    }

    fn step(&mut self, direction: Direction, map: &Map) -> bool {
        let (dx, dy) = direction.delta();
        let next_x = self.position.x as isize + dx;
        let next_y = self.position.y as isize + dy;
        if next_x < 0 || next_y < 0 {
            return false;
        }

        let next = Position {
            x: next_x as usize,
            y: next_y as usize,
        };

        if map.is_passable(next.x, next.y) {
            self.position = next;
            true
        } else {
            false
        }
    }

    fn jump_word_forward(&mut self, map: &Map) -> bool {
        let y = self.position.y;
        let current = map.get_tile(self.position.x, y);
        let mut last_valid_x = self.position.x;

        for x in (self.position.x + 1)..map.width {
            if !map.is_passable(x, y) {
                break;
            }
            let tile = map.get_tile(x, y);
            if tile != current {
                self.position.x = x;
                return true;
            }
            last_valid_x = x;
        }

        let changed = self.position.x != last_valid_x;
        if changed {
            self.position.x = last_valid_x;
        }
        changed
    }

    fn jump_word_backward(&mut self, map: &Map) -> bool {
        if self.position.x == 0 {
            return false;
        }

        let y = self.position.y;
        let current = map.get_tile(self.position.x, y);
        let mut last_valid_x = self.position.x;

        for x in (0..self.position.x).rev() {
            if !map.is_passable(x, y) {
                break;
            }
            let tile = map.get_tile(x, y);
            if tile != current {
                self.position.x = x;
                return true;
            }
            last_valid_x = x;
        }

        let changed = self.position.x != last_valid_x;
        if changed {
            self.position.x = last_valid_x;
        }
        changed
    }

    fn jump_row_start(&mut self, map: &Map) -> bool {
        for x in 0..map.width {
            if map.is_passable(x, self.position.y) {
                let changed = self.position.x != x;
                self.position.x = x;
                return changed;
            }
        }

        false
    }

    fn jump_row_end(&mut self, map: &Map) -> bool {
        for x in (0..map.width).rev() {
            if map.is_passable(x, self.position.y) {
                let changed = self.position.x != x;
                self.position.x = x;
                return changed;
            }
        }

        false
    }

    fn jump_to_column_bottom(&mut self, map: &Map) -> bool {
        let x = self.position.x;
        let mut last_valid_y = self.position.y;

        for y in (self.position.y + 1)..map.height {
            if !map.is_passable(x, y) {
                break;
            }
            last_valid_y = y;
        }

        let changed = self.position.y != last_valid_y;
        if changed {
            self.position.y = last_valid_y;
        }
        changed
    }

    fn jump_to_column_top(&mut self, map: &Map) -> bool {
        if self.position.y == 0 {
            return false;
        }

        let x = self.position.x;
        let mut last_valid_y = self.position.y;

        for y in (0..self.position.y).rev() {
            if !map.is_passable(x, y) {
                break;
            }
            last_valid_y = y;
        }

        let changed = self.position.y != last_valid_y;
        if changed {
            self.position.y = last_valid_y;
        }
        changed
    }

    fn find_char(&mut self, target: char, map: &Map) -> bool {
        let y = self.position.y;
        for x in (self.position.x + 1)..map.width {
            if map.get_tile(x, y).glyph() == target && map.is_passable(x, y) {
                self.position.x = x;
                return true;
            }
        }

        false
    }

    fn till_char(&mut self, target: char, map: &Map) -> bool {
        let y = self.position.y;
        for x in (self.position.x + 1)..map.width {
            if map.get_tile(x, y).glyph() == target {
                if x == 0 {
                    return false;
                }
                let stop_x = x - 1;
                if map.is_passable(stop_x, y) {
                    self.position.x = stop_x;
                    return true;
                }
                return false;
            }
        }

        false
    }

    fn delete_obstacle_on_row(&mut self, map: &mut Map) -> bool {
        let y = self.position.y;
        for x in self.position.x..map.width {
            if map.get_tile(x, y) == Tile::Obstacle {
                map.set_tile(x, y, Tile::Floor);
                return true;
            }
        }

        for x in (0..self.position.x).rev() {
            if map.get_tile(x, y) == Tile::Obstacle {
                map.set_tile(x, y, Tile::Floor);
                return true;
            }
        }

        false
    }
}
