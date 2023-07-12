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
        selector: WindowSelector,
    },
    Close {
        selector: WindowSelector,
    },
    SelectWorkspace {
        selector: WorkspaceSelector,
    },
}

#[derive(Serialize, Deserialize)]
pub enum Direction {
    East,
    West,
    North,
    South,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WindowSelector {
    Focused,
    Window(u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkspaceSelector {
    Index(usize),
    Name(String),
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
            args::Command::SelectWorkspace { selector } => Self::SelectWorkspace {
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

impl From<args::WindowSelector> for WindowSelector {
    fn from(selector: args::WindowSelector) -> Self {
        match selector {
            args::WindowSelector {
                focused: true,
                window: None,
            } => Self::Focused,
            args::WindowSelector {
                window: Some(window),
                ..
            } => Self::Window(window),
            // This is unreachable because the clap parser
            // will always return either a focused or a window.
            _ => unreachable!(),
        }
    }
}

impl From<args::WorkspaceSelector> for WorkspaceSelector {
    fn from(selector: args::WorkspaceSelector) -> Self {
        match selector {
            args::WorkspaceSelector {
                index: Some(index),
                name: None,
            } => Self::Index(index),
            args::WorkspaceSelector {
                name: Some(name),
                index: None,
            } => Self::Name(name),
            // This is unreachable because the clap parser
            // will always return either a focused or a window.
            _ => unreachable!(),
        }
    }
}
