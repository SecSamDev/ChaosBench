use std::{
    io::Write,
    net::TcpStream,
    sync::{
        atomic::AtomicBool,
        mpsc::Receiver,
        Arc,
    },
    time::Duration,
};

use chaos_core::api::user_actions::{CreateScenario, UserAction, UserActionResponse};
use dialoguer::{BasicHistory, Input, Select};
use rustls::{ClientConfig, RootCertStore};
use tungstenite::{stream::MaybeTlsStream, WebSocket};

const SERVER_ADDRESS: &str = env!("SERVER_ADDRESS");
const SERVER_PORT: &str = env!("SERVER_PORT");
pub const SERVER_CERTIFICATE : &[u8] = include_bytes!(env!("CA_CERT"));

type WsClient = WebSocket<MaybeTlsStream<TcpStream>>;

fn main() {
    let route = format!("wss://{}:{}/_user/connect", SERVER_ADDRESS, SERVER_PORT);
    let mut root_store = RootCertStore::empty();
    let cert = rustls_pemfile::read_one_from_slice(SERVER_CERTIFICATE).unwrap().unwrap().0;
    let cert = match cert {
        rustls_pemfile::Item::X509Certificate(v) => v,
        _ => panic!("Invalid CA certificate format"),
    };
    root_store.add(cert).unwrap();
    let config = Arc::new(ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth());
    let sock = TcpStream::connect(&format!("{}:{}", SERVER_ADDRESS, SERVER_PORT)).unwrap();
    
    let (client, _response) = tungstenite::client_tls_with_config(&route, sock, None, Some(tungstenite::Connector::Rustls(config))).unwrap();
    if let MaybeTlsStream::Rustls(stream) = client.get_ref() {
        //let _ = stream.set_nonblocking(true);
        let _ = stream.sock.set_read_timeout(Some(Duration::from_secs_f32(0.2)));
        let _ = stream.sock.set_write_timeout(Some(Duration::from_secs_f32(5.0)));
    }
    println!("Connected to: {}", SERVER_ADDRESS);
    read_commands(client);
}

fn process_message(client: &mut WsClient) -> Result<UserActionResponse, tungstenite::Error> {
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
    client: &mut WsClient,
    receiver: &Receiver<tungstenite::Message>,
) -> Result<(), tungstenite::Error> {
    if let Ok(msg) = receiver.recv_timeout(std::time::Duration::from_secs_f32(1.0)) {
        client.send(msg).unwrap();
    }
    Ok(())
}

fn read_commands(mut client: WsClient) {
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
        "report",
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
    println!("report: Generates a markdown report");
    println!("exit: exits the cli");
}

fn to_ws_message(
    command: &str,
    history: &mut BasicHistory,
    client: &mut WsClient,
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
        "report" => {
            client.send(user_action_to_message(&UserAction::Report)).unwrap();
            let msg = process_message(client).unwrap();
            if let UserActionResponse::Report(rprt) = msg {
                let report = Input::<String>::new()
                    .with_prompt("Save report as")
                    .history_with(history)
                    .interact_text()
                    .unwrap();
                let file_name = format!("report-{}.md", report);
                let mut file = std::fs::File::create(&file_name).unwrap();
                file.write_all(rprt.report.as_bytes()).unwrap();
                println!("Report {} saved to {}", rprt.id, file_name);
            }else {
                println!("{:?}", msg);
            }
            return None
        },
        "logs" => {
            listen_to_logs(client);
            return None;
        }
        _ => return None,
    };
    Some(user_action_to_message(&msg))
}

fn listen_to_logs(client: &mut WsClient) {
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