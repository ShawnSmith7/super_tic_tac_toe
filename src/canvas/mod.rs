mod size;
pub mod line;

pub use size::Size;
pub use canvas::Pos;

use terminal_tools::canvas;
use super::Degree;
use crate::board::Coords;
use std::fmt::{self, Display, Formatter};
use canvas::Canvas as InternalCanvas;

pub const GRID_DIMENSIONS: usize = 3;
pub const BASE_CELL_ROWS_PLUS_ONE: usize = 2;
pub const BASE_CELL_COLS_PLUS_ONE: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    degree: Degree,
    data: InternalCanvas,
}

impl From<Degree> for Canvas {
    fn from(degree: Degree) -> Self {
        Self {
            degree,
            data: InternalCanvas::from(Size::from(degree).0),
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::from(Degree::new(2))
    }
}

impl Display for Canvas {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl Canvas {
    pub fn degree(&self) -> Degree {
        self.degree
    }

    pub fn size(&self) -> Size {
        Size(self.data.size())
    }

    pub fn get_pos(&self, coords: &Coords) -> Pos {
        let mut pos = Pos::default();

        for (i, coord) in coords.0.iter().enumerate() {
            let (i, coord) = (i as u32, *coord.get());
            let scalar = GRID_DIMENSIONS.pow(*self.degree.get() as u32 - 1 - i);

            pos.row += BASE_CELL_ROWS_PLUS_ONE * scalar * (coord / GRID_DIMENSIONS);
            pos.col += BASE_CELL_COLS_PLUS_ONE * scalar * (coord % GRID_DIMENSIONS);
        }

        pos
    }
    
    pub unsafe fn set_unchecked(&mut self, pos: Pos, ch: char) {
        unsafe {
            self.data.set_unchecked(pos, ch)
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}