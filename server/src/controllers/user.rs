use actix_web::{Responder, web::{Data, self}, HttpRequest};
use actix_web_actors::ws;

use crate::{actors::user::UserConnection, state::ServerState};

pub fn user_config(cfg : &mut web::ServiceConfig) {
    cfg.service(web::resource("/_user/connect").
        route(web::get().to(connect_user)));
}

async fn connect_user(req : HttpRequest, stream : web::Payload, state : Data<ServerState>) -> impl Responder {
    let id = req.peer_addr().map(|v| v.ip().to_string()).unwrap_or_default();
    log::info!("User connection from ip={}", id);
    let connection = ws::start(UserConnection::new(id, state.as_ref().clone(), state.log_server.clone()), &req, stream);
    connection
}