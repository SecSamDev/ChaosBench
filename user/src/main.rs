use std::{io::{stdin, IsTerminal, Read}, net::TcpStream, sync::mpsc::{sync_channel, SyncSender, Receiver}};

use chaos_core::api::user_actions::UserAction;
use websocket::{sync::Client, ws::{self, Message}, OwnedMessage, WebSocketError};

const SERVER_ADDRESS : &str = env!("AGENT_SERVER_ADDRESS");

fn main() {
    let route = format!("ws://{}/user/connect",SERVER_ADDRESS);
    let mut client = websocket::ClientBuilder::new(&route).unwrap().connect_insecure().unwrap();
    client.set_nonblocking(true).unwrap();
    println!("Connected to: {}", SERVER_ADDRESS);
    let (sender, receiver) = sync_channel(32);
    read_commands(sender);
    loop {
        if let Err(e) = process_message(&mut client) {
            println!("Error: {:?}", e);
            return;
        };
        if let Err(e) = send_message(&mut client, &receiver) {
            println!("Error: {:?}", e);
            return;
        };
    }
    
}


fn process_message(client : &mut Client<TcpStream>) -> Result<(), WebSocketError> {
    let msg = match client.recv_message() {
        Ok(v) => v,
        Err(e) => match e {
            WebSocketError::NoDataAvailable => return Ok(()),
            WebSocketError::IoError(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => return Ok(()),
                _ => return Err(WebSocketError::IoError(e)),
            }
            _ => return Err(e)
        }
    };
    println!("Received: {:?}", msg);
    Ok(())
}

fn send_message(client : &mut Client<TcpStream>, receiver : &Receiver<OwnedMessage>) -> Result<(), WebSocketError> {
    if let Ok(msg) = receiver.recv_timeout(std::time::Duration::from_secs_f32(1.0)) {
        client.send_message(&msg).unwrap();
    }
    Ok(())
    
}

fn read_commands(channel : SyncSender<OwnedMessage>) {
    std::thread::spawn(move || {
        let mut buffer = String::with_capacity(4096);
        println!("Commands: help, start XXX, stop XXX");
        loop {
            buffer.clear();
            stdin().read_line(&mut buffer).expect("Must input characters");
            let msg = to_ws_message(buffer.trim());
            let msg = match msg {
                Some(v) => v,
                None => continue
            };
            channel.send(msg).unwrap();
        }
    });
}

fn to_ws_message(content : &str) -> Option<OwnedMessage> {
    if content.len() < 3 {
        return None
    }
    if content == "help" {
        println!("Commands: help, start XXX, stop XXX");
        return None
    }
    let msg = if content.starts_with("start") {
        let scenario = (&content[5..].trim()).to_string();
        println!("Starting scenario: {}", scenario);
        UserAction::TestScenario(scenario)
    }else if content.starts_with("stop") {
        let scenario = (&content[5..].trim()).to_string();
        println!("Stopping test scenario: {}", scenario);
        UserAction::StopTest(scenario)
    }else {
        return None
    };
    Some(OwnedMessage::Binary(serde_json::to_vec(&msg).unwrap()))
}