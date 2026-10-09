use std::{fmt, io};
use std::fmt::{Display, Formatter};
use io::Write;

pub mod canvas;

pub fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = "\x1B");
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParseError(pub String);

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse string: \"{}\"", self.0)
    }
}

impl std::error::Error for ParseError {}

pub fn get_input(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input)
}