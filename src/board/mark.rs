use super::Coords;
use math_tools::bounded_value::Digit;
use crate::{canvas, Degree};
use canvas::{line, Canvas, Pos, Size};
use line::*;
use std::fmt::{self, Display, Formatter};
use serde::{Serialize, Deserialize};
use PlayerMark::*;
use CellMark::*;
use Mark::*;
use MarkError::*;
use LineVariant::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerMark {
    X,
    O,
}

impl From<&PlayerMark> for char {
    fn from(mark: &PlayerMark) -> Self {
        match mark {
            X => 'X',
            O => 'O',
        }
    }
}

impl Display for PlayerMark {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        char::from(self).fmt(f)
    }
}

impl TryFrom<CellMark> for PlayerMark {
    type Error = MarkError;

    fn try_from(cell_mark: CellMark) -> Result<Self, Self::Error> {
        if let Player(player_mark) = cell_mark {
            return Ok(player_mark);
        }

        Err(DownCast)
    }
}

impl TryFrom<Mark> for PlayerMark {
    type Error = MarkError;

    fn try_from(mark: Mark) -> Result<Self, Self::Error> {
        if let Cell(cell_mark) = mark {
            return Ok(PlayerMark::try_from(cell_mark)?);
        }

        Err(DownCast)
    }
}

impl PlayerMark {
    pub fn not(&self) -> Self {
        match self {
            X => O,
            O => X,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellMark {
    Player(PlayerMark),
    T,
}

impl From<&CellMark> for char {
    fn from(mark: &CellMark) -> Self {
        match mark {
            Player(mark) => char::from(mark),
            T => 'T',
        }
    }
}

impl Display for CellMark {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        char::from(self).fmt(f)
    }
}

impl From<PlayerMark> for CellMark {
    fn from(player_mark: PlayerMark) -> Self {
        Player(player_mark)
    }
}

impl TryFrom<Mark> for CellMark {
    type Error = MarkError;

    fn try_from(mark: Mark) -> Result<Self, Self::Error> {
        if let Cell(cell_mark) = mark {
            return Ok(cell_mark);
        }

        Err(DownCast)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Grid,
    Num(Digit),
    Cell(CellMark),
}

impl From<&Mark> for char {
    fn from(mark: &Mark) -> Self {
        match mark {
            Grid => '#',
            Num(digit) => digit.to_string().parse().unwrap(),
            Cell(mark) => char::from(mark),
        }
    }
}

impl Display for Mark {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        char::from(self).fmt(f)
    }
}

impl From<PlayerMark> for Mark {
    fn from(player_mark: PlayerMark) -> Self {
        Cell(Player(player_mark))
    }
}

impl From<CellMark> for Mark {
    fn from(cell_mark: CellMark) -> Self {
        Cell(cell_mark)
    }
}

type Dimensions = (Pos, usize, usize, usize, usize);

impl Mark {
    unsafe fn draw_grid(&self, canvas: &mut Canvas, (pos, rows, cols, sub_rows, sub_cols): Dimensions) {
        let mut horiz = Line::new(pos + Pos::new(sub_rows, 0), cols, '-', Horiz);
        unsafe {
            horiz.draw_unchecked(canvas);
        }

        horiz.pos().row += sub_rows + 1;
        unsafe {
            horiz.draw_unchecked(canvas);
        }

        let mut vert = Line::new(pos + Pos::new(0, sub_cols), rows, '|', Vert);
        unsafe {
            vert.draw_unchecked(canvas);
        }

        vert.pos().col += sub_cols + 1;
        unsafe {
            vert.draw_unchecked(canvas);
        }
    }

    unsafe fn draw_x(&self, canvas: &mut Canvas, (pos, rows, cols, sub_rows, sub_cols): Dimensions) {
        let mut diag = Line::new(pos + Pos::new(sub_rows, sub_cols), sub_rows + 2, '\\', DiagDown);
        unsafe {
            diag.draw_unchecked(canvas);
        }

        *diag.ch() = '/';
        *diag.variant() = DiagUp;
        unsafe {
            diag.draw_unchecked(canvas);
            canvas.set_unchecked(pos + Pos::new(rows / 2, cols / 2), 'X');
        }
    }

    unsafe fn draw_t(&self, canvas: &mut Canvas, (pos, _, cols, sub_rows, sub_cols): Dimensions) {
        unsafe {
            Line::new(pos + Pos::new(sub_rows - 1, sub_cols), sub_cols + 2, '_', Horiz).draw_unchecked(canvas);
            Line::new(pos + Pos::new(sub_rows, cols / 2), sub_rows + 2, '|', Vert).draw_unchecked(canvas);
        }
    }

    unsafe fn draw_num(&self, canvas: &mut Canvas, (pos, rows, _, sub_rows, sub_cols): Dimensions, num: Digit) {
        let num = num.get();
        let mid = rows / 2;
        let vert_right_col = 2 * sub_cols + 1;
        let vert_bottom_row = mid + 1;
        let mut horiz = Line::new(pos + Pos::new(sub_rows - 1, sub_cols + 1), sub_cols, '-', Horiz);
        let mut vert = Line::new(pos + Pos::new(sub_rows, sub_cols), mid - sub_rows, '|', Vert);

        unsafe {
            if [0, 2, 3, 5, 6, 7, 8, 9].contains(num) {
                horiz.draw_unchecked(canvas);
            }
            if [0, 4, 5, 6, 8, 9].contains(num) {
                vert.draw_unchecked(canvas);
            }
            if [0, 1, 2, 3, 4, 7, 8, 9].contains(num) {
                vert.pos().col = pos.col + vert_right_col;
                vert.draw_unchecked(canvas);
            }
            if [2, 3, 4, 5, 6, 8, 9].contains(num) {
                horiz.pos().row = pos.row + mid;
                horiz.draw_unchecked(canvas);
            }
            if [0, 2, 6, 8].contains(num) {
                *vert.pos() = pos + Pos::new(vert_bottom_row, sub_cols);
                vert.draw_unchecked(canvas);
            }
            if [0, 1, 3, 4, 5, 6, 7, 8, 9].contains(num) {
                *vert.pos() = pos + Pos::new(vert_bottom_row, vert_right_col);
                vert.draw_unchecked(canvas);
            }
            if [0, 2, 3, 5, 6, 8, 9].contains(num) {
                horiz.pos().row = pos.row + 2 * (sub_rows + 1);
                horiz.draw_unchecked(canvas);
            }
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, coords: &Coords) {
        let mut pos = canvas.get_pos(coords);
        let degree = canvas.degree().get() - coords.0.len() as u8;

        if degree == 0 {
            pos.col += 1;

            unsafe {
                canvas.set_unchecked(pos, self.to_string().parse::<char>().unwrap());
            }
        } else {
            let size = Size::from(Degree::new(degree));
            let sub_size = Size::from(Degree::new(degree - 1));
            let dimensions = (pos, size.rows(), size.cols(), sub_size.rows(), sub_size.cols());

            unsafe {
                match self {
                    Grid => self.draw_grid(canvas, dimensions),
                    Num(num) => self.draw_num(canvas, dimensions, *num),
                    Cell(cell_mark) => match cell_mark {
                        Player(player_mark) => match player_mark {
                            X => self.draw_x(canvas, dimensions),
                            O => self.draw_num(canvas, dimensions, Digit::new(0)),
                        },
                        T => self.draw_t(canvas, dimensions),
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkError {
    DownCast,
}

impl Display for MarkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DownCast => write!(f, "Invalid down cast operation"),
        }
    }
}

impl std::error::Error for MarkError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::CellBranch;

    #[test]
    fn test() {
        let mut canvas = Canvas::from(Degree::new(4));
        let mut coords = Coords::default();

        Grid.draw(&mut canvas, &coords);

        while (*canvas.degree().get() as usize) > coords.0.len() {
            let depth = coords.0.len();

            let variants = [
                X.into(),
                T.into(),
                O.into(),
                Num(Digit::new(0)).into(),
                Grid,
                Num(Digit::new(6)).into(),
                Num(Digit::new(7)).into(),
                Num(Digit::new(8)).into(),
                Grid,
            ];

            coords.0.push(CellBranch::new(0));

            for (i, variant) in variants.into_iter().enumerate() {
                coords.0[depth] = CellBranch::new(i);
                variant.draw(&mut canvas, &coords);
            }

            if depth > 0 {
                let prev_depth = depth - 1;
                let original_parent = coords.0[prev_depth];

                coords.0[prev_depth] = CellBranch::new(8);

                for i in 0..=8 {
                    coords.0[depth] = CellBranch::new(i);
                    let digit: Mark = Num(Digit::new(i as u8 + 1)).into();
                    digit.draw(&mut canvas, &coords);
                }

                coords.0[prev_depth] = original_parent;
            }

            coords.0[depth] = CellBranch::new(4);
        }

        println!("{canvas}");
    }
}