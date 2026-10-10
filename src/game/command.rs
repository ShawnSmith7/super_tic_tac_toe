use crate::{board, Level};
use board::{CellBranch, UserCellBranch};
use std::str::FromStr;
use terminal_tools::ParseError;

use Command::*;
use ZoomTarget::*;

macro_rules! impl_from_str {
    ($name:ident) => {
        impl FromStr for $name {
            type Err = ParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut tokens = s.split_whitespace();
                let target = Self::parse_from_tokens(&mut tokens, s)?;

                if tokens.next().is_some() {
                    return Err(ParseError(s.to_string()));
                }
                Ok(target)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Move(CellBranch),
    Toggle,
    Zoom(ZoomTarget),
    Quit,
    Forfeit,
    Help,
}

impl FromStr for Command {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseError(s.to_string());
        let mut tokens = s.split_whitespace();

        let command = match tokens.next().ok_or_else(err)?.to_lowercase().as_str() {
            "toggle" => Toggle,
            "zoom" => Zoom(ZoomTarget::parse_from_tokens(&mut tokens, s)?),
            "quit" => Quit,
            "forfeit" => Forfeit,
            "help" => Help,
            branch => Move(CellBranch::try_from(
                &branch.parse::<UserCellBranch>().map_err(|_| err())?
            ).unwrap()),
        };

        if tokens.next().is_some() {
            return Err(err());
        }

        Ok(command)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomTarget {
    Frame(ZoomFrameAction),
    Canvas(ZoomCanvasAction),
}

impl_from_str!(ZoomTarget);

impl ZoomTarget {
    fn parse_from_tokens<'a, I>(tokens: &mut I, s: &str) -> Result<Self, ParseError>
    where
        I: Iterator<Item = &'a str>,
    {
        let err = || ParseError(s.to_string());

        match tokens.next().ok_or_else(err)?.to_lowercase().as_str() {
            "frame" => Ok(Frame(ZoomFrameAction::parse_from_tokens(tokens, s)?)),
            "canvas" => Ok(Canvas(ZoomCanvasAction::parse_from_tokens(tokens, s)?)),
            _ => Err(err()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomFrameAction {
    Cursor,
    In(CellBranch),
    Out,
    Reset,
}

impl_from_str!(ZoomFrameAction);

impl ZoomFrameAction {
    fn parse_from_tokens<'a, I>(tokens: &mut I, s: &str) -> Result<Self, ParseError>
    where
        I: Iterator<Item = &'a str>,
    {
        let err = || ParseError(s.to_string());

        match tokens.next().ok_or_else(err)?.to_lowercase().as_str() {
            "cursor" => Ok(ZoomFrameAction::Cursor),
            "out" => Ok(ZoomFrameAction::Out),
            "reset" => Ok(ZoomFrameAction::Reset),
            branch => Ok(ZoomFrameAction::In(CellBranch::try_from(
                &branch.parse::<UserCellBranch>().map_err(|_| err())?
            ).unwrap())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomCanvasAction {
    In,
    Out,
    Reset,
    Set(Level),
}

impl_from_str!(ZoomCanvasAction);

impl ZoomCanvasAction {
    fn parse_from_tokens<'a, I>(tokens: &mut I, s: &str) -> Result<Self, ParseError>
    where
        I: Iterator<Item = &'a str>,
    {
        let err = || ParseError(s.to_string());

        match tokens.next().ok_or_else(err)?.to_lowercase().as_str() {
            "in" => Ok(ZoomCanvasAction::In),
            "out" => Ok(ZoomCanvasAction::Out),
            "reset" => Ok(ZoomCanvasAction::Reset),
            level => Ok(ZoomCanvasAction::Set(
                level.parse().map_err(|_| err())?
            )),
        }
    }
}