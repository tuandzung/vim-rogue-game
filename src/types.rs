use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};

use bracket_lib::prelude::VirtualKeyCode;

use crate::animation::AnimationState;
use crate::audio::AudioManager;
use crate::map::Map;
use crate::player::Player;
use crate::visibility::VisibilityMap;

pub const TRAIL_MAX: usize = 8;
pub const TOTAL_LEVELS: usize = 3;
pub const FOV_RADIUS: i32 = 10;

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
    G,
    GotoLine,
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
            Self::G => "G",
            Self::GotoLine => "gg",
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
            Self::G => "Last row",
            Self::GotoLine => "First row",
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
            Self::G => "Jump to the last passable tile on the bottom row",
            Self::GotoLine => "Jump to the first passable tile on the top row",
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
    Lost,
    Quit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PendingInput {
    Find,
    Till,
    Delete,
    GotoLine,
}

/// A single cell in the render grid.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderCell {
    pub glyph: char,
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    pub blink: bool,
}

impl RenderCell {
    pub fn new(glyph: char, fg: (u8, u8, u8), bg: (u8, u8, u8)) -> Self {
        Self {
            glyph,
            fg,
            bg,
            blink: false,
        }
    }

    pub fn with_blink(mut self) -> Self {
        self.blink = true;
        self
    }
}

/// 2D grid of render cells — the "frame" to be drawn.
#[derive(Debug, Clone)]
pub struct RenderGrid {
    cells: Vec<Vec<RenderCell>>, // cells[y][x]
    width: usize,
    height: usize,
}

impl RenderGrid {
    pub fn new(width: usize, height: usize, default: RenderCell) -> Self {
        Self {
            cells: vec![vec![default; width]; height],
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> &RenderCell {
        &self.cells[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, cell: RenderCell) {
        self.cells[y][x] = cell;
    }

    pub fn fill(&mut self, cell: RenderCell) {
        for row in &mut self.cells {
            for current in row {
                *current = cell.clone();
            }
        }
    }
}

/// Which screen is being rendered.
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenModel {
    Title,
    Gameplay,
    Win,
    Lost,
}

/// Holds current screen model + frame info for the renderer.
#[derive(Debug, Clone)]
pub struct ViewModel {
    pub screen: ScreenModel,
    pub frame_number: u64,
}

impl ViewModel {
    pub fn new(screen: ScreenModel) -> Self {
        Self {
            screen,
            frame_number: 0,
        }
    }

    pub fn advance_frame(&mut self) {
        self.frame_number += 1;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Enemy {
    pub position: Position,
    pub glyph: char,
}

pub struct App {
    pub map: Map,
    pub visibility: VisibilityMap,
    pub player: Player,
    pub player_animation: Option<AnimationState>,
    pub enemy_animations: Vec<(usize, AnimationState)>,
    pub input_queue: Vec<(VirtualKeyCode, bool)>,
    pub enemies: Vec<Enemy>,
    pub lives: usize,
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
    pub audio: AudioManager,
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
            (VimMotion::G, "G"),
            (VimMotion::GotoLine, "gg"),
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

    #[test]
    fn new_motion_display_names() {
        assert_eq!(VimMotion::G.display_name(), "Last row");
        assert_eq!(VimMotion::GotoLine.display_name(), "First row");
    }

    #[test]
    fn new_motion_descriptions_non_empty() {
        assert!(!VimMotion::G.description().is_empty());
        assert!(!VimMotion::GotoLine.description().is_empty());
    }

    #[test]
    fn enemy_struct_fields() {
        let enemy = Enemy {
            position: Position { x: 5, y: 10 },
            glyph: 'X',
        };
        assert_eq!(enemy.position.x, 5);
        assert_eq!(enemy.position.y, 10);
        assert_eq!(enemy.glyph, 'X');
    }

    #[test]
    fn total_levels_is_three() {
        assert_eq!(TOTAL_LEVELS, 3);
    }

    #[test]
    fn render_cell_new() {
        let cell = RenderCell::new('@', (1, 2, 3), (4, 5, 6));
        assert_eq!(cell.glyph, '@');
        assert_eq!(cell.fg, (1, 2, 3));
        assert_eq!(cell.bg, (4, 5, 6));
        assert!(!cell.blink);
    }

    #[test]
    fn render_cell_with_blink() {
        let cell = RenderCell::new('@', (1, 2, 3), (4, 5, 6)).with_blink();
        assert!(cell.blink);
    }

    #[test]
    fn render_grid_new_dimensions() {
        let default = RenderCell::new('.', (1, 1, 1), (0, 0, 0));
        let grid = RenderGrid::new(3, 2, default);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 2);
    }

    #[test]
    fn render_grid_get_set() {
        let default = RenderCell::new('.', (1, 1, 1), (0, 0, 0));
        let mut grid = RenderGrid::new(2, 2, default);
        let cell = RenderCell::new('@', (10, 20, 30), (40, 50, 60));
        grid.set(1, 0, cell.clone());
        assert_eq!(grid.get(1, 0), &cell);
    }

    #[test]
    fn render_grid_fill() {
        let default = RenderCell::new('.', (1, 1, 1), (0, 0, 0));
        let mut grid = RenderGrid::new(2, 2, default);
        let fill_cell = RenderCell::new('#', (9, 9, 9), (8, 8, 8)).with_blink();
        grid.fill(fill_cell.clone());

        for y in 0..grid.height() {
            for x in 0..grid.width() {
                assert_eq!(grid.get(x, y), &fill_cell);
            }
        }
    }

    #[test]
    fn render_grid_default_cells() {
        let default = RenderCell::new('.', (1, 1, 1), (0, 0, 0));
        let grid = RenderGrid::new(2, 2, default.clone());
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                assert_eq!(grid.get(x, y), &default);
            }
        }
    }

    #[test]
    fn screen_model_covers_all_states() {
        let cases = [
            (ScreenModel::Title, "title"),
            (ScreenModel::Gameplay, "gameplay"),
            (ScreenModel::Win, "win"),
            (ScreenModel::Lost, "lost"),
        ];

        for (screen, expected) in cases {
            let label = match screen {
                ScreenModel::Title => "title",
                ScreenModel::Gameplay => "gameplay",
                ScreenModel::Win => "win",
                ScreenModel::Lost => "lost",
            };
            assert_eq!(label, expected);
        }
    }

    #[test]
    fn view_model_frame_advances() {
        let mut view = ViewModel::new(ScreenModel::Gameplay);
        view.advance_frame();
        view.advance_frame();
        assert_eq!(view.frame_number, 2);
    }

    #[test]
    fn view_model_new_starts_at_frame_zero() {
        let view = ViewModel::new(ScreenModel::Title);
        assert_eq!(view.frame_number, 0);
    }
}
