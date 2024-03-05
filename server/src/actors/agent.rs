use actix::{
    fut, Actor, Addr, AsyncContext, Handler, Message as ActixMessage, Recipient, Running, StreamHandler, WrapFuture
};
use actix_web_actors::ws;
use chaos_core::api::{agent::AgentLogReq, user_actions::{CreateScenario, UserAction, UserActionResponse}, Log};

use crate::{domains::connection::{AgentLog, Connect, Disconnect}, state::ServerState};

use super::logs::LogServer;
pub struct AgentConnection {
    pub(crate) addr: Addr<LogServer>,
    pub(crate) id: String,
}

impl Actor for AgentConnection {
    type Context = ws::WebsocketContext<Self>;
}


impl AgentConnection {
    pub fn new(id: String, addr: Addr<LogServer>) -> Self {
        Self { addr, id }
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
        log::info!("{} - {}", data.agent, data.log.msg);
        self.addr.do_send(AgentLog {
            msg : Log {
                agent : data.log.agent,
                file : data.log.file,
                msg : data.log.msg
            }
        });
    }
}

fn backup_db(location: String, state: &ServerState) -> Option<UserActionResponse> {
    let res = state.services.backup_db(&location);
    Some(UserActionResponse::BackupDB(res))
}
fn create_scenario(create: CreateScenario, state: &ServerState) -> Option<UserActionResponse> {
    let res = state
        .services
        .create_testing_scenario(create.id, &create.base_id);
    Some(UserActionResponse::CreateScenario(res))
}

fn stop_scenario(scenario: String, state: &ServerState) -> Option<UserActionResponse> {
    let res = state.services.stop_testing_scenario(scenario);
    Some(UserActionResponse::StopScenario(res))
}
fn start_scenario(scenario: String, state: &ServerState) -> Option<UserActionResponse> {
    let res = state.services.execute_testing_scenario(scenario);
    Some(UserActionResponse::StartScenario(res))
}
fn list_scenarios(state: &ServerState) -> Option<UserActionResponse> {
    let scenarios = state.services.list_scenarios();
    Some(UserActionResponse::EnumerateScenarios(scenarios))
}
fn list_testing_scenarios(state: &ServerState) -> Option<UserActionResponse> {
    let scenarios = state.services.list_testing_scenarios();
    Some(UserActionResponse::EnumerateTestingScenarios(scenarios))
}

fn process_agent_message(msg: &[u8]) -> Option<AgentLogReq> {
    log::info!("{}", String::from_utf8_lossy(msg));
    Some(serde_json::from_slice(msg).ok()?)
}
