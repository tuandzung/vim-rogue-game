use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};

use crate::map::Map;
use crate::player::Player;

pub const TRAIL_MAX: usize = 8;
pub const TOTAL_LEVELS: usize = 2;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tile {
    Wall,
    Floor,
    Exit,
    Obstacle,
}

impl Tile {
    pub fn glyph(self) -> char {
        match self {
            Self::Wall => '#',
            Self::Floor => '.',
            Self::Exit => '>',
            Self::Obstacle => '▒',
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.glyph())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Zone {
    Zone1,
    Zone2,
    Zone3,
    Zone4,
    Zone5,
}

impl Zone {
    pub fn title(self) -> &'static str {
        match self {
            Self::Zone1 => "Zone 1 — Basic Steps",
            Self::Zone2 => "Zone 2 — Word Leaps",
            Self::Zone3 => "Zone 3 — Line Ends",
            Self::Zone4 => "Zone 4 — Target Search",
            Self::Zone5 => "Zone 5 — Delete Gates",
        }
    }
}

impl Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VimMotion {
    H,
    J,
    K,
    L,
    W,
    B,
    Zero,
    Dollar,
    Find,
    Till,
    DeleteLine,
}

impl VimMotion {
    pub fn key_label(self) -> &'static str {
        match self {
            Self::H => "h",
            Self::J => "j",
            Self::K => "k",
            Self::L => "l",
            Self::W => "w",
            Self::B => "b",
            Self::Zero => "0",
            Self::Dollar => "$",
            Self::Find => "f<char>",
            Self::Till => "t<char>",
            Self::DeleteLine => "dd",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::H => "Left",
            Self::J => "Down",
            Self::K => "Up",
            Self::L => "Right",
            Self::W => "Next word",
            Self::B => "Back word",
            Self::Zero => "Line start",
            Self::Dollar => "Line end",
            Self::Find => "Find char",
            Self::Till => "Till char",
            Self::DeleteLine => "Delete row obstacle",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::H => "Move one tile left",
            Self::J => "Move one tile down",
            Self::K => "Move one tile up",
            Self::L => "Move one tile right",
            Self::W => "Jump to the next passable segment",
            Self::B => "Jump to the previous passable segment",
            Self::Zero => "Jump to the first passable tile on the row",
            Self::Dollar => "Jump to the last passable tile on the row",
            Self::Find => "Jump to the next matching tile character",
            Self::Till => "Stop one tile before the next matching character",
            Self::DeleteLine => "Turn the nearest obstacle on the row into floor",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

impl Direction {
    pub fn delta(self) -> (isize, isize) {
        match self {
            Self::Left => (-1, 0),
            Self::Down => (0, 1),
            Self::Up => (0, -1),
            Self::Right => (1, 0),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Won,
    Quit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PendingInput {
    Find,
    Till,
    Delete,
}

pub struct App {
    pub map: Map,
    pub player: Player,
    pub game_state: GameState,
    pub started: bool,
    pub pending_input: Option<PendingInput>,
    pub start_time: Instant,
    pub elapsed: Duration,
    pub final_time: Option<Duration>,
    pub motion_count: usize,
    pub status_message: String,
    pub discovered_motions: HashSet<VimMotion>,
    pub trail: VecDeque<Position>,
    pub level: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_glyph_wall() {
        assert_eq!(Tile::Wall.glyph(), '#');
    }

    #[test]
    fn tile_glyph_floor() {
        assert_eq!(Tile::Floor.glyph(), '.');
    }

    #[test]
    fn tile_glyph_exit() {
        assert_eq!(Tile::Exit.glyph(), '>');
    }

    #[test]
    fn tile_glyph_obstacle() {
        assert_eq!(Tile::Obstacle.glyph(), '▒');
    }

    #[test]
    fn vim_motion_key_labels() {
        let cases = [
            (VimMotion::H, "h"),
            (VimMotion::J, "j"),
            (VimMotion::K, "k"),
            (VimMotion::L, "l"),
            (VimMotion::W, "w"),
            (VimMotion::B, "b"),
            (VimMotion::Zero, "0"),
            (VimMotion::Dollar, "$"),
            (VimMotion::Find, "f<char>"),
            (VimMotion::Till, "t<char>"),
            (VimMotion::DeleteLine, "dd"),
        ];

        for (motion, expected) in cases {
            assert_eq!(motion.key_label(), expected);
        }
    }

    #[test]
    fn zone_titles_exist() {
        let zones = [
            Zone::Zone1,
            Zone::Zone2,
            Zone::Zone3,
            Zone::Zone4,
            Zone::Zone5,
        ];

        for zone in zones {
            assert!(!zone.title().trim().is_empty());
        }
    }

    #[test]
    fn direction_deltas() {
        assert_eq!(Direction::Left.delta(), (-1, 0));
        assert_eq!(Direction::Down.delta(), (0, 1));
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }

    #[test]
    fn position_is_copy() {
        let original = Position { x: 3, y: 7 };
        let _copied = original;
        assert_eq!(original.x, 3);
    }
}
