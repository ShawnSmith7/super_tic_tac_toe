pub mod line;
mod prims;

pub use prims::{Size, Pos};

use std::fmt::{self, Display, Formatter};
use std::ops::{Index, IndexMut};

use CanvasError::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Canvas {
    size: Size,
    data: Vec<char>,
}

impl From<Size> for Canvas {
    fn from(size: Size) -> Self {
        Self {
            size,
            data: vec![' '; size.rows * size.cols],
        }
    }
}

impl Display for Canvas {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.size.cols == 0 {
            return Ok(());
        }

        for (i, row) in self.data.chunks(self.size.cols).enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            for &ch in row {
                write!(f, "{ch}")?;
            }
        }

        Ok(())
    }
}

impl Index<Pos> for Canvas {
    type Output = char;

    fn index(&self, pos: Pos) -> &Self::Output {
        let idx = self.index_of(pos).unwrap_or_else(|err| panic!("{err}"));
        &self.data[idx]
    }
}

impl IndexMut<Pos> for Canvas {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        let idx = self.index_of(pos).unwrap_or_else(|err| panic!("{err}"));
        &mut self.data[idx]
    }
}

impl Canvas {
    pub fn size(&self) -> Size {
        self.size
    }

    pub fn is_in_bounds(&self, pos: Pos) -> bool {
        pos.row < self.size.rows && pos.col < self.size.cols
    }

    fn index_of(&self, pos: Pos) -> Result<usize, CanvasError> {
        if !self.is_in_bounds(pos) {
            return Err(OutOfBounds(pos));
        }

        Ok(pos.row * self.size.cols + pos.col)
    }

    pub fn get(&self, pos: Pos) -> Result<char, CanvasError> {
        let idx = self.index_of(pos)?;
        Ok(self.data[idx])
    }

    pub fn get_mut(&mut self, pos: Pos) -> Result<&mut char, CanvasError> {
        let idx = self.index_of(pos)?;
        Ok(&mut self.data[idx])
    }

    pub fn set(&mut self, pos: Pos, ch: char) -> Result<(), CanvasError> {
        *self.get_mut(pos)? = ch;
        Ok(())
    }

    pub fn get_row_slice(&self, pos: Pos, len: usize) -> Result<&[char], CanvasError> {
        if !self.is_in_bounds(pos) || len > self.size.cols - pos.col {
            return Err(OutOfBounds(pos));
        }

        let idx = self.index_of_unchecked(pos);
        Ok(&self.data[idx..idx + len])
    }

    pub fn get_rows(
        &self,
        start_row: usize,
        end_row: usize,
    ) -> Result<impl Iterator<Item = &[char]> + DoubleEndedIterator, CanvasError> {
        if start_row > end_row || end_row > self.size.rows {
            return Err(OutOfBounds(Pos::new(start_row, 0)));
        }

        Ok(self.data
            .chunks_exact(self.size.cols)
            .skip(start_row)
            .take(end_row - start_row))
    }

    pub fn get_row_slice_mut(&mut self, pos: Pos, len: usize) -> Result<&mut [char], CanvasError> {
        if !self.is_in_bounds(pos) || len > self.size.cols - pos.col {
            return Err(OutOfBounds(pos));
        }

        let idx = self.index_of_unchecked(pos);
        Ok(&mut self.data[idx..idx + len])
    }

    pub fn get_rows_mut(
        &mut self,
        start_row: usize,
        end_row: usize
    ) -> Result<impl Iterator<Item = &mut [char]> + DoubleEndedIterator, CanvasError> {
        if start_row > end_row || end_row > self.size.rows {
            return Err(OutOfBounds(Pos::new(start_row, 0)));
        }

        Ok(self.data
            .chunks_exact_mut(self.size.cols)
            .skip(start_row)
            .take(end_row - start_row))
    }

    fn index_of_unchecked(&self, pos: Pos) -> usize {
        pos.row * self.size.cols + pos.col
    }

    pub unsafe fn get_unchecked(&self, pos: Pos) -> char {
        let idx = self.index_of_unchecked(pos);
        unsafe {
            *self.data.get_unchecked(idx)
        }
    }

    pub unsafe fn get_mut_unchecked(&mut self, pos: Pos) -> &mut char {
        let idx = self.index_of_unchecked(pos);
        unsafe {
            self.data.get_unchecked_mut(idx)
        }
    }

    pub unsafe fn set_unchecked(&mut self, pos: Pos, ch: char) {
        unsafe {
            *self.get_mut_unchecked(pos) = ch;
        }
    }

    pub unsafe fn get_row_slice_mut_unchecked(&mut self, pos: Pos, len: usize) -> &mut [char] {
        let idx = self.index_of_unchecked(pos);
        unsafe {
            self.data.get_unchecked_mut(idx..idx + len)
        }
    }

    pub unsafe fn get_rows_mut_unchecked(
        &mut self,
        start_row: usize,
        end_row: usize,
    ) -> impl Iterator<Item = &mut [char]> + DoubleEndedIterator {
        let start_idx = start_row * self.size.cols;
        let end_idx = end_row * self.size.cols;

        unsafe {
            self.data
                .get_unchecked_mut(start_idx..end_idx)
                .chunks_exact_mut(self.size.cols)
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(' ');
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasError {
    OutOfBounds(Pos),
}

impl Display for CanvasError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OutOfBounds(pos) => write!(f, "{pos:?} is out of mod bounds"),
        }
    }
}

impl std::error::Error for CanvasError {}

pub trait Draw {
    fn draw(&self, canvas: &mut Canvas) -> Result<(), CanvasError>;
}

pub trait DrawUnchecked {
    unsafe fn draw_unchecked(&self, canvas: &mut Canvas);
}