use actix_web::{
    web::{self, Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_actors::ws;
use chaos_core::{api::agent::*, tasks::AgentTask};

use crate::{actors, state::ServerState};

pub fn agent_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/agent/connect").route(web::get().to(connect_agent)))
        .service(web::resource("/agent/file/{filename}").route(web::get().to(download_file)))
        .service(web::resource("/agent/next_task").route(web::post().to(next_task)));
}

async fn connect_agent(
    req : HttpRequest, stream : web::Payload, state : Data<ServerState>
) -> impl Responder {
    let ip = req.peer_addr().map(|v| v.ip().to_string()).unwrap_or_default();
    log::info!("Agent connection from ip={}", ip);
    let connection = ws::start(actors::agent::AgentConnection::new(ip,state.as_ref().clone()), &req, stream);
    connection
}

async fn download_file(
    request: HttpRequest,
    state: Data<ServerState>,
    filename : Path<String>
) -> impl Responder {
    match state.services.download_file(&filename).await {
        Some(v) => v.into_response(&request),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn next_task(
    request: Json<NextTaskForAgentReq>,
    state: Data<ServerState>,
) -> impl Responder {
    let task = state.services.get_next_task_for_agent(&request.agent_id).await;
    let res = NextTaskForAgentRes {
        task
    };
    HttpResponse::Ok().json(&res)
}