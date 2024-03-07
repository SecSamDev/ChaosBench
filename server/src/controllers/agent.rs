use actix_web::{
    web::{self, Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_actors::ws;
use chaos_core::{api::agent::*, tasks::AgentTask};

use crate::{actors, state::ServerState};

pub fn agent_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/agent/connect/{agent_id}").route(web::get().to(connect_agent)))
        .service(web::resource("/agent/file/{filename}").route(web::get().to(download_file)));
}

async fn connect_agent(
    req : HttpRequest, stream : web::Payload, state : Data<ServerState>, agent_id : Path<String>
) -> impl Responder {
    let ip = req.peer_addr().map(|v| v.ip().to_string()).unwrap_or_default();
    log::info!("Agent connection from ip={}", ip);
    let connection = ws::start(actors::agent::AgentConnection::new(agent_id.as_str().to_string(),state.as_ref().clone()), &req, stream);
    connection
}

async fn download_file(
    request: HttpRequest,
    filename : Path<String>
) -> impl Responder {
    log::info!("Downloading file: {}", filename);
    let file_path = std::env::current_dir().unwrap().join("workspace").join(filename.as_str());

    let file = match actix_files::NamedFile::open_async(file_path).await {
        Ok(v) => v,
        Err(_) => return HttpResponse::NotFound().finish()
    };
    file.into_response(&request)
}