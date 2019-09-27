#![allow(unused_imports)]
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

struct Clients {
    clients: HashMap<u16, Sender<ServerMessage>>,
}

enum ClientMessage {
    Joined(u16, Sender<ServerMessage>),
    Left(u16),
}

type ServerMessage = ClientMessage; // for now

impl Clients {
    fn new() -> Clients {
        Clients {
            clients: HashMap::new(),
        }
    }
    fn add(&mut self, id: u16, channel: Sender<ServerMessage>) {
        self.clients.insert(id, channel);
    }
    fn remove(&mut self, id: &u16) {
        self.clients.remove(id).unwrap();
    }
}

fn main() {
    let (server_tx, server_rx) = channel();
    thread::spawn(move || server(server_rx));
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let server_tx = Sender::clone(&server_tx);
        thread::spawn(move || handle_client(stream, server_tx));
    }
}

fn server(receiver: Receiver<ClientMessage>) {
    let mut clients = Clients::new();
    for message in receiver {
        match message {
            ClientMessage::Joined(id, sender) => {
                clients.add(id, sender);
                println!("Joined by {}", id);
            }
            ClientMessage::Left(id) => {
                clients.remove(&id);
                println!("{} left", id),
            }
        }
    }
}

fn handle_client(stream: TcpStream, server_sender: Sender<ClientMessage>) {
    let client_id = stream.peer_addr().unwrap().port();
    let (client_sender, client_receiver) = channel();
    server_sender
        .send(ClientMessage::Joined(
            client_id,
            Sender::clone(&client_sender),
        ))
        .unwrap();

    for l in BufReader::new(stream).lines() {
        println!("{}", l.unwrap());
    }
    server_sender.send(ClientMessage::Left(client_id)).unwrap();
}

//     {
//     let (tx, rx) = channel();
//     let mut handles = vec![];

//     for _ in 1..10 {
//         let tx = Sender::clone(&tx);
//         let handle = thread::spawn(move || {
//             tx.send("Hello").unwrap();
//         });
//         handles.push(handle);
//     }

//     thread::spawn(|| {
//         for received in rx {
//             println!("Got: {}", received);
//         }
//     });

//     for handle in handles {
//         handle.join().unwrap()
//     }
// }
