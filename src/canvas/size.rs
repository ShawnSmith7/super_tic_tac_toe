use terminal_tools::canvas;
use canvas::Size as InternalSize;
use crate::Level;
use super::{GRID_DIMENSIONS, BASE_CELL_ROWS_PLUS_ONE, BASE_CELL_COLS_PLUS_ONE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(pub(in crate::canvas)InternalSize);

impl Default for Size {
    fn default() -> Self {
        Self::from(Level::new(1))
    }
}

impl From<Level> for Size {
    fn from(level: Level) -> Self {
        let scalar = GRID_DIMENSIONS.pow(*level.get() as u32);
        Self(InternalSize::new(BASE_CELL_ROWS_PLUS_ONE * scalar - 1, BASE_CELL_COLS_PLUS_ONE * scalar - 1))
    }
}

impl Size {
    pub fn rows(&self) -> usize {
        self.0.rows
    }

    pub fn cols(&self) -> usize {
        self.0.cols
    }
}