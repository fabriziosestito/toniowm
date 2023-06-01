use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Subcommand, Serialize, Deserialize)]
pub enum Command {
    Quit,
    FocusClosest { direction: Direction },
}

#[derive(ValueEnum, Clone, Serialize, Deserialize)]
pub enum Direction {
    East,
    West,
    North,
    South,
}
