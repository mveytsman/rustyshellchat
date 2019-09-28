use std::net::TcpListener;
use std::sync::mpsc::Sender;
use std::thread;

use rustyshellchat::{handle_client, run_server};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    let server_tx = run_server();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let server_tx = Sender::clone(&server_tx);
        thread::spawn(move || handle_client(stream, server_tx));
    }
}
