pub mod mark;
mod coords;
pub mod cursor;
mod overlay;

pub use coords::Coords;
pub use overlay::Overlay;

use bounded_value::BoundedValue;
use math_tools::{bounded_value, define_bounds};
use data_structures::arena_tree::{ArenaTree, ArenaTreeError, Node};
use std::{fmt, ops};
use fmt::{Display, Formatter};
use super::{canvas, UserLevel};
use canvas::Canvas;
use mark::{CellMark, Mark};
use cursor::BoardCursor;
use serde::{Serialize, Deserialize};

use ops::Bound::*;
use ArenaTreeError::*;
use BoardError::*;
use Mark::*;

const CELLS_PER_GRID: usize = 9;
define_bounds!(CellBranchBounds, usize, Included(0), Included(8));
pub type CellBranch = BoundedValue<CellBranchBounds>;
define_bounds!(UserCellBranchBounds, usize, Included(1), Included(9));
pub type UserCellBranch = BoundedValue<UserCellBranchBounds>;
pub type Cell = Node<Option<CellMark>, CELLS_PER_GRID>;
pub use data_structures::arena_tree::NodeId as CellId;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Board {
    level: UserLevel,
    inner: ArenaTree<Option<CellMark>, CELLS_PER_GRID>,
}

impl From<UserLevel> for Board {
    fn from(level: UserLevel) -> Self {
        Self {
            level,
            inner: Default::default(),
        }
    }
}

impl Board {
    pub fn level(&self) -> UserLevel {
        self.level
    }

    pub fn get_checked(&self, id: CellId) -> Result<&Cell, BoardError> {
        self.inner.get_checked(id).map_err(|err| Inner(err))
    }

    pub fn get(&self, id: CellId) -> &Cell {
        self.get_checked(id).expect("Invalid CellId passed to Board::get")
    }

    pub unsafe fn get_unchecked(&self, id: CellId) -> &Cell {
        unsafe { self.inner.get_unchecked(id) }
    }

    pub fn set_checked(&mut self, mark: CellMark, id: CellId) -> Result<(), BoardError> {
        let cell = self.inner.get_checked(id).map_err(|err| Inner(err))?;

        if cell.val.is_some() {
            return Err(MarkOverwrite { mark: cell.val, other: mark, id });
        }

        unsafe {
            for branch in cell.branches.iter().flatten().copied().collect::<Vec<_>>() {
                self.inner.delete_branch_unchecked(id, branch);
            }

            self.inner.set_unchecked(id, Some(mark));
        }
        Ok(())
    }

    pub fn set(&mut self, mark: CellMark, id: CellId) {
        self.set_checked(mark, id).expect("Invalid CellId passed to Board::set");
    }

    pub unsafe fn set_unchecked(&mut self, mark: CellMark, id: CellId) {
        unsafe {
            for branch in self.inner
                .get_unchecked(id)
                .branches.iter()
                .enumerate()
                .filter_map(|(branch, branch_id)| branch_id.is_some().then_some(branch))
                .collect::<Vec<usize>>() {
                self.inner.delete_branch_unchecked(id, branch);
            }

            self.inner.set_unchecked(id, Some(mark));
        }
    }

    pub fn add_cell_checked(&mut self, val: Option<CellMark>, parent_id: CellId, cell_branch: CellBranch) -> Result<usize, BoardError> {
        let parent = self.inner.get_checked(parent_id).map_err(|err| Inner(err))?;
        if let Some(mark) = parent.val {
            return Err(MarkOverwrite { mark: None, other: mark, id: parent_id });
        }

        let branch = *cell_branch.get();
        
        unsafe {
            if parent.branches.get_unchecked(branch).is_some() {
                return Err(Inner(BranchOverwrite { id: parent_id, branch }));
            }
            
            Ok(self.inner.add_node_unchecked(val, parent_id, branch))
        }
    }
    
    pub fn add_cell(&mut self, val: Option<CellMark>, parent_id: CellId, cell_branch: CellBranch) -> usize {
        self.add_cell_checked(val, parent_id, cell_branch).expect("Invalid CellId passed to Board::add_cell")
    }
    
    pub unsafe fn add_cell_unchecked(&mut self, val: Option<CellMark>, parent_id: CellId, cell_branch: CellBranch) -> usize {
        unsafe {
            self.inner.add_node_unchecked(val, parent_id, *cell_branch.get())
        }
    }
    
    pub fn delete_branch_checked(&mut self, id: CellId, cell_branch: CellBranch) -> Result<(), BoardError> {
        let branch = *cell_branch.get();
        
        unsafe {
            match *self.inner.get_checked(id).map_err(|err| Inner(err))?.branches.get_unchecked(branch) {
                None => Err(Inner(BranchDoesNotExist { branch, id })),
                Some(_) => {
                    self.inner.delete_branch_unchecked(id, branch);
                    Ok(())
                },
            }
        }
    }
    
    pub fn delete_branch(&mut self, id: CellId, cell_branch: CellBranch) {
        self.delete_branch_checked(id, cell_branch).expect("Invalid CellId passed to Board::delete_branch")
    }
    
    pub unsafe fn delete_branch_unchecked(&mut self, id: CellId, cell_branch: CellBranch) {
        unsafe {
            self.inner.delete_branch_unchecked(id, *cell_branch.get())
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, overlay: Option<&Overlay>, frame_coords: &Coords) {
        let mut cursor = BoardCursor::from_coords(self, frame_coords);
        self.draw_recursive(&mut cursor, overlay, frame_coords, canvas);
    }

    fn draw_recursive(&self, cursor: &mut BoardCursor, overlay: Option<&Overlay>, frame_coords: &Coords, canvas: &mut Canvas) {
        let frame_depth = frame_coords.0.len();
        let depth = cursor.depth();
        let coords = &Coords(cursor.coords().0[frame_depth..].to_vec());

        if let Ok(cell) = cursor.get_checked() && let Some(mark) = cell.val {
            Mark::from(mark).draw(canvas, coords);
        } else if let Some(overlay) = overlay && let Some(mark) = overlay.get(cursor.coords()) {
            mark.draw(canvas, coords);
        } else if depth < *self.level.get() as usize {
            Grid.draw(canvas, coords);

            if depth < *canvas.level().get() as usize + frame_depth {
                for i in 0..9 {
                    unsafe {
                        cursor.move_to_child(CellBranch::new_unchecked(i));
                        self.draw_recursive(cursor, overlay, frame_coords, canvas);
                        cursor.move_to_parent_unchecked();
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    Inner(ArenaTreeError<CELLS_PER_GRID>),
    MarkOverwrite {
        mark: Option<CellMark>,
        other: CellMark,
        id: CellId,
    },
    NonZeroGhostDepth(usize),
}

impl Display for BoardError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Inner(err) => err.fmt(f),
            MarkOverwrite { mark, other, id } =>
                write!(f, "Mark {other} cannot overwrite {mark:?} at {id:?}"),
            NonZeroGhostDepth(ghost_depth) =>
                write!(f, "The ghost_depth is {ghost_depth} and must be 0"),
        }
    }
}

impl std::error::Error for BoardError {}