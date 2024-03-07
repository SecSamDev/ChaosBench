use std::{fs::File, io::Write};

use actix::{
    Actor, Addr, StreamHandler
};
use actix_web_actors::ws;
use chaos_core::api::{agent::{AgentLogReq, AgentRequest, AgentResponse}, Log};

use crate::{domains::connection::AgentLog, state::ServerState};

use super::logs::LogServer;
pub struct AgentConnection {
    pub(crate) addr: Addr<LogServer>,
    pub(crate) id: String,
    pub(crate) state : ServerState,
    pub(crate) log : Option<File>
}

impl Actor for AgentConnection {
    type Context = ws::WebsocketContext<Self>;
}


impl AgentConnection {
    pub fn new(id: String, state : ServerState) -> Self {
        let log = std::fs::File::create(format!("agent-{}.log", id)).ok();
        Self { addr : state.log_server.clone(), id, state, log }
    }
    fn write_log_to_file(&mut self, log : &str) {
        if let Some(file) = &mut self.log {
            let _ = file.write_all(log.as_bytes());
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for AgentConnection {

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let data = match &msg {
            Ok(ws::Message::Text(text)) => process_agent_message(text.as_bytes()),
            Ok(ws::Message::Binary(bin)) => process_agent_message(bin),
            _ => {
                ctx.close(None);
                return;
            }
        };
        let data = match data {
            Some(v) => v,
            None => return,
        };
        match data {
            AgentRequest::Log(log) => {
                self.write_log_to_file(&log);
                self.addr.do_send(AgentLog(Log {
                    agent : self.id.clone(),
                    msg : log
                }));
            },
            AgentRequest::CompleteTask(task) => {
                self.state.services.set_task_as_executed(task);
            },
            AgentRequest::HeartBeat => {},
            AgentRequest::NextTask(hash) => {
                let actual_hash = self.state.services.hash_state();
                let scenario = match self.state.services.current_scenario() {
                    Ok(v) => v,
                    Err(_) => return
                };
                if actual_hash != hash {
                    let bin = serde_json::to_vec(&AgentResponse::Parameters(scenario.parameters)).unwrap();
                    ctx.binary(bin);
                    let bin = serde_json::to_vec(&AgentResponse::CustomActions(scenario.actions)).unwrap();
                    ctx.binary(bin);
                    return
                }
                let task = match self.state.services.get_next_task_for_agent(&self.id) {
                    Some(v) => v,
                    None => return
                };
                let bin = serde_json::to_vec(&AgentResponse::NextTask(task)).unwrap();
                ctx.binary(bin);
            }
        }
        
        
    }
}

fn process_agent_message(msg: &[u8]) -> Option<AgentRequest> {
    Some(serde_json::from_slice(msg).ok()?)
}
