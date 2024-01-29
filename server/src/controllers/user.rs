use actix_web::{Responder, HttpResponse, web::{Json, Data, self}, HttpRequest};
use actix_web_actors::ws;

use crate::{state::ServerState, actors};

pub fn user_config(cfg : &mut web::ServiceConfig) {
    cfg.service(web::resource("/user/connect").
        route(web::get().to(connect_user)));
}

async fn connect_user(req : HttpRequest, stream : web::Payload, state : Data<ServerState>) -> impl Responder {
    log::info!("User connection from ip={}", req.peer_addr().map(|v| v.ip().to_string()).unwrap_or_default());
    let connection = ws::start(actors::user::UserConnection {
        state : state.as_ref().clone()
    }, &req, stream);
    connection
}