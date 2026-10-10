pub mod command;
mod game_manager;

pub use game_manager::GameManager;

use super::{Level, UserLevel, board, canvas};
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

impl From<UserLevel> for Game {
    fn from(level: UserLevel) -> Self {
        Self {
            state: InProgress,
            turn: X,
            coords: Default::default(),
            board: Board::from(level),
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

            if data.process_command(command, &mut self.turn, &mut self.state)?.0 {
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

    fn process_command(&mut self, command: Command, turn: &mut PlayerMark, state: &mut GameState) -> io::Result<Break> {
        match command {
            Command::Move(branch) => {
                self.make_move(branch, turn, state);
                return Ok(Break(false))
            },
            Command::Toggle => self.has_overlay = !self.has_overlay,
            Command::Zoom(target) => self.zoom(target),
            Command::Quit => return Ok(Break(true)),
            Command::Forfeit => {
                *state = Forfeit;
                self.has_overlay = false;
            },
            Command::Help => help_screen()?,
        }

        self.message = None;
        Ok(Break(false))
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
            if self.cursor.depth() + 1 == *self.cursor.board().level().get() as usize {
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
                if self.frame_coords.0.len() + 1 >= *self.cursor.board().level().get() as usize
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
                if let Some(level) = self.canvas.level()
                    .get().checked_add(1)
                    .map(|level| unsafe { Level::new_unchecked(level) }) {
                    self.canvas = Canvas::from(level);
                } else {
                    self.message = Some(String::from("Cannot zoom canvas in any further"));
                },
            Canvas(ZoomCanvasAction::Out) =>
                if let Some(level) = self.canvas.level()
                    .get().checked_sub(1)
                    .map(|level| unsafe { Level::new_unchecked(level) }) {
                    self.canvas = Canvas::from(level);
                } else {
                    self.message = Some(String::from("Cannot zoom canvas out any further"));
                },
            Canvas(ZoomCanvasAction::Reset) =>
                self.canvas = Canvas::from(unsafe { Level::new_unchecked(2) }),
            Canvas(ZoomCanvasAction::Set(level)) =>
                self.canvas = Canvas::from(level),
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

pub fn help_screen() -> io::Result<()> {
    clear_terminal();
    println!(
"================================================================================
                               GAME RULES & MANUAL
================================================================================

[ DEFINITIONS ]

  Mark              An 'X', 'O', or 'T' (representing a Tied board).
  Cell              A discrete space within a Board that holds either a Mark
                    or a nested Sub-Board.
  Board             A 3x3 grid composed of nine Cells.
  Level             The depth layer of a Board. The \"Top Level\" is the root
                    (shallowest) Board.
  3-in-a-Row        Three identical Marks ('X' or 'O') aligned horizontally,
                    vertically, or diagonally.


[ GAMEPLAY MECHANICS ]

1. Starting Play
   Player X opens the game by placing a Mark in any Cell at the bottom-most Level.

2. Turn Progression & Board Constraint
   Subsequent moves are constrained by the previous turn:
   - A player's move forces the opponent to play on the Board located one Level
     higher, specifically in the Board position corresponding to the Cell selected
     in the previous move.
   - Players alternate turns following this structural routing.

3. Resolving Sub-Boards
   - Victory: Achieving a 3-in-a-Row on a Sub-Board claims that Board. A large
     Mark representing the winner is then placed one Level up in the Cell
     occupied by that Sub-Board.
   - Tie: If a Sub-Board fills completely without a 3-in-a-Row, it resolves as a
     Tie ('T') and becomes inactive.
   - Large Marks ('X', 'O', 'T') on higher-level Boards dictate turn routing
     identically to standard Cell Marks.

4. Game Resolution
   The game ends when a player achieves a 3-in-a-Row on the Top-Level Board (Victory),
   or when no valid moves remain on the Top-Level Board (Draw).


[ EXCEPTIONS & EDGE CASES ]

• Inactive Board Route: If a move routes a player to a Sub-Board that has already
  been won or tied, that player is granted a \"free move\" and may play in any active
  Board one Level up.

• Top-Level Resolution Route: Winning a Board at the Top Level routes the opponent
  based on the specific Cell played during that winning move, rather than the
  overall Top-Level Board position.


================================================================================
                                COMMAND REFERENCE
================================================================================

[ CORE COMMANDS ]

  [1-9]             Select and move to the corresponding Cell (1-9).
  toggle            Toggle the move overlay visualization on or off.
  help              Display this rules and command reference screen.
  forfeit           Forfeit the current match.
  quit              Exit the game (progress is saved automatically).


[ VIEWPORT & ZOOM COMMANDS ]

  zoom frame cursor               Zoom the frame viewport directly to the cursor.
  zoom frame in [1-9]             Zoom the frame viewport into Cell [1-9].
  zoom frame out                  Zoom the frame viewport out by one level.
  zoom frame reset                Reset the frame viewport to default zoom.

  zoom canvas in                  Increase the overall canvas zoom level.
  zoom canvas out                 Decrease the overall canvas zoom level.
  zoom canvas reset               Reset canvas zoom to default scale (Level 2).
  zoom canvas set <N>             Set canvas zoom to a specific level <N> (≥ 0)."
    );
    get_input("Click ENTER to continue: ")?;
    Ok(())
}