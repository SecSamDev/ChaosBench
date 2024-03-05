use std::{
    io::{stdin, IsTerminal, Read},
    net::TcpStream,
    sync::{
        atomic::AtomicBool,
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc,
    },
    time::Duration,
};

use chaos_core::api::user_actions::{CreateScenario, UserAction, UserActionResponse};
use dialoguer::{BasicHistory, Input, Select};
use websocket::{
    sync::Client,
    ws::{self, Message},
    OwnedMessage, WebSocketError,
};

const SERVER_ADDRESS: &str = env!("AGENT_SERVER_ADDRESS");

fn main() {
    let route = format!("ws://{}/user/connect", SERVER_ADDRESS);
    let mut client = websocket::ClientBuilder::new(&route)
        .unwrap()
        .connect_insecure()
        .unwrap();
    client.set_nonblocking(true).unwrap();
    println!("Connected to: {}", SERVER_ADDRESS);
    read_commands(client);
}

fn process_message(client: &mut Client<TcpStream>) -> Result<(), WebSocketError> {
    let msg = match client.recv_message() {
        Ok(v) => v,
        Err(e) => match e {
            WebSocketError::NoDataAvailable => return Ok(()),
            WebSocketError::IoError(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => return Ok(()),
                _ => return Err(WebSocketError::IoError(e)),
            },
            _ => return Err(e),
        },
    };
    let res: UserActionResponse = match msg {
        OwnedMessage::Text(v) => serde_json::from_str(&v).unwrap_or_default(),
        OwnedMessage::Binary(v) => serde_json::from_slice(&v).unwrap_or_default(),
        _ => return Ok(()),
    };
    println!("{:?}", res);
    Ok(())
}

fn send_message(
    client: &mut Client<TcpStream>,
    receiver: &Receiver<OwnedMessage>,
) -> Result<(), WebSocketError> {
    if let Ok(msg) = receiver.recv_timeout(std::time::Duration::from_secs_f32(1.0)) {
        client.send_message(&msg).unwrap();
    }
    Ok(())
}

fn read_commands(mut client: Client<TcpStream>) {
    let commands = [
        "help",
        "create-scenario",
        "start-scenario",
        "stop-scenario",
        "edit-scenario",
        "list-scenarios",
        "list-test-scenarios",
        "logs",
        "exit",
    ];
    let mut history = BasicHistory::new().max_entries(16).no_duplicates(true);
    loop {
        let selection = Select::new()
            .with_prompt("Execute a command")
            .items(&commands)
            .interact()
            .unwrap();
        let command = commands[selection];
        if command == "exit" {
            return;
        }
        let msg = to_ws_message(command, &mut history, &mut client);
        let msg = match msg {
            Some(v) => v,
            None => continue,
        };
        client.send_message(&msg).unwrap();
        process_message(&mut client).unwrap();
    }
}

fn print_help() {
    println!("Commands: ");
    println!("help: Shows this help");
    println!("logs: Listen to logs");
    println!("create-scenario: Creates a new testing scenario");
    println!("start-scenario: Starts a testing scenario");
    println!("stop-scenario: Stops a testing scenario");
    println!("edit-scenario: Edit parameters of a testing scenario");
    println!("list-scenarios: List all file scenarios");
    println!("list-test-scenarios: List all testing scenarios");
    println!("exit: exits the cli");
}

fn to_ws_message(
    command: &str,
    history: &mut BasicHistory,
    client: &mut Client<TcpStream>,
) -> Option<OwnedMessage> {
    if command == "help" {
        print_help();
        return None;
    }
    let msg = match command {
        "help" => {
            print_help();
            return None;
        }
        "start-scenario" => {
            let scenario_id = Input::<String>::new()
                .with_prompt("Scenario ID")
                .history_with(history)
                .interact_text()
                .unwrap();
            UserAction::StartScenario(scenario_id)
        }
        "list-scenarios" => UserAction::EnumerateScenarios,
        "list-test-scenarios" => UserAction::EnumerateTestingScenarios,
        "create-scenario" => {
            let id = Input::<String>::new()
                .with_prompt("Scenario ID")
                .history_with(history)
                .interact_text()
                .unwrap();
            let base_id = Input::<String>::new()
                .with_prompt("Base scenario ID")
                .history_with(history)
                .interact_text()
                .unwrap();
            UserAction::CreateScenario(CreateScenario { base_id, id })
        }
        "logs" => {
            listen_to_logs(client);
            return None;
        }
        _ => return None,
    };
    Some(OwnedMessage::Binary(serde_json::to_vec(&msg).unwrap()))
}

fn listen_to_logs(client: &mut Client<TcpStream>) {
    let run = Arc::new(AtomicBool::new(true));
    let thd = std::thread::spawn(|| {
        loop {
            let ext = Input::<String>::new()
            .with_prompt("Type exit...")
            .interact_text()
            .unwrap();
        if ext == "exit" {
            break;
        }
        }
    });
    loop {
        if !run.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        process_message(client).unwrap();
    }
    let _ = thd.join();
}
