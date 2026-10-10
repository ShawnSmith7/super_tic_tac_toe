# Multi-Level Terminal Strategy Game

![Rust](https://img.shields.io/badge/rust-2021-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

A high-performance terminal strategy game written in Rust, featuring nested sub-board routing, a custom viewport manager, and standalone reusable library crates.

---

## Features

- **Nested Board Mechanics:** Multi-level tactical gameplay where moves in sub-boards determine opponent routing.
- **Dynamic Viewport Controls:** Zoom in and out across frames and canvas levels seamlessly.
- **Cargo Workspace Architecture:** Decoupled subcrates allowing individual components (e.g., board logic, UI overlay) to be consumed independently in other projects.

---

## Installation & Quick Start

### Option 1: Install Directly via Cargo (Recommended)

Requires the [Rust toolchain](https://rustup.rs/) installed.

```bash
cargo install --git https://github.com/ShawnSmith7/super_tic_tac_toe.git
