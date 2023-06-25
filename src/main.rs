use anyhow::Result;
use clap::Parser;
use crossbeam::channel;
use std::thread;
use window_manager::WindowManager;

mod args;
mod client;
mod commands;
mod config;
mod ewmh;
mod state;
mod vector;
mod window_manager;

fn main() -> Result<()> {
    let cli = args::Args::parse();
    match cli.command {
        Some(args::Commands::Start) => start(),
        Some(args::Commands::Client(command)) => {
            client::dispatch_command(command.into());
            Ok(())
        }
        _ => Ok(()),
    }
}

fn start() -> Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    // Initialize the client channel
    let (client_sender, client_receiver) = channel::unbounded();

    // Spawn the IPC thread
    thread::spawn(move || {
        client::handle_ipc(client_sender);
    });

    // Start the window manager
    let mut wm = WindowManager::new(conn, screen_num, client_receiver);
    wm.run()
}
