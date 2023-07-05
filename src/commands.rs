//! This module contains the commands which can be executed by the window manager.
//! A command represents the intent of the user to change the state of the wm.
//! From traits are implemented to convert from clap arguments to commands.

use serde::{Deserialize, Serialize};

use crate::args;

#[derive(Serialize, Deserialize)]
pub enum Command {
    Quit,
    FocusClosest {
        direction: Direction,
        selector: Selector,
    },
    Close {
        selector: Selector,
    },
}

#[derive(Serialize, Deserialize)]
pub enum Direction {
    East,
    West,
    North,
    South,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Selector {
    Focused,
    Window(u32),
}

impl From<args::Command> for Command {
    fn from(command: args::Command) -> Self {
        match command {
            args::Command::Quit => Self::Quit,
            args::Command::FocusClosest {
                direction,
                selector,
            } => Self::FocusClosest {
                direction: direction.into(),
                selector: selector.into(),
            },
            args::Command::Close { selector } => Self::Close {
                selector: selector.into(),
            },
        }
    }
}

impl From<args::Direction> for Direction {
    fn from(direction: args::Direction) -> Self {
        match direction {
            args::Direction::East => Self::East,
            args::Direction::West => Self::West,
            args::Direction::North => Self::North,
            args::Direction::South => Self::South,
        }
    }
}

impl From<args::Selector> for Selector {
    fn from(selector: args::Selector) -> Self {
        match selector {
            args::Selector {
                focused: true,
                window: None,
            } => Self::Focused,
            args::Selector {
                window: Some(window),
                ..
            } => {
                println!("window: {}", window);
                Self::Window(window)
            }
            // This is unreachable because the clap parser
            // will always return either a focused or a window.
            _ => unreachable!(),
        }
    }
}
