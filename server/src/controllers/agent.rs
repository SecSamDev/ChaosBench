use actix_web::{
    web::{self, Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};
use actix_web_actors::ws;
use chaos_core::{action::metrics::MetricsArtifact, api::agent::ConnectAgent};
use tokio::io::AsyncWriteExt;

use crate::{actors, state::ServerState};

pub fn agent_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/_agent/connect").route(web::get().to(connect_agent)))
        .service(web::resource("/_agent/file/{filename}").route(web::get().to(download_file)).route(web::post().to(upload_artifact)))
        .service(web::resource("/_agent/metric/{metric_name}").route(web::post().to(upload_metrics)));
}

async fn connect_agent(
    req : HttpRequest, stream : web::Payload, state : Data<ServerState>
) -> impl Responder {
    let ip = req.peer_addr().map(|v| v.ip().to_string()).unwrap_or_default();
    let info = match agent_info(&req) {
        Some(v) => v,
        None => return HttpResponse::Forbidden().await
    };
    log::info!("Agent {} on {} connected with ip {ip}", info.id, info.hostname);
    let id = info.id.clone();
    state.services.register_new_agent(info);
    ws::start(actors::agent::AgentConnection::new(id, state.as_ref().clone()), &req, stream)
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

async fn upload_metrics(
    request : HttpRequest,
    metrics: Json<MetricsArtifact>,
    metric_name : Path<String>,
    state : Data<ServerState>
) -> impl Responder {
    let headers = request.headers();
    let agent_id = match headers.get("Agent-ID") {
        Some(v) => {
            v.to_str().unwrap_or_default()
        },
        None => return HttpResponse::Unauthorized().finish()
    };
    if agent_id.is_empty() {
        return HttpResponse::Unauthorized().finish()
    }
    log::info!("Uploading metrics: {}", metric_name);
    let parent = std::env::current_dir().unwrap().join("workspace").join(agent_id);
    let file_path = parent.join(format!("metric-{}.json", metric_name.as_str()));
    std::fs::create_dir_all(&parent).unwrap();
    let mut fs = tokio::fs::File::create(&file_path).await.unwrap();
    let buff = serde_json::to_vec_pretty(&metrics.0).unwrap();
    fs.write_all(&buff).await.unwrap();
    match state.services.set_metrics_for_agent(agent_id, metric_name.as_str(), metrics.0) {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::BadRequest()
    }.finish()
    
}

fn agent_info(req : &HttpRequest) -> Option<ConnectAgent> {
    let agent_id = req.headers().get("Agent-Id")?.to_str().ok()?;
    let agent_host = req.headers().get("Agent-Host")?.to_str().ok()?;
    let arch = req.headers().get("Agent-Arch")?.to_str().ok()?;
    let os = req.headers().get("Agent-Os")?.to_str().ok()?;
    Some(ConnectAgent {
        id : agent_id.to_string(),
        hostname : agent_host.to_string(),
        arch : arch.into(),
        os : os.into(),
        ip : req.connection_info().peer_addr().unwrap_or_default().to_string()
    })
}

async fn upload_artifact(
    request : HttpRequest,
    stream : web::Payload,
    file_name : Path<String>
) -> impl Responder {
    let headers = request.headers();
    let agent_id = match headers.get("Agent-ID") {
        Some(v) => {
            v.to_str().unwrap_or_default()
        },
        None => return HttpResponse::Unauthorized().finish()
    };
    if agent_id.is_empty() {
        return HttpResponse::Unauthorized().finish()
    }
    log::info!("Uploading artifact: {}", file_name);
    let parent = std::env::current_dir().unwrap().join("workspace").join(agent_id).join("artifacts");
    if !parent.exists() {
        match tokio::fs::create_dir_all(&parent).await {
            Ok(_) => {},
            Err(e) => {
                log::warn!("Cannot create agent artifact folder: {}", e);
                return HttpResponse::InternalServerError().finish()
            }
        }
    }
    let file_path = parent.join(file_name.as_str());
    let mut file = match tokio::fs::File::create(&file_path).await {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Cannot create artifact file: {}", e);
            return HttpResponse::InternalServerError().finish()
        }
    };
    let byts = match stream.to_bytes().await {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Cannot extract bytes: {}", e);
            return HttpResponse::InternalServerError().finish()
        }
    };
    match file.write_all(&byts).await {
        Ok(_) => {},
        Err(e) => {
            log::warn!("Cannot write artifact content: {}", e);
            return HttpResponse::InternalServerError().finish()
        }
    };
    HttpResponse::Ok().finish()
}