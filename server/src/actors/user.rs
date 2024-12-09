use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use chaos_core::api::user_actions::{CreateScenario, UserAction, UserActionResponse};

use crate::{domains::connection::{AgentAppLog, AgentCompletionUpdate, AgentLog, ConnectAppLog, ConnectAppLogById, ConnectLog, ConnectLogByAgent, DisconnectLog}, state::ServerState};

use super::logs::LogServer;
pub struct UserConnection {
    pub(crate) addr: Addr<LogServer>,
    pub(crate) state : ServerState,
    pub(crate) id: String,
}

impl UserConnection {
    pub fn new(id : String, state : ServerState, addr : Addr<LogServer>) -> Self {
        Self {
            id,
            addr,
            state
        }
    }
}

impl Actor for UserConnection {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<AgentLog> for UserConnection {
    type Result = ();

    fn handle(&mut self, msg: AgentLog, ctx: &mut Self::Context) -> Self::Result {
        let res = UserActionResponse::Logs(msg.0);
        let bin = serde_json::to_vec(&res).unwrap();
        ctx.binary(bin);
    }
}
impl Handler<AgentAppLog> for UserConnection {
    type Result = ();

    fn handle(&mut self, msg: AgentAppLog, ctx: &mut Self::Context) -> Self::Result {
        let res = UserActionResponse::AppLogs(msg.0);
        let bin = serde_json::to_vec(&res).unwrap();
        ctx.binary(bin);
    }
}

impl Handler<AgentCompletionUpdate> for UserConnection {
    type Result = ();

    fn handle(&mut self, msg: AgentCompletionUpdate, ctx: &mut Self::Context) -> Self::Result {
        let res = UserActionResponse::AgentCompletion((msg.completed, msg.total));
        let bin = serde_json::to_vec(&res).unwrap();
        ctx.binary(bin);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UserConnection {
    fn finished(&mut self, _ctx: &mut Self::Context) {
        self.addr.do_send(DisconnectLog {
            id : self.id.clone()
        });
    }

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let data = match &msg {
            Ok(ws::Message::Text(text)) => process_user_message(text.as_bytes()),
            Ok(ws::Message::Binary(bin)) => process_user_message(bin),
            _ => {
                ctx.close(None);
                return;
            },
        };
        let data = match data {
            Some(v) => v,
            None => return
        };
        log::info!("Received action: {:?}", data);

        let res = match data {
            UserAction::AgentLogsAll => {
                let upd = ctx.address().recipient();
                let addr = ctx.address().recipient();
                self.addr.do_send(ConnectLog {
                    addr,
                    upd,
                    id :self.id.clone()
                });
                return
            },
            UserAction::AgentLogs(s) => {
                let upd = ctx.address().recipient();
                let addr = ctx.address().recipient();
                self.addr.do_send(ConnectLogByAgent {
                    addr,
                    upd,
                    agent : s.agent,
                    id :self.id.clone()
                });
                return
            },
            UserAction::AppLogs(s) => {
                let addr = ctx.address().recipient();
                self.addr.do_send(ConnectAppLogById {
                    addr,
                    agent : s.agent,
                    id :self.id.clone()
                });
                return
            },
            UserAction::AppLogsAll => {
                let addr = ctx.address();
                self.addr.do_send(ConnectAppLog {
                    addr : addr.recipient(),
                    id :self.id.clone()
                });
                return
            },
            UserAction::Report => generate_report(&self.state),
            UserAction::BackupDB(v) => backup_db(v, &self.state),
            UserAction::StartScenario(v) => start_scenario(v, &self.state),
            UserAction::StopScenario => stop_scenario(&self.state),
            UserAction::EnumerateScenarios => list_scenarios(&self.state),
            UserAction::EnumerateTestingScenarios => list_testing_scenarios(&self.state),
            UserAction::EnumerateAgents => list_agents(&self.state),
            UserAction::CreateScenario(v) => create_scenario(v, &self.state),
            _ => return
        };
        if let Some(res) = res {
            let bin = serde_json::to_vec(&res).unwrap();
            ctx.binary(bin);
        }
    }
}

fn backup_db(location : String, state: &ServerState) -> Option<UserActionResponse> {
    let res = state.services.backup_db(&location);
    Some(UserActionResponse::BackupDB(res))
}
fn create_scenario(create : CreateScenario, state: &ServerState) -> Option<UserActionResponse> {
    let res = state.services.create_testing_scenario(create.id, &create.base_id);
    Some(UserActionResponse::CreateScenario(res))
}

fn stop_scenario(state : &ServerState) -> Option<UserActionResponse> {
    let res = state.services.stop_testing_scenario();
    Some(UserActionResponse::StopScenario(res))
}
fn start_scenario(scenario : String, state : &ServerState) -> Option<UserActionResponse> {
    let res = state.services.execute_testing_scenario(scenario);
    Some(UserActionResponse::StartScenario(res))
}
fn list_scenarios(state : &ServerState) -> Option<UserActionResponse> {
    let scenarios = state.services.list_scenarios();
    Some(UserActionResponse::EnumerateScenarios(scenarios))
}
fn list_testing_scenarios(state : &ServerState) -> Option<UserActionResponse> {
    let scenarios = state.services.list_testing_scenarios();
    Some(UserActionResponse::EnumerateTestingScenarios(scenarios))
}
fn generate_report(state : &ServerState) -> Option<UserActionResponse> {
    let rprt = state.services.generate_report().ok()?;
    Some(UserActionResponse::Report(rprt))
}

fn process_user_message(msg : &[u8]) -> Option<UserAction> {
    serde_json::from_slice(msg).ok()
}
fn list_agents(state : &ServerState) -> Option<UserActionResponse> {
    let scenarios = state.services.list_agents();
    Some(UserActionResponse::EnumerateAgents(scenarios))
}