pub mod command;
mod game_manager;

pub use game_manager::GameManager;

use super::{Degree, UserDegree, board, canvas};
use board::{Board, CellBranch, CellId, Coords, Overlay, cursor, mark};
use canvas::Canvas;
use command::*;
use cursor::{BoardCursor, BoardCursorMut};
use mark::{CellMark, PlayerMark};
use serde::{Deserialize, Serialize};
use std::{io, str};
use terminal_tools::{clear_terminal, get_input};

use CellId::*;
use CellMark::*;
use PlayerMark::*;
use ZoomTarget::*;
use GameState::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    state: GameState,
    turn: PlayerMark,
    #[serde(default)]
    coords: Coords,
    board: Board,
}

impl From<UserDegree> for Game {
    fn from(degree: UserDegree) -> Self {
        Self {
            state: InProgress,
            turn: X,
            coords: Default::default(),
            board: Board::from(degree),
        }
    }
}

impl Game {
    pub fn run(&mut self) -> io::Result<()> {
        let mut data = RuntimeData {
            canvas: Canvas::default(),
            cursor: BoardCursorMut::from_coords(&mut self.board, &self.coords),
            message: None,
            has_overlay: self.state == InProgress,
            frame_coords: Coords::default(),
        };

        loop {
            data.prepare_frame();
            data.print_frame(self.turn, self.state);

            let Some(command) = data.get_command(self.state)? else {
                continue;
            };

            if data.process_command(command, &mut self.turn, &mut self.state).0 {
                break;
            }
        }

        self.coords = data.cursor.coords().clone();
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RuntimeData<'a> {
    canvas: Canvas,
    cursor: BoardCursorMut<'a>,
    message: Option<String>,
    has_overlay: bool,
    frame_coords: Coords,
}

impl<'a> RuntimeData<'a> {
    fn prepare_frame(&mut self) {
        let overlay = if self.has_overlay { Some(&Overlay::from(self.cursor.coords())) } else { None };
        self.canvas.clear();
        self.cursor.board().draw(&mut self.canvas, overlay, &self.frame_coords);
        clear_terminal();
    }

    fn print_frame(&self, turn: PlayerMark, state: GameState) {
        if state == InProgress {
            println!(
                "Turn: {}\n\
                Cursor Coords: {}",
                turn, self.cursor.coords()
            );
        }

        println!(
            "Frame Coords: {}\n\
            {}\n",
            self.frame_coords, self.canvas
        );

        match state {
            InProgress => (),
            Won => println!("{turn} is the winner!"),
            Forfeit => println!("{} won by forfeit!", turn.not()),
            Tie => println!("You tied!"),
        }

        if let Some(message) = &self.message {
            println!("{message}");
        }
    }

    fn get_command(&mut self, state: GameState) -> io::Result<Option<Command>> {
        match get_input("Enter a Command: ")?.trim().parse::<Command>() {
            Ok(Command::Move(_)) if state != InProgress => {
                self.message = Some(String::from("Cannot make move after win"));
                Ok(None)
            },
            Ok(Command::Toggle) if state != InProgress => {
                self.message = Some(String::from("Cannot toggle overlay after win"));
                Ok(None)
            },
            Ok(command) => Ok(Some(command)),
            Err(err) => {
                self.message = Some(format!("Invalid command: \"{}\"", err.0));
                Ok(None)
            },
        }
    }

    fn process_command(&mut self, command: Command, turn: &mut PlayerMark, state: &mut GameState) -> Break {
        match command {
            Command::Move(branch) => {
                self.make_move(branch, turn, state);
                return Break(false)
            },
            Command::Toggle => self.has_overlay = !self.has_overlay,
            Command::Zoom(target) => self.zoom(target),
            Command::Quit => return Break(true),
            Command::Forfeit => {
                *state = Forfeit;
                self.has_overlay = false;
            },
        }

        self.message = None;
        Break(false)
    }

    fn make_move(&mut self, branch: CellBranch, turn: &mut PlayerMark, state: &mut GameState) -> Break {
        unsafe {
            self.cursor.move_to_child(branch);
            if self.cursor.get_checked().map(|cell| cell.val.is_some()).unwrap_or(false) {
                self.cursor.move_to_parent_unchecked();
                self.message = Some(String::from("You cannot move where a mark already exists"));
                return Break(true);
            }
            self.cursor.move_to_parent_unchecked();

            let mut branch = branch;
            if self.cursor.depth() + 1 == *self.cursor.board().degree().get() as usize {
                self.cursor.add_cell_unchecked(Some(CellMark::from(*turn)), branch);

                let mut prev_branch = branch;
                while let Some(mark) = self.check_win() {
                    prev_branch = branch;
                    branch = match self.cursor.coords().0.last() {
                        Some(branch) => *branch,
                        None => {
                            *state = if mark == T {
                                Tie
                            } else {
                                Won
                            };

                            self.has_overlay = false;
                            return Break(false);
                        },
                    };
                    self.cursor.set(mark);
                    self.cursor.move_to_parent_unchecked();
                }

                if self.cursor.id() == Root {
                    branch = prev_branch;
                } else {
                    self.cursor.move_to_parent_unchecked();
                }

                *turn = turn.not();
            }

            self.cursor.move_to_child(branch);
            if self.cursor.get_checked().map(|cell| cell.val.is_some()).unwrap_or(false) {
                self.cursor.move_to_parent_unchecked();
            }

            Break(false)
        }
    }

    fn check_win(&self) -> Option<CellMark> {
        let mut cursor = self.cursor.as_cursor();
        let mut marks: [Option<CellMark>; 9] = Default::default();

        for i in 0..9 {
            unsafe {
                cursor.move_to_child(CellBranch::new_unchecked(i));
                marks[i] = cursor.get_checked().map(|cell| cell.val).unwrap_or(None);
                cursor.move_to_parent_unchecked();
            }
        }

        for i in 0..3 {
            let base = 3 * i;
            let mark = marks[base];
            if let Some(Player(_)) = mark && mark == marks[base + 1] && mark == marks[base + 2] {
                return marks[3 * i];
            }

            let mark = marks[i];
            if let Some(Player(_)) = mark && mark == marks[i + 3] && mark == marks[i + 6] {
                return marks[i];
            }
        }

        let mid = marks[4];
        if let Some(Player(_)) = mid && ((mid == marks[0] && mid == marks[8]) || (mid == marks[2] && mid == marks[6])) {
            return mid;
        }

        if marks.iter().all(|mark| mark.is_some()) {
            return Some(T)
        }

        None
    }

    fn zoom(&mut self, target: ZoomTarget) {
        match target {
            Frame(ZoomFrameAction::Cursor) => {
                self.frame_coords = self.cursor.coords().clone();
                self.frame_coords.0.pop();
            },
            Frame(ZoomFrameAction::In(branch)) => {
                if self.frame_coords.0.len() + 1 >= *self.cursor.board().degree().get() as usize
                    || BoardCursor::from_coords(self.cursor.board(), &self.frame_coords)
                    .get_checked()
                    .is_ok_and(|cell| cell.val.is_some()) {
                    self.message = Some(String::from("Cannot zoom frame in any further"));
                }

                self.frame_coords.0.push(branch);
            },
            Frame(ZoomFrameAction::Out) =>
                if self.frame_coords.0.pop().is_none() {
                    self.message = Some(String::from("Cannot zoom frame out any further"));
                },
            Frame(ZoomFrameAction::Reset) =>
                self.frame_coords = Coords::default(),
            Canvas(ZoomCanvasAction::In) =>
                if let Some(degree) = self.canvas.degree()
                    .get().checked_add(1)
                    .map(|degree| unsafe { Degree::new_unchecked(degree) }) {
                    self.canvas = Canvas::from(degree);
                } else {
                    self.message = Some(String::from("Cannot zoom canvas in any further"));
                },
            Canvas(ZoomCanvasAction::Out) =>
                if let Some(degree) = self.canvas.degree()
                    .get().checked_sub(1)
                    .map(|degree| unsafe { Degree::new_unchecked(degree) }) {
                    self.canvas = Canvas::from(degree);
                } else {
                    self.message = Some(String::from("Cannot zoom canvas out any further"));
                },
            Canvas(ZoomCanvasAction::Reset) =>
                self.canvas = Canvas::from(unsafe { Degree::new_unchecked(2) }),
            Canvas(ZoomCanvasAction::Set(degree)) =>
                self.canvas = Canvas::from(degree),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Break(bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum GameState {
    InProgress,
    Won,
    Forfeit,
    Tie,
}