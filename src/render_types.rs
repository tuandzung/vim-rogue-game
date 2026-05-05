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
