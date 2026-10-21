use data_structures::arena_tree::{self, cursor};
use cursor::ArenaTreeCursor;
use super::{mark, CELLS_PER_GRID, Board, CellId, Cell, CellBranch, BoardError, Coords, Result};
use mark::CellMark;
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

            pub fn get_checked(&self) -> Result<&Cell> {
                if self.invalid_id() {
                    return Err(NonZeroGhostDepth(self.ghost_depth));
                }

                Ok(self.inner.get(&self.board.inner))
            }

            pub fn get(&self) -> &Cell {
                #[cfg(debug_assertions)]
                {
                    self.get_checked().unwrap()
                }
                #[cfg(not(debug_assertions))]
                {
                    self.inner.get(&self.board.inner)
                }
            }

            pub fn move_to_child(&mut self, branch: CellBranch) {
                self.coords.0.push(branch);
                let branch = *branch.get();

                if let Ok(cell) = self.get_checked() && cell.get_branch(branch).is_some() {
                    self.inner.move_to_child(branch, &self.board.inner);
                } else {
                    self.ghost_depth += 1;
                }
            }

            pub fn move_to_parent_checked(&mut self) -> Result<CellId> {
                if !self.invalid_id() && self.id() == Root {
                    return Err(Inner(RootHasNoParent));
                }

                Ok(self.move_to_parent_internal())
            }

            pub fn move_to_parent(&mut self) -> CellId {
                #[cfg(debug_assertions)]
                {
                    self.move_to_parent_checked().unwrap()
                }
                #[cfg(not(debug_assertions))]
                {
                    self.move_to_parent_internal()
                }
            }

            fn move_to_parent_internal(&mut self) -> CellId {
                self.coords.0.pop();
                if self.invalid_id() {
                    self.ghost_depth -= 1;
                    self.inner.id()
                } else {
                    self.inner.move_to_parent(&self.board.inner)
                }
            }
        }
    };
}

define_cursor!(#[derive(Debug, Clone, PartialEq, Eq)], BoardCursor);

define_cursor!(#[derive(Debug, PartialEq, Eq)], BoardCursorMut, mut);

impl<'a> BoardCursorMut<'a> {
    pub fn set(&mut self, mark: CellMark) {
        self.board.set(mark, self.id())
    }

    pub fn add_cell_checked(&mut self, val: Option<CellMark>, cell_branch: CellBranch) -> Result<usize> {
        let invalid_id = self.invalid_id();

        let branches = if invalid_id {
            let branches = self.coords().0[self.depth() - self.ghost_depth()..].to_vec();

            while self.invalid_id() {
                self.move_to_parent();
            }

            Some(branches)
        } else { None };

        if let Some(mark) = self.get().val {
            return Err(MarkOverwrite { mark: None, other: mark, id: self.id() });
        }

        if invalid_id {
            for branch in branches.unwrap() {
                self.board.add_cell(None, self.id(), branch);
                self.move_to_child(branch);
            }
        }

        let branch = *cell_branch.get();
        if !invalid_id && self.get().get_branch(branch).is_some() {
            return Err(Inner(BranchOverwrite { id: self.id(), branch }));
        }

        Ok(self.board.add_cell(val, self.id(), cell_branch))
    }

    pub fn add_cell(&mut self, val: Option<CellMark>, cell_branch: CellBranch) -> usize {
        #[cfg(debug_assertions)]
        {
            self.add_cell_checked(val, cell_branch).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            if self.invalid_id() {
                let branches = self.coords().0[self.depth() - self.ghost_depth()..].to_vec();

                while self.invalid_id() {
                    self.move_to_parent();
                }

                for branch in branches {
                    self.board.add_cell(None, self.id(), branch);
                    self.move_to_child(branch);
                }
            }

            self.board.add_cell(val, self.id(), cell_branch)
        }
    }

    pub fn delete_branch_checked(&mut self, cell_branch: CellBranch) -> Result {
        let branch = *cell_branch.get();

        match *self.get_checked()?.get_branch(branch) {
            None => Err(Inner(BranchDoesNotExist { branch, id: self.id() })),
            Some(_) => {
                self.delete_branch(cell_branch);
                Ok(())
            },
        }
    }

    pub fn delete_branch(&mut self, cell_branch: CellBranch) {
        #[cfg(debug_assertions)]
        {
            self.delete_branch_checked(cell_branch).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            self.board.delete_branch(self.id(), cell_branch)
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