use clap::{Parser, Subcommand, ValueEnum};

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


#[derive(Subcommand)]
pub enum Command {
    Quit,
    FocusClosest {
        direction: Direction,
        #[clap(flatten)]
        selector: WindowSelector,
    },
    Close {
        #[clap(flatten)]
        selector: WindowSelector,
    },
    AddWorkspace {
        #[clap(short, long)]
        name: Option<String>,
    },
    RenameWorkspace {
        #[clap(flatten)]
        selector: WorkspaceSelector,
        #[clap( value_name = "NEW_NAME" )]
        new_name: String,
    },
    SelectWorkspace {
        #[clap(flatten)]
        selector: WorkspaceSelector,
    },
}

#[derive(ValueEnum, Clone)]
pub enum Direction {
    East,
    West,
    North,
    South,
}

#[derive(clap::Args, Clone)]
#[group(multiple = false)]
pub struct WindowSelector {
    #[clap(long, short, default_value = "true")]
    pub focused: bool,

    #[clap(long, short)]
    pub window: Option<u32>,
}

#[derive(clap::Args, Clone)]
#[group(multiple = false, required = true)]
pub struct WorkspaceSelector {
    #[clap(long, short)]
    pub index: Option<usize>,

    #[clap(long, short)]
    pub name: Option<String>,
}
