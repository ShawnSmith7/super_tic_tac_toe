use super::{Pos, Canvas, CanvasError, Draw, DrawUnchecked};

use LineVariant::*;
use CanvasError::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub pos: Pos,
    pub len: usize,
    pub ch: char,
    pub variant: LineVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineVariant {
    Horiz,
    Vert,
    DiagDown,
    DiagUp,
}

impl DrawUnchecked for Line {
    unsafe fn draw_unchecked(&self, canvas: &mut Canvas) {
        unsafe {
            match self.variant {
                Horiz => self.draw_horiz(canvas),
                other => {
                    let rows = canvas.get_rows_mut_unchecked(self.pos.row, self.pos.row + self.len);

                    match other {
                        Horiz => unreachable!(),
                        Vert => self.draw_vert(rows),
                        DiagDown => self.draw_diag_down(rows),
                        DiagUp => self.draw_diag_up(rows),
                    }
                }
            }
        }
    }
}

impl Draw for Line {
    fn draw(&self, canvas: &mut Canvas) -> Result<(), CanvasError> {
        if !self.is_in_bounds(canvas) {
            return Err(OutOfBounds(self.pos));
        }

        unsafe {
            self.draw_unchecked(canvas);
        }
        Ok(())
    }
}

impl Line {
    pub fn new(pos: Pos, len: usize, ch: char, variant: LineVariant) -> Self {
        Self {
            pos,
            len,
            ch,
            variant,
        }
    }

    unsafe fn draw_horiz(&self, canvas: &mut Canvas) {
        unsafe {
            canvas.get_row_slice_mut_unchecked(self.pos, self.len).fill(self.ch);
        }
    }

    unsafe fn draw_vert<'a>(&self, rows: impl Iterator<Item = &'a mut [char]>) {
        for row in rows {
            unsafe {
                *row.get_unchecked_mut(self.pos.col) = self.ch;
            }
        }
    }

    unsafe fn draw_diag_down<'a>(&self, rows: impl Iterator<Item = &'a mut [char]>) {
        for (i, row) in rows.enumerate() {
            unsafe {
                *row.get_unchecked_mut(self.pos.col + 2 * i) = self.ch;
            }
        }
    }

    unsafe fn draw_diag_up<'a>(&self, rows: impl Iterator<Item = &'a mut [char]> + DoubleEndedIterator) {
        for (i, row) in rows.rev().enumerate() {
            unsafe {
                *row.get_unchecked_mut(self.pos.col + 2 * i) = self.ch;
            }
        }
    }

    pub fn is_in_bounds(&self, canvas: &Canvas) -> bool {
        if self.len == 0 {
            return true;
        }

        let canvas_size = canvas.size();
        if !canvas.is_in_bounds(self.pos) {
            return false;
        }

        match self.variant {
            Horiz => self.pos.col + self.len <= canvas_size.cols,
            Vert => self.pos.row + self.len <= canvas_size.rows,
            DiagDown => {
                self.pos.row + self.len <= canvas_size.rows
                    && self.pos.col + self.len <= canvas_size.cols
            }
            DiagUp => {
                self.pos.row + self.len <= canvas_size.rows
                    && self.pos.col + self.len <= canvas_size.cols
            }
        }
    }
}