use std::{net::TcpStream, str::FromStr, sync::{mpsc::{Receiver, RecvTimeoutError, SyncSender}, Arc}, time::{Duration, SystemTime, UNIX_EPOCH}};

use chaos_core::{action::TestActionType, api::agent::{AgentRequest, AgentResponse, ConnectAgent}, err::ChaosError, tasks::AgentTaskResult};
use rustls::{ClientConfig, RootCertStore};
use tungstenite::{handshake::client::generate_key, http::{Uri, Version}, stream::MaybeTlsStream, Message, WebSocket};

use crate::{actions::execute_action, api::SERVER_CERTIFICATE, common::{now_milliseconds, AgentTaskInternal, StopCommand}, logging::init_logging, state::{AgentState, SERVER_ADDRESS, SERVER_PORT}, sys_info::{get_hostname, get_system_uuid}};

type WsClient = WebSocket<MaybeTlsStream<TcpStream>>;

pub fn wait_for_service_signal(signal_sender : SyncSender<StopCommand>, signal : Receiver<StopCommand>) {
    let receiver = init_logging();
    let mut state = AgentState::new(signal_sender);
    if let Some(v) = receiver {
        state.set_log_receiver(v);
    }
    let mut notified_start = false;
    //on_start_service(&mut state);
    'out: loop {
        if check_shutdown_signal(&signal, &mut state) {
            break 'out
        }
        let mut client = match create_ws_client() {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Cannot connect to WS client: {}", e);
                std::thread::sleep(Duration::from_secs_f32(5.0));
                continue
            }
        };
        if !notified_start {
            // Notify of started agent
            notified_start = match on_start_service(&mut state, &mut client) {
                Some(v) => v,
                None => true
            };
        }
        loop {
            if check_shutdown_signal(&signal, &mut state) {
                break 'out
            }
            if let Err(e) = agent_loop(&mut state, &mut client) {
                log::warn!("{}", e);
                continue 'out;
            }
        }
    }
    log::info!("Stopping ChaosAgent");
}

fn agent_loop(state : &mut AgentState, client : &mut WsClient) -> Result<(), tungstenite::Error> {
    send_logs(state,client)?;
    read_messages(state, client)?;
    do_work(state,client)?;
    Ok(())
}

fn check_shutdown_signal(signal : &Receiver<StopCommand>, state : &mut AgentState) -> bool {
    match signal.recv_timeout(Duration::from_secs_f32(0.2)) {
        Ok(v) => {
            if let StopCommand::Shutdown = v {
                signal_agent_shutdown(state);
            }
            return true
        },
        Err(e) => if let RecvTimeoutError::Disconnected = e {
            return true
        }
    };
    false
}

fn create_ws_client() -> Result<WsClient, tungstenite::Error>{
    let agent = get_system_uuid().unwrap();
    let hostname = get_hostname().unwrap_or_default();
    let route = format!("wss://{}:{}/_agent/connect", SERVER_ADDRESS, SERVER_PORT);
    let req = ConnectAgent::default();
    let uri = tungstenite::http::Uri::from_str(&route)?;
    let req = tungstenite::http::Request::builder()
        .uri(&route)
        .version(tungstenite::http::Version::HTTP_11)
        .header("Sec-WebSocket-Protocol", "ws")
        .header("Sec-WebSocket-Key", generate_key())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Agent-Id", agent)
        .header("Agent-Host", hostname)
        .header("Agent-Arch", Into::<&str>::into(req.arch))
        .header("Agent-Os", Into::<&str>::into(req.os))
        .header("Sec-WebSocket-Version", 13)
        .header("Host", uri.host().unwrap());
    let request = tungstenite::handshake::client::Request::from(req.body(())?);
    let mut root_store = RootCertStore::empty();
    let cert = rustls_pemfile::read_one_from_slice(SERVER_CERTIFICATE).unwrap().unwrap().0;
    let cert = match cert {
        rustls_pemfile::Item::X509Certificate(v) => v,
        _ => panic!("Invalid CA certificate format"),
    };
    root_store.add(cert).unwrap();
    let config = Arc::new(ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth());
    let sock = TcpStream::connect(&format!("{}:{}", SERVER_ADDRESS, SERVER_PORT)).unwrap();
    
    let (client, _response) = tungstenite::client_tls_with_config(request, sock, None, Some(tungstenite::Connector::Rustls(config))).unwrap();
    if let MaybeTlsStream::Rustls(stream) = client.get_ref() {
        //let _ = stream.set_nonblocking(true);
        let _ = stream.sock.set_read_timeout(Some(Duration::from_secs_f32(0.2)));
        let _ = stream.sock.set_write_timeout(Some(Duration::from_secs_f32(5.0)));
    }
    log::info!("Agent connected to: {}", route);
    Ok(client)
}

fn on_start_service(state : &mut AgentState, client : &mut WsClient) -> Option<bool> {
    let task = state.db.get_current_task()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
    if TestActionType::RestartHost != task.action {
        return None
    }
    let mut task: AgentTaskResult = task.to_owned().into();
    task.end = now;
    task.result = Ok(());
    state.db.clean_current_task();
    if let Err(err) = client.send(agent_request_to_message(&AgentRequest::CompleteTask(task))) {
        log::error!("Cannot notify of completed task: {:?}", err);
        Some(false)
    }else{
        Some(true)
    }
}

fn send_logs(state : &mut AgentState, client : &mut WsClient) -> Result<(), tungstenite::Error> {
    loop {
        let log = match state.try_recv_app_log() {
            Some(v) => v,
            None => break
        };
        client.send(agent_request_to_message(&AgentRequest::AppLog(log)))?;
    }
    let mut c = 0;
    loop {
        if c > 5 {
            return Ok(())
        }
        let log = match state.try_recv_log() {
            Some(v) => v,
            None => {
                c += 1;
                continue;
            }
        };
        c = 0;
        client.send(agent_request_to_message(&AgentRequest::Log(log)))?;
    }
}

fn read_messages(state : &mut AgentState, client : &mut WsClient) -> Result<(), tungstenite::Error> {
    loop {
        let a = match client.read() {
            Ok(v) => v,
            Err(e) => match e {
                tungstenite::Error::Io(io) => {
                    match io.kind() {
                        std::io::ErrorKind::TimedOut => return Ok(()),
                        std::io::ErrorKind::WouldBlock => return Ok(()),
                        _ => return Err(tungstenite::Error::Io(io))
                    }
                },
                _ => return Err(e)
            }
        };
        let res = match message_to_agent_response(a) {
            Some(v) => v,
            None => return Ok(())
        };
        log::info!("{:?}", res);
        match res {
            AgentResponse::NextTask(task) => {
                state.db.set_current_task(Some(task));
            },
            AgentResponse::CleanTask => {
                state.db.set_current_task(None);
            },
            AgentResponse::Parameters(v) => {
                state.db.set_global_parameters(v);
            },
            AgentResponse::CustomActions(v) => {
                state.db.set_commands(v);
            }
            AgentResponse::Stop => {
                state.signal_shutdown(StopCommand::Stop);
            },
            AgentResponse::Variables(v) => {
                state.db.set_global_variables(v);
            },
            AgentResponse::Wait => {
                log::info!("No task to execute. Waiting...");
                send_logs(state, client)?;
                std::thread::sleep(Duration::from_secs_f32(30.0));
                break
            },
        }
    }
    

    Ok(())
}

fn do_work(state : &mut AgentState, client : &mut WsClient) -> Result<(), tungstenite::Error> {
    // Do things while waiting for the service stop signal
    let mut task = match state.db.get_current_task() {
        None => {
            let _ = client.send(agent_request_to_message(&AgentRequest::NextTask(state.state_hash())));
            return Ok(())
        },
        Some(v) => v.clone(),
    };
    log::info!("Task to execute ID={} Start={} Limit={} TTL={}", task.id, task.start, task.limit, (task.start + task.limit) - now_milliseconds());
    if task.start == 0 {
        task.start = now_milliseconds();
    }
    match execute_action(task.action.clone(), state, &mut task) {
        Ok(_) => {},
        Err(err) => {
            let tries = state.increase_task_try();
            if task_reached_max_duration(&task) {
                log::info!("Error executing task {} ({tries}): {:?}", task.id, err);
                task.end = Some(now_milliseconds());
                task.result = Some(Err(ChaosError::Other(format!("Error executing task {} ({tries}): {:?}", task.id, err))));
            }
        }
    };
    if task_reached_max_duration(&task) && task.result.is_none() {
        log::warn!("Max timeout reached for task: {}", task.id);
        task.end = Some(now_milliseconds());
        task.result = Some(Err(ChaosError::Other(format!("Error executing task {}: Timeout reached", task.id))));
    }
    if task.result.is_some() {
        client.send(agent_request_to_message(&AgentRequest::CompleteTask(task.into())))?;
        state.db.clean_current_task();
        log::info!("Sent completed task");
    }else {
        state.db.update_current_task(task);
    }
    Ok(())
}

fn task_reached_max_duration(task : &AgentTaskInternal) -> bool {
    (task.start + task.limit) - now_milliseconds() < 0
}

fn signal_agent_shutdown(_state : &mut AgentState) {
    // The agent was signaled to shutdown -> Create file to manage that
}

fn agent_request_to_message(action : &AgentRequest) -> tungstenite::Message {
    tungstenite::Message::Binary(serde_json::to_vec(action).unwrap())
}

fn message_to_agent_response(message : Message) -> Option<AgentResponse> {
    let res: AgentResponse = match message {
        tungstenite::Message::Text(v) => serde_json::from_str(&v).unwrap_or_default(),
        tungstenite::Message::Binary(v) => serde_json::from_slice(&v).unwrap_or_default(),
        _ => return None
    };
    Some(res)
}