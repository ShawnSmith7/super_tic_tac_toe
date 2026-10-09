use super_tic_tac_toe::game::GameManager;
use std::process;

fn main() {
    if let Err(err) = GameManager::run() {
        eprintln!("Err: {err}");
        process::exit(1);
    }
}