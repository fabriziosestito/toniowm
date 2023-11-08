//! This module contains the commands which can be executed by the window manager.
//! A command represents the intent of the user to change the state of the wm.
//! From traits are implemented to convert from clap arguments to commands.

use serde::{Deserialize, Serialize};

use crate::args;

#[derive(Serialize, Deserialize)]
pub enum Command {
    Quit,
    Focus {
        selector: WindowSelector,
    },
    Close {
        selector: WindowSelector,
    },
    AddWorkspace {
        name: Option<String>,
    },
    RenameWorkspace {
        selector: WorkspaceSelector,
        name: String,
    },
    ActivateWorkspace {
        selector: WorkspaceSelector,
    },
    SetBorderWidth {
        width: u32,
    },
    SetBorderColor {
        color: u32,
    },
    SetFocusedBorderColor {
        color: u32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CardinalDirection {
    East,
    West,
    North,
    South,
}

impl From<args::CardinalDirection> for CardinalDirection {
    fn from(direction: args::CardinalDirection) -> Self {
        match direction {
            args::CardinalDirection::East => Self::East,
            args::CardinalDirection::West => Self::West,
            args::CardinalDirection::North => Self::North,
            args::CardinalDirection::South => Self::South,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CycleDirection {
    Next,
    Prev,
}

impl From<args::CycleDirection> for CycleDirection {
    fn from(direction: args::CycleDirection) -> Self {
        match direction {
            args::CycleDirection::Next => Self::Next,
            args::CycleDirection::Prev => Self::Prev,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WindowSelector {
    Focused,
    Window(u32),
    Closest(CardinalDirection),
    Cycle(CycleDirection),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkspaceSelector {
    Index(usize),
    Name(String),
    Cycle(CycleDirection),
}

impl From<args::Command> for Command {
    fn from(command: args::Command) -> Self {
        match command {
            args::Command::Quit => Self::Quit,
            args::Command::Focus { selector } => Self::Focus {
                selector: selector.into(),
            },
            args::Command::Close { selector } => Self::Close {
                selector: selector.into(),
            },
            args::Command::AddWorkspace { name } => Self::AddWorkspace { name },
            args::Command::RenameWorkspace {
                selector,
                new_name: name,
            } => Self::RenameWorkspace {
                selector: selector.into(),
                name,
            },
            args::Command::ActivateWorkspace { selector } => Self::ActivateWorkspace {
                selector: selector.into(),
            },
            args::Command::Config(args::Config::BorderWidth { width }) => {
                Self::SetBorderWidth { width }
            }
            args::Command::Config(args::Config::BorderColor { color }) => {
                Self::SetBorderColor { color }
            }
            args::Command::Config(args::Config::FocusedBorderColor { color }) => {
                Self::SetFocusedBorderColor { color }
            }
        }
    }
}

impl From<args::WindowSelector> for WindowSelector {
    fn from(selector: args::WindowSelector) -> Self {
        match selector {
            args::WindowSelector {
                focused: true,
                window: None,
                closest: None,
                cycle: None,
            } => Self::Focused,
            args::WindowSelector {
                window: Some(window),
                ..
            } => Self::Window(window),
            args::WindowSelector {
                closest: Some(direction),
                ..
            } => Self::Closest(direction.into()),
            args::WindowSelector {
                cycle: Some(direction),
                ..
            } => Self::Cycle(direction.into()),
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
                cycle: None,
            } => Self::Index(index),
            args::WorkspaceSelector {
                name: Some(name),
                index: None,
                cycle: None,
            } => Self::Name(name),
            args::WorkspaceSelector {
                cycle: Some(direction),
                ..
            } => Self::Cycle(direction.into()),
            // This is unreachable because the clap parser
            // will always return either a focused or a window.
            _ => unreachable!(),
        }
    }
}
