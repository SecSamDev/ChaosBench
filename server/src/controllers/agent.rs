use actix_web::{
    web::{self, Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};
use chaos_core::{api::agent::*, tasks::AgentTask};

use crate::state::ServerState;

pub fn agent_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/agent/connect").route(web::post().to(connect_agent)))
        .service(web::resource("/agent/file/{filename}").route(web::get().to(download_file)))
        .service(web::resource("/agent/next_task").route(web::post().to(next_task)));
}

async fn connect_agent(
    request: Json<ConnectAgentRequest>,
    state: Data<ServerState>,
) -> impl Responder {
    HttpResponse::Ok().await
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
    let res = NextTaskForAgentRes {
        task : Some(AgentTask {
            ..Default::default()
        })
    };
    HttpResponse::Ok().json(&res)
}