use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use chaos_core::api::user_actions::UserAction;

use crate::state::ServerState;
pub struct UserConnection {
    pub(crate) state : ServerState
}

impl Actor for UserConnection {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UserConnection {
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
        

    }
}


fn process_user_message(msg : &[u8]) -> Option<UserAction> {
    Some(serde_json::from_slice(msg).ok()?)
}