use std::{io, error};
use terminal_tools::{clear_terminal, get_input};
use crate::{UserLevel, game};
use game::{Game, GameState};
use rand::RngExt;
use reqwest::blocking::Client;
use error::Error;

use GameState::*;

const FIREBASE_URL: &str = "https://super-tic-tac-toe-fbe39-default-rtdb.firebaseio.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameManager;

impl GameManager {
    pub fn run() -> io::Result<()> {
        let mut message: Option<String> = None;

        loop {
            clear_terminal();
            println!("\
                *****************\n\
                Super Tic Tac Toe\n\
                *****************\n\
                \n\
                1. New Game\n\
                2. Load Game\n\
                3. Quit\n"
            );

            if let Some(message) = &message {
                println!("{message}");
            }

            match get_input("Enter a Command: ")?.trim().to_lowercase().as_str() {
                "1" | "new game" => {
                    match get_input("Enter a Level: ")?.trim().parse::<UserLevel>() {
                        Ok(level) =>
                            Self::run_game(&mut Game::from(level), &mut message)?,
                        Err(err) =>
                            message = Some(format!("Invalid command: \"{}\"", err.0)),
                    };
                },
                "2" | "load game" => {
                    let code = get_input("Enter a game code: ")?;
                    let code = code.trim();

                    match Self::load(code) {
                        Ok(Some(ref mut game)) =>
                            Self::run_game(game, &mut message)?,
                        Ok(None) =>
                            message = Some(format!("No game found with code {}", code)),
                        Err(err) =>
                            message = Some(format!("Error loading game: {}", err)),
                    }
                },
                "3" | "quit" => break,
                invalid_command => {
                    message = Some(format!("Invalid command: \"{invalid_command}\""));
                },
            }
        }

        Ok(())
    }

    fn run_game(game: &mut Game, message: &mut Option<String>) -> io::Result<()> {
        game.run()?;

        if game.state == InProgress {
            *message = Some(match Self::save(&game) {
                Ok(code) =>
                    format!("Game saved!\nEnter this code to resume game: {code}"),
                Err(err) =>
                    format!("Failed to save game: {err}"),
            });
        }

        Ok(())
    }

    fn gen_code() -> String {
        let mut rng = rand::rng();
        let code: u32 = rng.random_range(1000..=9999);
        code.to_string()
    }

    fn gen_unique_code(client: &Client) -> String {
        loop {
            let code = Self::gen_code();
            let url = format!("{}/games/{}.json", FIREBASE_URL, code);

            if let Ok(res) = client.get(&url).send() {
                if let Ok(text) = res.text() {
                    if text == "null" {
                        return code;
                    }
                }
            }
        }
    }

    pub fn save(game: &Game) -> Result<String, Box<dyn Error>> {
        let client = Client::new();

        let code = Self::gen_unique_code(&client);
        let url = format!("{}/games/{}.json", FIREBASE_URL, code);

        client.put(&url)
            .json(game)
            .send()?;

        Ok(code)
    }

    pub fn load(code: &str) -> Result<Option<Game>, Box<dyn Error>> {
        let url = format!("{}/games/{}.json", FIREBASE_URL, code);
        let client = Client::new();

        let response = client.get(&url).send()?;
        let game: Option<Game> = response.json()?;

        if game.is_some() {
            client.delete(&url).send()?;
        }

        Ok(game)
    }
}