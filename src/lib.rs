use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
type ClientId = u16;

pub enum ClientMessage {
    Joined(ClientId, Sender<ServerMessage>),
    Left(ClientId),
    Message(ClientId, String),
}

pub enum ServerMessage {
    Joined(ClientId),
    Left(ClientId),
    Message(ClientId, String),
}

pub fn run_server() -> Sender<ClientMessage> {
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

pub fn handle_client(stream: TcpStream, server_sender: Sender<ClientMessage>) {
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
