use clap::{Parser, Subcommand};

use crate::commands::Command;

#[derive(Parser)]
#[command(author, 
    version, 
    about, 
    long_about = None,     
    propagate_version = true,
    subcommand_required = true,
    arg_required_else_help = true,
)]
pub struct Args {
    // /// Optional name to operate on
    // name: Option<String>,

    // /// Sets a custom config file
    // #[arg(short, long, value_name = "FILE")]
    // config: Option<PathBuf>,

    // /// Turn debugging information on
    // #[arg(short, long, action = clap::ArgAction::Count)]
    // debug: u8,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the window manager
    Start,
    /// Send a command to the window manager
    #[command(subcommand)]
    Client(Command),
}

