# Super Tic-Tac-Toe

![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

A high-performance terminal strategy game written in Rust, featuring nested sub-board routing, a custom viewport manager, and standalone reusable library crates.

---

## Features

- **Nested Board Mechanics:** Multi-level tactical gameplay where moves in sub-boards determine opponent routing.
- **Dynamic Viewport Controls:** Zoom in and out across frames and canvas levels seamlessly.
- **Cargo Workspace Architecture:** Decoupled sub-crates allowing individual components (e.g., board logic, UI overlay) to be consumed independently in other projects.

---

## Install

Requires the [Rust toolchain](https://rustup.rs/) installed.

```bash
cargo install --git https://github.com/ShawnSmith7/super_tic_tac_toe.git
```

---

## Game Rules & Manual

### Definitions

- **Mark:** An 'X', 'O', or 'T' (representing a Tied board).
- **Cell:** A discrete space within a Board that holds either a Mark or a nested Sub-Board.
- **Board:** A 3x3 grid composed of nine Cells.
- **Level:** The depth layer of a Board. The "Top Level" is the root (shallowest) Board.
- **3-in-a-Row:** Three identical Marks ('X' or 'O') aligned horizontally, vertically, or diagonally.


### Gameplay Mechanics

1. **Starting Play:** Player X opens the game by placing a Mark in any Cell at the bottom-most Level.

2. **Turn Progression & Board Constraint:** Subsequent moves are constrained by the previous turn:
    - A player's move forces the opponent to play on the Board located one Level higher, specifically in the Board position corresponding to the Cell selected in the previous move.
    - Players alternate turns following this structural routing.

3. **Resolving Sub-Boards:**
    - Victory: Achieving a 3-in-a-Row on a Sub-Board claims that Board. A large Mark representing the winner is then placed one Level up in the Cell occupied by that Sub-Board.
    - Tie: If a Sub-Board fills completely without a 3-in-a-Row, it resolves as a Tie ('T') and becomes inactive.
    - Large Marks ('X', 'O', 'T') on higher-level Boards dictate turn routing identically to standard Cell Marks.

4. **Game Resolution:** The game ends when a player achieves a 3-in-a-Row on the Top-Level Board (Victory), or when no valid moves remain on the Top-Level Board (Draw).


### Exceptions and Edge Cases

- Inactive Board Route: If a move routes a player to a Sub-Board that has already
   been won or tied, that player is granted a \"free move\" and may play in any active
   Board one Level up.

- Top-Level Resolution Route: Winning a Board at the Top Level routes the opponent
    based on the specific Cell played during that winning move, rather than the
    overall Top-Level Board position.

---

## Command Reference

### Core Commands

- **[1-9]:** Select and move to the corresponding Cell (1-9).
- **toggle:** Toggle the move overlay visualization on or off.
- **help:** Display this rules and command reference screen.
- **forfeit:** Forfeit the current match.
- **quit:** Exit the game (progress is saved automatically).


### Viewport & Zoom Commands

- **zoom frame cursor:** Zoom the frame viewport directly to the cursor.
- **zoom frame in [1-9]:** Zoom the frame viewport into Cell [1-9].
- **zoom frame out:** Zoom the frame viewport out by one level.
- **zoom frame reset:** Reset the frame viewport to default zoom.
- **zoom canvas in:** Increase the overall canvas zoom level.
- **zoom canvas out:** Decrease the overall canvas zoom level.
- **zoom canvas reset:** Reset canvas zoom to default scale (Level 2).
- **zoom canvas set <N>:** Set canvas zoom to a specific level <N> (≥ 0).