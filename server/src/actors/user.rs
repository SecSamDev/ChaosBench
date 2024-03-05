use actix::{Actor, Addr, AsyncContext, Message as ActixMessage, Recipient, StreamHandler};
use actix_web_actors::ws;
use chaos_core::api::user_actions::{CreateScenario, UserAction, UserActionResponse};

use crate::state::ServerState;

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

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UserConnection {
    fn started(&mut self, ctx: &mut Self::Context) {
        let ad = ctx.address();
    }

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        log::info!("Msg received from user");
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
            UserAction::BackupDB(v) => backup_db(v, &self.state),
            UserAction::StartScenario(v) => start_scenario(v, &self.state),
            UserAction::StopScenario(v) => stop_scenario(v, &self.state),
            UserAction::EnumerateScenarios => list_scenarios(&self.state),
            UserAction::EnumerateTestingScenarios => list_testing_scenarios(&self.state),
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

fn stop_scenario(scenario : String, state : &ServerState) -> Option<UserActionResponse> {
    let res = state.services.stop_testing_scenario(scenario);
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


fn process_user_message(msg : &[u8]) -> Option<UserAction> {
    Some(serde_json::from_slice(msg).ok()?)
}