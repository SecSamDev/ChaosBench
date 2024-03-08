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
use tungstenite::{stream::MaybeTlsStream, WebSocket};

const SERVER_ADDRESS: &str = env!("AGENT_SERVER_ADDRESS");

fn main() {
    let route = format!("ws://{}/user/connect", SERVER_ADDRESS);
    let (mut client, response) = tungstenite::client::connect(&route)
        .unwrap();
    println!("Connected to: {}", SERVER_ADDRESS);
    read_commands(client);
}

fn process_message(client: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<UserActionResponse, tungstenite::Error> {
    let msg = match client.read() {
        Ok(v) => v,
        Err(e) => match e {
            tungstenite::Error::Io(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => return Ok(UserActionResponse::None),
                _ => return Err(tungstenite::Error::Io(e)),
            },
            _ => return Err(e),
        },
    };
    let res: UserActionResponse = match msg {
        tungstenite::Message::Text(v) => serde_json::from_str(&v).unwrap_or_default(),
        tungstenite::Message::Binary(v) => serde_json::from_slice(&v).unwrap_or_default(),
        _ => return Ok(UserActionResponse::None),
    };
    Ok(res)
}


fn send_message(
    client: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    receiver: &Receiver<tungstenite::Message>,
) -> Result<(), tungstenite::Error> {
    if let Ok(msg) = receiver.recv_timeout(std::time::Duration::from_secs_f32(1.0)) {
        client.send(msg).unwrap();
    }
    Ok(())
}

fn read_commands(mut client: WebSocket<MaybeTlsStream<TcpStream>>) {
    let commands = [
        "help",
        "create-scenario",
        "start-scenario",
        "stop-scenario",
        "edit-scenario",
        "list-scenarios",
        "list-test-scenarios",
        "logs",
        "backup",
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
        client.send(msg).unwrap();
        let msg = process_message(&mut client).unwrap();
        println!("{:?}", msg);
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
    println!("backup: Saves the server state to disk");
    println!("exit: exits the cli");
}

fn to_ws_message(
    command: &str,
    history: &mut BasicHistory,
    client: &mut WebSocket<MaybeTlsStream<TcpStream>>,
) -> Option<tungstenite::Message> {
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
        },
        "backup" => {
            let backup = Input::<String>::new()
                .with_prompt("Backup name")
                .history_with(history)
                .interact_text()
                .unwrap();
            UserAction::BackupDB(backup)
        },
        "logs" => {
            listen_to_logs(client);
            return None;
        }
        _ => return None,
    };
    Some(user_action_to_message(&msg))
}

fn listen_to_logs(client: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
    client.send(user_action_to_message(&UserAction::Logs)).unwrap();
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
        let msg = process_message(client).unwrap();
        if let UserActionResponse::Logs(log) = msg {
            println!("{} - {}", log.agent, log.msg.trim());
        }else {
            println!("{:?}", msg);
        }
    }
    let _ = thd.join();
    client.send(user_action_to_message(&UserAction::NoLogs)).unwrap();
}


fn user_action_to_message(action : &UserAction) -> tungstenite::Message {
    tungstenite::Message::Binary(serde_json::to_vec(action).unwrap())
}