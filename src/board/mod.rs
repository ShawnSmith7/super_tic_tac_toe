pub mod mark;
mod coords;
pub mod cursor;
mod overlay;

pub use coords::Coords;
pub use overlay::Overlay;

use bounded_value::BoundedValue;
use math_tools::{bounded_value, define_bounds};
use data_structures::arena_tree::{ArenaTree, ArenaTreeError, Node};
use std::{fmt, ops, result};
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
pub type Result<T = ()> = result::Result<T, BoardError>;

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

    pub fn get_checked(&self, id: CellId) -> Result<&Cell> {
        self.inner.get_checked(id).map_err(Inner)
    }

    pub fn get(&self, id: CellId) -> &Cell {
        self.inner.get(id)
    }

    pub fn set_checked(&mut self, mark: CellMark, id: CellId) -> Result {
        let cell = self.get_checked(id)?;

        if let Some(existing) = cell.val {
            return Err(MarkOverwrite {
                mark: Some(existing),
                other: mark,
                id,
            });
        }

        self.set_internal(mark, id);
        Ok(())
    }

    pub fn set(&mut self, mark: CellMark, id: CellId) {
        #[cfg(debug_assertions)]
        {
            self.set_checked(mark, id).unwrap();
        }
        #[cfg(not(debug_assertions))]
        {
            self.set_internal(mark, id);
        }
    }

    fn set_internal(&mut self, mark: CellMark, id: CellId) {
        let branches = self.get(id).branches;

        for (branch_idx, child_id) in branches.iter().enumerate() {
            if child_id.is_some() {
                self.delete_branch(id, CellBranch::new(branch_idx));
            }
        }

        self.inner.set(id, Some(mark));
    }

    pub fn add_cell_checked(&mut self, val: Option<CellMark>, parent_id: CellId, cell_branch: CellBranch) -> Result<usize> {
        let parent = self.get_checked(parent_id)?;
        if let Some(mark) = parent.val {
            return Err(MarkOverwrite { mark: None, other: mark, id: parent_id });
        }

        let branch = *cell_branch.get();
        if parent.get_branch(branch).is_some() {
            return Err(Inner(BranchOverwrite { id: parent_id, branch }));
        }

        Ok(self.inner.add_node(val, parent_id, branch))
    }
    
    pub fn add_cell(&mut self, val: Option<CellMark>, parent_id: CellId, cell_branch: CellBranch) -> usize {
        #[cfg(debug_assertions)]
        {
            self.add_cell_checked(val, parent_id, cell_branch).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            self.inner.add_node(val, parent_id, *cell_branch.get())
        }
    }
    
    pub fn delete_branch_checked(&mut self, id: CellId, cell_branch: CellBranch) -> Result {
        let branch = *cell_branch.get();

        match *self.get_checked(id)?.get_branch(branch) {
            None => Err(Inner(BranchDoesNotExist { branch, id })),
            Some(_) => {
                self.inner.delete_branch(id, branch);
                Ok(())
            },
        }
    }
    
    pub fn delete_branch(&mut self, id: CellId, cell_branch: CellBranch) {
        #[cfg(debug_assertions)]
        {
            self.delete_branch_checked(id, cell_branch).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            self.inner.delete_branch(id, *cell_branch.get())
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
                    cursor.move_to_child(CellBranch::new(i));
                    self.draw_recursive(cursor, overlay, frame_coords, canvas);
                    cursor.move_to_parent();
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