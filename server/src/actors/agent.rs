use std::{fs::File, io::Write};

use actix::{
    Actor, Addr, StreamHandler
};
use actix_web_actors::ws;
use chaos_core::api::{agent::AgentLogReq, Log};

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
        if let Some(file) = &mut self.log {
            let _ = file.write_all(data.log.msg.as_bytes());
        }
        self.addr.do_send(AgentLog {
            msg : Log {
                agent : data.log.agent,
                file : data.log.file,
                msg : data.log.msg
            }
        });
    }
}

fn process_agent_message(msg: &[u8]) -> Option<AgentLogReq> {
    Some(serde_json::from_slice(msg).ok()?)
}
