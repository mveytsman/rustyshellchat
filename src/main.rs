#![allow(unused_imports)]
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

type ClientId = u16;

enum ClientMessage {
    Joined(ClientId, Sender<ServerMessage>),
    Left(ClientId),
    Message(ClientId, String),
}

enum ServerMessage {
    Joined(ClientId),
    Left(ClientId),
    Message(ClientId, String),
}

fn run_server() -> Sender<ClientMessage> {
    let (sender, receiver) = channel();
    thread::spawn(move || {
        let mut clients: HashMap<ClientId, Sender<ServerMessage>> = HashMap::new();
        for message in receiver {
            match message {
                ClientMessage::Joined(id, sender) => {
                    for sender in clients.values() {
                        sender.send(ServerMessage::Joined(id)).unwrap();
                    }
                    clients.insert(id, sender);
                    println!("Joined by {}", id);
                }
                ClientMessage::Left(id) => {
                    clients.remove(&id);
                    for sender in clients.values() {
                        sender.send(ServerMessage::Left(id)).unwrap();
                    }
                    println!("{} left", id);
                }
                ClientMessage::Message(sender_id, body) => {
                    for (id, sender) in clients.iter() {
                        println!("loop");
                        if *id != sender_id {
                            // don't send to the sender
                            sender
                                .send(ServerMessage::Message(sender_id, body.clone()))
                                .unwrap();
                        }
                    }
                    println!("{}: {}", sender_id, body)
                }
            }
        }
    });
    sender
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    let server_tx = run_server();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let server_tx = Sender::clone(&server_tx);
        thread::spawn(move || handle_client(stream, server_tx));
    }
}

fn handle_client(stream: TcpStream, server_sender: Sender<ClientMessage>) {
    let client_id = stream.peer_addr().unwrap().port();
    let (client_sender, client_receiver) = channel();
    let mut stream2 = stream.try_clone().unwrap();
    thread::spawn(move || {
        for message in client_receiver {
            match message {
                ServerMessage::Joined(id) => writeln!(&mut stream2, "Joined {}", id).unwrap(),

                ServerMessage::Left(id) => writeln!(&mut stream2, "{} left", id).unwrap(),

                ServerMessage::Message(id, body) => writeln!(stream2, "{}: {}", id, body).unwrap(),
            }
        }
    });
    server_sender
        .send(ClientMessage::Joined(
            client_id,
            Sender::clone(&client_sender),
        ))
        .unwrap();

    for line in BufReader::new(stream).lines() {
        server_sender
            .send(ClientMessage::Message(client_id, line.unwrap()))
            .unwrap();
    }
    server_sender.send(ClientMessage::Left(client_id)).unwrap();
}
