use std::collections::HashSet;

use crate::map::Map;
use crate::types::{Direction, Position, Tile, VimMotion};

pub struct Player {
    pub position: Position,
    pub used_motions: HashSet<VimMotion>,
}

impl Player {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            used_motions: HashSet::new(),
        }
    }

    pub fn handle_motion(
        &mut self,
        motion: VimMotion,
        target: Option<char>,
        map: &mut Map,
    ) -> bool {
        self.used_motions.insert(motion);

        match motion {
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
            VimMotion::G => self.jump_to_last_row(map),
            VimMotion::GotoLine => self.jump_to_first_row(map),
        }
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
        let mut seen_separator = false;

        for x in (self.position.x + 1)..map.width {
            let tile = map.get_tile(x, y);
            if !map.is_passable(x, y) {
                seen_separator = true;
                continue;
            }

            let previous = if x > 0 {
                map.get_tile(x - 1, y)
            } else {
                Tile::Wall
            };
            let starts_segment = !matches!(previous, Tile::Floor | Tile::Exit);
            let different_kind = tile != current;
            if seen_separator || starts_segment || different_kind {
                self.position.x = x;
                return true;
            }
        }

        false
    }

    fn jump_word_backward(&mut self, map: &Map) -> bool {
        if self.position.x == 0 {
            return false;
        }

        let y = self.position.y;
        let current = map.get_tile(self.position.x, y);
        let mut seen_separator = false;

        for x in (0..self.position.x).rev() {
            let tile = map.get_tile(x, y);
            if !map.is_passable(x, y) {
                seen_separator = true;
                continue;
            }

            let next = map.get_tile(x + 1, y);
            let starts_segment = !matches!(next, Tile::Floor | Tile::Exit);
            let different_kind = tile != current;
            if seen_separator || starts_segment || different_kind {
                self.position.x = x;
                return true;
            }
        }

        false
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

    fn jump_to_last_row(&mut self, map: &Map) -> bool {
        for y in (0..map.height).rev() {
            for x in 0..map.width {
                if map.is_passable(x, y) {
                    let changed = self.position != Position { x, y };
                    self.position = Position { x, y };
                    return changed;
                }
            }
        }

        false
    }

    fn jump_to_first_row(&mut self, map: &Map) -> bool {
        for y in 0..map.height {
            for x in 0..map.width {
                if map.is_passable(x, y) {
                    let changed = self.position != Position { x, y };
                    self.position = Position { x, y };
                    return changed;
                }
            }
        }

        false
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Zone;

    fn test_map(width: usize, height: usize) -> Map {
        Map {
            grid: vec![vec![Tile::Floor; width]; height],
            zones: vec![vec![Zone::Zone1; width]; height],
            width,
            height,
            start: Position { x: 0, y: 0 },
            exit: Position {
                x: width - 1,
                y: height - 1,
            },
            enemy_spawns: vec![],
        }
    }

    #[test]
    fn player_new_has_starting_position() {
        let player = Player::new(Position { x: 2, y: 3 });

        assert_eq!(player.position, Position { x: 2, y: 3 });
    }

    #[test]
    fn player_new_has_empty_used_motions() {
        let player = Player::new(Position { x: 2, y: 3 });

        assert!(player.used_motions.is_empty());
    }

    #[test]
    fn player_step_right() {
        let mut map = test_map(5, 5);
        let mut player = Player::new(Position { x: 1, y: 1 });

        assert!(player.handle_motion(VimMotion::L, None, &mut map));
        assert_eq!(player.position, Position { x: 2, y: 1 });
    }

    #[test]
    fn player_step_left() {
        let mut map = test_map(5, 5);
        let mut player = Player::new(Position { x: 2, y: 1 });

        assert!(player.handle_motion(VimMotion::H, None, &mut map));
        assert_eq!(player.position, Position { x: 1, y: 1 });
    }

    #[test]
    fn player_step_down() {
        let mut map = test_map(5, 5);
        let mut player = Player::new(Position { x: 1, y: 1 });

        assert!(player.handle_motion(VimMotion::J, None, &mut map));
        assert_eq!(player.position, Position { x: 1, y: 2 });
    }

    #[test]
    fn player_step_up() {
        let mut map = test_map(5, 5);
        let mut player = Player::new(Position { x: 1, y: 2 });

        assert!(player.handle_motion(VimMotion::K, None, &mut map));
        assert_eq!(player.position, Position { x: 1, y: 1 });
    }

    #[test]
    fn player_step_into_wall_fails() {
        let mut map = test_map(5, 5);
        map.set_tile(2, 1, Tile::Wall);
        let mut player = Player::new(Position { x: 1, y: 1 });

        assert!(!player.handle_motion(VimMotion::L, None, &mut map));
        assert_eq!(player.position, Position { x: 1, y: 1 });
    }

    #[test]
    fn player_step_at_boundary_fails() {
        let mut map = test_map(5, 5);
        let mut player = Player::new(Position { x: 0, y: 1 });

        assert!(!player.handle_motion(VimMotion::H, None, &mut map));
        assert_eq!(player.position, Position { x: 0, y: 1 });
    }

    #[test]
    fn player_step_records_motion() {
        let mut map = test_map(5, 5);
        let mut player = Player::new(Position { x: 1, y: 1 });

        player.handle_motion(VimMotion::L, None, &mut map);

        assert!(player.used_motions.contains(&VimMotion::L));
    }

    #[test]
    fn player_w_jumps_forward() {
        let mut map = test_map(6, 1);
        map.set_tile(2, 0, Tile::Wall);
        let mut player = Player::new(Position { x: 1, y: 0 });

        assert!(player.handle_motion(VimMotion::W, None, &mut map));
        assert_eq!(player.position, Position { x: 3, y: 0 });
    }

    #[test]
    fn player_b_jumps_backward() {
        let mut map = test_map(6, 1);
        map.set_tile(2, 0, Tile::Wall);
        let mut player = Player::new(Position { x: 4, y: 0 });

        assert!(player.handle_motion(VimMotion::B, None, &mut map));
        assert_eq!(player.position, Position { x: 1, y: 0 });
    }

    #[test]
    fn player_zero_jumps_to_row_start() {
        let mut map = test_map(6, 1);
        map.set_tile(0, 0, Tile::Wall);
        map.set_tile(1, 0, Tile::Wall);
        let mut player = Player::new(Position { x: 4, y: 0 });

        assert!(player.handle_motion(VimMotion::Zero, None, &mut map));
        assert_eq!(player.position, Position { x: 2, y: 0 });
    }

    #[test]
    fn player_dollar_jumps_to_row_end() {
        let mut map = test_map(6, 1);
        map.set_tile(5, 0, Tile::Wall);
        let mut player = Player::new(Position { x: 1, y: 0 });

        assert!(player.handle_motion(VimMotion::Dollar, None, &mut map));
        assert_eq!(player.position, Position { x: 4, y: 0 });
    }

    #[test]
    fn player_find_char_finds_target() {
        let mut map = test_map(6, 1);
        map.set_tile(4, 0, Tile::Exit);
        let mut player = Player::new(Position { x: 1, y: 0 });

        assert!(player.handle_motion(VimMotion::Find, Some('>'), &mut map));
        assert_eq!(player.position, Position { x: 4, y: 0 });
    }

    #[test]
    fn player_find_char_not_found_fails() {
        let mut map = test_map(6, 1);
        let mut player = Player::new(Position { x: 1, y: 0 });

        assert!(!player.handle_motion(VimMotion::Find, Some('>'), &mut map));
        assert_eq!(player.position, Position { x: 1, y: 0 });
    }

    #[test]
    fn player_till_char_stops_before_target() {
        let mut map = test_map(6, 1);
        map.set_tile(4, 0, Tile::Exit);
        let mut player = Player::new(Position { x: 1, y: 0 });

        assert!(player.handle_motion(VimMotion::Till, Some('>'), &mut map));
        assert_eq!(player.position, Position { x: 3, y: 0 });
    }

    #[test]
    fn player_dd_destroys_obstacle() {
        let mut map = test_map(6, 1);
        map.set_tile(3, 0, Tile::Obstacle);
        let mut player = Player::new(Position { x: 1, y: 0 });

        assert!(player.handle_motion(VimMotion::DeleteLine, None, &mut map));
        assert_eq!(map.get_tile(3, 0), Tile::Floor);
    }

    #[test]
    fn player_dd_no_obstacle_fails() {
        let mut map = test_map(6, 1);
        let mut player = Player::new(Position { x: 1, y: 0 });

        assert!(!player.handle_motion(VimMotion::DeleteLine, None, &mut map));
    }

    #[test]
    fn player_g_jumps_to_last_passable_row() {
        let mut map = test_map(5, 5);
        map.set_tile(0, 4, Tile::Wall);
        map.set_tile(1, 4, Tile::Wall);
        let mut player = Player::new(Position { x: 2, y: 0 });

        assert!(player.handle_motion(VimMotion::G, None, &mut map));
        assert_eq!(player.position, Position { x: 2, y: 4 });
    }

    #[test]
    fn player_gg_jumps_to_first_passable_row() {
        let mut map = test_map(5, 5);
        map.set_tile(0, 0, Tile::Wall);
        map.set_tile(1, 0, Tile::Wall);
        let mut player = Player::new(Position { x: 2, y: 4 });

        assert!(player.handle_motion(VimMotion::GotoLine, None, &mut map));
        assert_eq!(player.position, Position { x: 2, y: 0 });
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
        };
        map.set_tile(2, 2, Tile::Floor);
        let mut player = Player::new(Position { x: 2, y: 2 });

        assert!(!player.handle_motion(VimMotion::G, None, &mut map));
        assert_eq!(player.position, Position { x: 2, y: 2 });
    }

    #[test]
    fn player_gg_no_passable_rows_fails() {
        let mut map = Map {
            grid: vec![vec![Tile::Wall; 5]; 5],
            zones: vec![vec![Zone::Zone1; 5]; 5],
            width: 5,
            height: 5,
            start: Position { x: 0, y: 0 },
            exit: Position { x: 4, y: 4 },
            enemy_spawns: vec![],
        };
        map.set_tile(2, 2, Tile::Floor);
        let mut player = Player::new(Position { x: 2, y: 2 });

        assert!(!player.handle_motion(VimMotion::GotoLine, None, &mut map));
        assert_eq!(player.position, Position { x: 2, y: 2 });
    }

    #[test]
    fn player_g_already_on_target_row_returns_false() {
        let mut map = test_map(5, 5);
        let mut player = Player::new(Position { x: 0, y: 4 });

        assert!(!player.handle_motion(VimMotion::G, None, &mut map));
        assert_eq!(player.position, Position { x: 0, y: 4 });
    }

    #[test]
    fn player_gg_already_on_target_row_returns_false() {
        let mut map = test_map(5, 5);
        let mut player = Player::new(Position { x: 0, y: 0 });

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
            let mut player = Player::new(Position { x: 4, y: 1 });
            let target = match motion {
                VimMotion::Find | VimMotion::Till => Some('>'),
                _ => None,
            };

            player.handle_motion(motion, target, &mut map);

            assert!(player.used_motions.contains(&motion));
        }
    }
}
