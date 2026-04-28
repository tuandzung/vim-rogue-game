use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};

use bracket_lib::prelude::VirtualKeyCode;

use crate::animation::{AnimationState, AttackEffect};
use crate::audio::AudioManager;
use crate::map::Map;
use crate::player::Player;
use crate::visibility::VisibilityMap;

pub const TRAIL_MAX: usize = 8;
pub const TOTAL_LEVELS: usize = 4;
pub const FOV_RADIUS: i32 = 10;
pub const MAX_HP: i32 = 30;
pub const TORCHLIGHT_FOV_RADIUS: i32 = 6;
pub const ENEMY_FOV_RADIUS: i32 = 8;

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
    Dying,
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
        Self { glyph, fg, bg, blink: false }
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
        Self { cells: vec![vec![default; width]; height], width, height }
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
        Self { screen, frame_number: 0 }
    }

    pub fn advance_frame(&mut self) {
        self.frame_number += 1;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct PatrolArea {
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl PatrolArea {
    pub fn point(x: usize, y: usize) -> Self {
        Self { min_x: x, min_y: y, max_x: x, max_y: y }
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Enemy {
    pub position: Position,
    pub glyph: char,
    pub hp: Option<i32>,
    pub stunned_turns: usize,
    pub patrol_area: PatrolArea,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct CheatBuffer {
    buf: [Option<char>; 2],
    len: usize,
}

#[cfg(debug_assertions)]
impl CheatBuffer {
    pub fn new() -> Self {
        Self { buf: [None; 2], len: 0 }
    }

    pub fn push(&mut self, ch: char) {
        if self.len < 2 {
            self.buf[self.len] = Some(ch);
            self.len += 1;
        } else {
            self.buf[0] = self.buf[1];
            self.buf[1] = Some(ch);
        }
    }

    pub fn clear(&mut self) {
        self.buf = [None; 2];
        self.len = 0;
    }

    pub fn chars(&self) -> (Option<char>, Option<char>) {
        (self.buf[0], self.buf[1])
    }
}

#[cfg(debug_assertions)]
impl Default for CheatBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct World {
    pub map: Map,
    pub visibility: VisibilityMap,
    pub enemies: Vec<Enemy>,
    pub activated_torchlights: HashSet<Position>,
}

impl World {
    pub fn new(map: Map) -> Self {
        let visibility = VisibilityMap::new(map.width, map.height);
        Self { map, visibility, enemies: Vec::new(), activated_torchlights: HashSet::new() }
    }

    pub fn reset_for_level(&mut self, level: usize) {
        self.map = Map::level(level);
        if self.visibility.width() != self.map.width || self.visibility.height() != self.map.height
        {
            self.visibility = VisibilityMap::new(self.map.width, self.map.height);
        }
        self.visibility.reset();
        self.enemies.clear();
        self.activated_torchlights.clear();
    }

    pub fn spawn_enemies(&mut self, level: usize) {
        self.enemies = self
            .map
            .enemy_spawns
            .iter()
            .enumerate()
            .map(|(i, &pos)| {
                let patrol_area = self
                    .map
                    .enemy_patrol_areas
                    .get(i)
                    .copied()
                    .unwrap_or_else(|| PatrolArea::point(pos.x, pos.y));
                if level == 4 {
                    Enemy { position: pos, glyph: 'e', hp: Some(30), stunned_turns: 0, patrol_area }
                } else {
                    let mut e = Enemy::new(pos);
                    e.patrol_area = patrol_area;
                    e
                }
            })
            .collect();
    }

    pub fn update_visibility(&mut self, player_pos: Position) {
        if self.visibility.width() != self.map.width || self.visibility.height() != self.map.height
        {
            self.visibility = VisibilityMap::new(self.map.width, self.map.height);
        }
        self.visibility.demote_visible_to_explored();
        self.visibility.compute_fov(player_pos, FOV_RADIUS, |pos| {
            matches!(
                self.map.get_tile(pos.x, pos.y),
                Tile::Floor | Tile::Exit | Tile::Obstacle | Tile::Torchlight
            )
        });
        let sources: Vec<(Position, i32)> =
            self.activated_torchlights.iter().map(|&pos| (pos, TORCHLIGHT_FOV_RADIUS)).collect();
        if !sources.is_empty() {
            self.visibility.compute_multi_fov(&sources, |pos| {
                matches!(
                    self.map.get_tile(pos.x, pos.y),
                    Tile::Floor | Tile::Exit | Tile::Obstacle | Tile::Torchlight
                )
            });
        }
    }
}

pub struct PlayerState {
    pub inner: Player,
    pub hp: i32,
    pub trail: VecDeque<Position>,
    pub motion_count: usize,
    pub discovered_motions: HashSet<VimMotion>,
    pub level: usize,
    pub last_checkpoint: Option<Position>,
    pub pending_respawn: Option<Position>,
}

impl PlayerState {
    pub fn new(position: Position) -> Self {
        Self {
            inner: Player::new(position),
            hp: MAX_HP,
            trail: VecDeque::new(),
            motion_count: 0,
            discovered_motions: HashSet::new(),
            level: 1,
            last_checkpoint: None,
            pending_respawn: None,
        }
    }

    pub fn advance_level(&mut self, level: usize, start: Position) {
        self.level = level;
        self.inner.position = start;
        self.trail.clear();
        self.last_checkpoint = None;
        self.pending_respawn = None;
    }

    pub fn retry_level(&mut self, start: Position) {
        self.inner.position = start;
        self.hp = MAX_HP;
        self.trail.clear();
        self.last_checkpoint = None;
        self.pending_respawn = None;
    }

    pub fn motion_feedback(&self, motion: VimMotion, target: Option<char>) -> String {
        match motion {
            VimMotion::DeleteLine => String::from("dd clears the nearest obstacle on your row."),
            VimMotion::Find => target
                .map(|ch| format!("f{ch} searches forward for the next matching tile."))
                .unwrap_or_else(|| String::from("Find motion ready.")),
            VimMotion::Till => target
                .map(|ch| format!("t{ch} stops one tile before the next match."))
                .unwrap_or_else(|| String::from("Till motion ready.")),
            _ => format!("{} — {}", motion.key_label(), motion.description()),
        }
    }

    pub fn damage_feedback(&self) -> String {
        format!("Hit! {} HP remaining.", self.hp)
    }
}

pub struct InputState {
    pub input_queue: Vec<(VirtualKeyCode, bool)>,
    pub pending_input: Option<PendingInput>,
}

impl InputState {
    pub fn new() -> Self {
        Self { input_queue: Vec::new(), pending_input: None }
    }

    pub fn clear(&mut self) {
        self.input_queue.clear();
        self.pending_input = None;
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Session {
    pub game_state: GameState,
    pub pause_selection: PauseOption,
    pub started: bool,
    pub status_message: String,
    pub start_time: Instant,
    pub elapsed: Duration,
    pub final_time: Option<Duration>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            game_state: GameState::Playing,
            pause_selection: PauseOption::Resume,
            started: false,
            status_message: String::from(
                "Explore the dungeon and practice the highlighted motions.",
            ),
            start_time: Instant::now(),
            elapsed: Duration::default(),
            final_time: None,
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

pub struct App {
    pub world: World,
    pub player: PlayerState,
    pub input: InputState,
    pub session: Session,
    pub player_animation: Option<AnimationState>,
    pub enemy_animations: Vec<(usize, AnimationState)>,
    pub attack_effects: Vec<AttackEffect>,
    pub audio: AudioManager,
    #[cfg(debug_assertions)]
    pub cheat_buf: CheatBuffer,
    #[cfg(debug_assertions)]
    pub cheat_god_mode: bool,
}
