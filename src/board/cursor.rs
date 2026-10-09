use data_structures::arena_tree::{self, cursor};
use cursor::ArenaTreeCursor;
use super::mark::CellMark;
use super::{CELLS_PER_GRID, Board, CellId, Cell, CellBranch, BoardError, Coords};
use arena_tree::ArenaTreeError;

use CellId::*;
use BoardError::*;
use ArenaTreeError::*;

macro_rules! define_cursor {
    ($(#[$attr:meta])*, $name:ident $(, $m:ident)?) => {
        $(#[$attr])*
        pub struct $name<'a> {
            inner: ArenaTreeCursor<'a, Option<CellMark>, CELLS_PER_GRID>,
            board: &'a $($m)? Board,
            coords: Coords,
            ghost_depth: usize,
        }

        impl<'a> From<&'a $($m)? Board> for $name<'a> {
            fn from(board: &'a $($m)? Board) -> Self {
                Self {
                    inner: Default::default(),
                    board,
                    coords: Default::default(),
                    ghost_depth: Default::default(),
                }
            }
        }

        impl<'a> $name<'a> {
            pub fn from_coords(board: &'a $($m)? Board, coords: &Coords) -> Self {
                let mut cursor = Self::from(board);

                for &coord in &coords.0 {
                    cursor.move_to_child(coord);
                }

                cursor
            }

            pub fn id(&self) -> CellId {
                self.inner.id()
            }
            
            pub fn board(&self) -> &Board {
                self.board
            }

            pub fn coords(&self) -> &Coords {
                &self.coords
            }

            pub fn ghost_depth(&self) -> usize {
                self.ghost_depth
            }

            pub fn depth(&self) -> usize {
                self.coords.0.len()
            }

            pub fn invalid_id(&self) -> bool {
                self.ghost_depth != 0
            }

            pub fn get_checked(&self) -> Result<&Cell, BoardError> {
                if self.invalid_id() {
                    return Err(NonZeroGhostDepth(self.ghost_depth));
                }

                unsafe {
                    Ok(self.get_unchecked())
                }
            }

            pub fn get(&self) -> &Cell {
                self.get_checked().expect(concat!("Invalid state passed to ", stringify!($name), "::get"))
            }

            pub unsafe fn get_unchecked(&self) -> &Cell {
                unsafe {
                    self.inner.get_unchecked(&self.board.inner)
                }
            }

            pub fn move_to_child(&mut self, branch: CellBranch) {
                self.coords.0.push(branch);
                let branch = *branch.get();

                unsafe {
                    if let Ok(cell) = self.get_checked() && cell.branches.get_unchecked(branch).is_some() {
                        self.inner.move_to_child_unchecked(branch, &self.board.inner);
                    } else {
                        self.ghost_depth += 1;
                    }
                }
            }

            pub fn move_to_parent_checked(&mut self) -> Result<CellId, BoardError> {
                if !self.invalid_id() && self.id() == Root {
                    return Err(Inner(RootHasNoParent));
                }

                unsafe {
                    Ok(self.move_to_parent_unchecked())
                }
            }

            pub fn move_to_parent(&mut self) -> CellId {
                self.move_to_parent_checked().expect(concat!("Invalid state passed to ", stringify!($name),"::move_to_parent"))
            }

            pub unsafe fn move_to_parent_unchecked(&mut self) -> CellId {
                self.coords.0.pop();
                if self.invalid_id() {
                    self.ghost_depth -= 1;
                    self.inner.id()
                } else {
                    unsafe {
                        self.inner.move_to_parent_unchecked(&self.board.inner)
                    }
                }
            }
        }
    };
}

define_cursor!(#[derive(Debug, Clone, PartialEq, Eq)], BoardCursor);

define_cursor!(#[derive(Debug, PartialEq, Eq)], BoardCursorMut, mut);

impl<'a> BoardCursorMut<'a> {
    pub fn set(&mut self, mark: CellMark) {
        unsafe {
            self.board.set_unchecked(mark, self.id())
        }
    }

    pub fn add_cell_checked(&mut self, val: Option<CellMark>, cell_branch: CellBranch) -> Result<usize, BoardError> {
        unsafe {
            let invalid_id = self.invalid_id();

            let branches = if invalid_id {
                let branches = self.coords().0[self.depth() - self.ghost_depth()..].to_vec();

                while self.invalid_id() {
                    self.move_to_parent_unchecked();
                }

                Some(branches)
            } else { None };

            if let Some(mark) = self.get_unchecked().val {
                return Err(MarkOverwrite { mark: None, other: mark, id: self.id() });
            }

            if invalid_id {
                for branch in branches.unwrap() {
                    self.board.add_cell_unchecked(None, self.id(), branch);
                    self.move_to_child(branch);
                }
            }

            let branch = *cell_branch.get();
            if !invalid_id && self.get_unchecked().branches.get_unchecked(branch).is_some() {
                return Err(Inner(BranchOverwrite { id: self.id(), branch }));
            }

            Ok(self.board.add_cell_unchecked(val, self.id(), cell_branch))
        }
    }

    pub fn add_cell(&mut self, val: Option<CellMark>, cell_branch: CellBranch) -> usize {
        self.add_cell_checked(val, cell_branch).expect("Invalid state passed to BoardCursorMut::add_cell")
    }

    pub unsafe fn add_cell_unchecked(&mut self, val: Option<CellMark>, cell_branch: CellBranch) -> usize {
        unsafe {
            if self.invalid_id() {
                let branches = self.coords().0[self.depth() - self.ghost_depth()..].to_vec();

                while self.invalid_id() {
                    self.move_to_parent_unchecked();
                }

                for branch in branches {
                    self.board.add_cell_unchecked(None, self.id(), branch);
                    self.move_to_child(branch);
                }
            }

            self.board.add_cell_unchecked(val, self.id(), cell_branch)
        }
    }

    pub fn delete_branch_checked(&mut self, cell_branch: CellBranch) -> Result<(), BoardError> {
        let branch = *cell_branch.get();

        unsafe {
            match *self.get_checked()?.branches.get_unchecked(branch) {
                None => Err(Inner(BranchDoesNotExist { branch, id: self.id() })),
                Some(_) => {
                    self.delete_branch_unchecked(cell_branch);
                    Ok(())
                },
            }
        }
    }

    pub fn delete_branch(&mut self, cell_branch: CellBranch) {
        self.delete_branch_checked(cell_branch).expect("Invalid CellBranch passed to BoardCursorMut::delete_branch")
    }

    pub unsafe fn delete_branch_unchecked(&mut self, cell_branch: CellBranch) {
        unsafe {
            self.board.delete_branch_unchecked(self.id(), cell_branch)
        }
    }

    pub fn as_cursor(&self) -> BoardCursor<'_> {
        BoardCursor {
            inner: self.inner,
            board: self.board,
            coords: self.coords.clone(),
            ghost_depth: self.ghost_depth,
        }
    }
}