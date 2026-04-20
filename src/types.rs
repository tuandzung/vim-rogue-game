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
pub const TOTAL_LEVELS: usize = 4;
pub const FOV_RADIUS: i32 = 10;
pub const MAX_HP: i32 = 30;
pub const TORCHLIGHT_FOV_RADIUS: i32 = 6;

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
    Torchlight,
}

impl Tile {
    pub fn glyph(self) -> char {
        match self {
            Self::Wall => '#',
            Self::Floor => '.',
            Self::Exit => '>',
            Self::Obstacle => '▒',
            Self::Torchlight => 'i',
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
            Self::G => "Column bottom",
            Self::GotoLine => "Column top",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::H => "Move one tile left",
            Self::J => "Move one tile down",
            Self::K => "Move one tile up",
            Self::L => "Move one tile right",
            Self::W => "Jump to the next segment along a clear path",
            Self::B => "Jump to the previous segment along a clear path",
            Self::Zero => "Jump to the first passable tile on the row",
            Self::Dollar => "Jump to the last passable tile on the row",
            Self::Find => "Jump to the next matching tile character",
            Self::Till => "Stop one tile before the next matching character",
            Self::DeleteLine => "Turn the nearest obstacle on the row into floor",
            Self::G => "Jump down the column until blocked",
            Self::GotoLine => "Jump up the column until blocked",
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
    Paused,
    Quit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PauseOption {
    Resume,
    RetryLevel,
    QuitGame,
}

impl PauseOption {
    pub fn next(self) -> Self {
        match self {
            Self::Resume => Self::RetryLevel,
            Self::RetryLevel => Self::QuitGame,
            Self::QuitGame => Self::Resume,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Resume => Self::QuitGame,
            Self::RetryLevel => Self::Resume,
            Self::QuitGame => Self::RetryLevel,
        }
    }
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
    pub hp: Option<i32>,
    pub stunned_turns: usize,
}

pub struct App {
    pub map: Map,
    pub visibility: VisibilityMap,
    pub player: Player,
    pub player_animation: Option<AnimationState>,
    pub enemy_animations: Vec<(usize, AnimationState)>,
    pub input_queue: Vec<(VirtualKeyCode, bool)>,
    pub enemies: Vec<Enemy>,
    pub hp: i32,
    pub game_state: GameState,
    pub pause_selection: PauseOption,
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
    pub last_checkpoint: Option<Position>,
    pub activated_torchlights: HashSet<Position>,
}
