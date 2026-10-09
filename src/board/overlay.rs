use super::{Mark, Coords};
use math_tools::bounded_value::Digit;

use Mark::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cell {
    mark: Mark,
    coords: Coords,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overlay<'a> {
    cells: Vec<Cell>,
    coords: &'a Coords,
}

impl<'a> From<&'a Coords> for Overlay<'a> {
    fn from(coords: &'a Coords) -> Self {
        let mut overlay = Self {
            cells: Default::default(),
            coords,
        };

        for i in 0..9 {
            unsafe {
                overlay.cells.push(Cell {
                    mark: Num(Digit::new_unchecked(i + 1)),
                    coords: Coords::new(vec![i as usize]),
                });
            }
        }

        overlay
    }
}

impl<'a> Overlay<'a> {
    pub fn get(&self, coords: &Coords) -> Option<Mark> {
        for cell in &self.cells {
            if coords == &(self.coords + &cell.coords) {
                return Some(cell.mark);
            }
        }

        None
    }
}