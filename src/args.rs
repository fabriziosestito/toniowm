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
    Start{
        ///Sets the path of the rc file
        #[clap(short, long, default_value = "~/.config/toniowm/toniorc")]
        autostart: String,
    },
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
    ActivateWorkspace {
        #[clap(flatten)]
        selector: WorkspaceSelector,
    },
    #[command(subcommand)]
    Config(Config),
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

#[derive(Subcommand)]
pub enum Config {
    #[clap(about = "Set the border width")]
    BorderWidth{
        #[clap(value_name = "BORDER_WIDTH")]
        width: u32,
    },
    #[clap(about = "Set the border color")]
    BorderColor{
        #[clap(value_name = "BORDER_COLOR")]
        color: u32,
    },
    #[clap(about = "Set the focused border color")]
    FocusedBorderColor{
        #[clap(value_name = "FOCUSED_BORDER_COLOR")]
        color: u32
    },
}



