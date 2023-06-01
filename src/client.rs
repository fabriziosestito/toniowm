use std::{
    io::{BufReader, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
};

use crossbeam::channel;

use crate::commands::{self, Command};

pub fn handle_ipc(client_sender: channel::Sender<commands::Command>) {
    std::fs::remove_file("/tmp/toniowm.socket").unwrap_or_default();
    let listener = UnixListener::bind("/tmp/toniowm.socket").unwrap();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                let client_sender = client_sender.clone();
                thread::spawn(|| handle_client(stream, client_sender));
            }
            Err(err) => {
                /* connection failed */
                println!("Error: {}", err);
                break;
            }
        }
    }
}

fn handle_client(stream: UnixStream, client_sender: channel::Sender<commands::Command>) {
    let mut buf = BufReader::new(stream);

    let mut data = String::new();
    if let Err(err) = buf.read_to_string(&mut data) {
        eprintln!("Error: {}", err);
        return;
    }

    let command = match serde_json::from_str(&data) {
        Ok(command) => command,
        Err(_) => {
            eprintln!("Error: Invalid command");
            return;
        }
    };
    client_sender.send(command).unwrap();
}
// TODO: handle errors
pub fn dispatch_command(command: Command) {
    let socket = std::path::Path::new("/tmp/toniowm.socket");
    let mut stream = std::os::unix::net::UnixStream::connect(socket).unwrap();
    let serialized_command = serde_json::to_string(&command).unwrap();

    stream.write_all(serialized_command.as_bytes()).unwrap();
}
