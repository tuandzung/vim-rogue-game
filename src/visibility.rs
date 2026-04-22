use crate::types::Position;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisibilityState {
    Hidden,
    Explored,
    Visible,
}

pub struct VisibilityMap {
    width: usize,
    height: usize,
    states: Vec<VisibilityState>,
}

impl VisibilityMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            states: vec![VisibilityState::Hidden; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, pos: Position) -> VisibilityState {
        if self.in_bounds(pos) {
            self.states[pos.y * self.width + pos.x]
        } else {
            VisibilityState::Hidden
        }
    }

    pub fn set(&mut self, pos: Position, state: VisibilityState) {
        if self.in_bounds(pos) {
            self.states[pos.y * self.width + pos.x] = state;
        }
    }

    pub fn compute_fov<F>(&mut self, center: Position, radius: i32, is_transparent: F)
    where
        F: Fn(Position) -> bool,
    {
        self.set(center, VisibilityState::Visible);

        let cx = center.x as i32;
        let cy = center.y as i32;
        let r = radius.max(0);
        let num_rays = 360;
        let step: f64 = 2.0 * std::f64::consts::PI / num_rays as f64;

        for i in 0..num_rays {
            let angle = i as f64 * step;
            let dx = angle.cos();
            let dy = angle.sin();

            let mut x = cx as f64;
            let mut y = cy as f64;

            for _ in 0..r {
                x += dx;
                y += dy;

                let ix = x.round() as i32;
                let iy = y.round() as i32;

                if ix < 0 || iy < 0 {
                    break;
                }
                let ux = ix as usize;
                let uy = iy as usize;
                let pos = Position { x: ux, y: uy };

                if !self.in_bounds(pos) {
                    break;
                }

                let dist_sq = (ix - cx) * (ix - cx) + (iy - cy) * (iy - cy);
                if dist_sq > r * r {
                    break;
                }

                let transparent = is_transparent(pos);
                self.set(pos, VisibilityState::Visible);

                if !transparent {
                    break;
                }
            }
        }
    }

    /// Compute FOV from multiple sources, unioning the visible results.
    /// Each source is (position, radius). Tiles visible from ANY source are set to Visible.
    pub fn compute_multi_fov<F>(&mut self, sources: &[(Position, i32)], is_transparent: F)
    where
        F: Fn(Position) -> bool,
    {
        for &(center, radius) in sources {
            self.compute_fov(center, radius, &is_transparent);
        }
    }

    pub fn demote_visible_to_explored(&mut self) {
        for state in &mut self.states {
            if *state == VisibilityState::Visible {
                *state = VisibilityState::Explored;
            }
        }
    }

    pub fn reset(&mut self) {
        for state in &mut self.states {
            *state = VisibilityState::Hidden;
        }
    }

    fn in_bounds(&self, pos: Position) -> bool {
        pos.x < self.width && pos.y < self.height
    }
}
