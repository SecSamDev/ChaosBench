use std::collections::BTreeMap;

use actix_web::{http::Uri, web::{self, Data}, HttpMessage, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};
use chaos_core::{action::TestActionType, err::{ChaosError, ChaosResult}, tasks::AgentTaskResult};
use reqwest::{RequestBuilder, StatusCode};
use rhai::{Engine, Scope};

use crate::{state::ServerState, utils::now_milliseconds};

pub fn proxy_config(cfg: &mut web::ServiceConfig) {
    cfg.default_service(web::to(proxy_request));
}

async fn proxy_request(
    req : HttpRequest, stream : web::Payload, state : Data<ServerState>
) -> impl Responder {
    match proxy_request_wrapper(req, stream, state).await {
        Ok(v) => v,
        Err(e) => {
            log::warn!("{}", e);
            HttpResponse::InternalServerError()
        }
    }.finish()
}

fn generate_client(req : &HttpRequest, state : &ServerState) -> RequestBuilder {
    let server = state.services.remote_server().unwrap_or_default();
    let uri = req.uri();
    let mut url = Uri::builder();
    if let Some(scheme) = uri.scheme_str() {
        url = url.scheme(scheme);
    }
    if let Some(v) = uri.path_and_query() {
        url = url.path_and_query(v.clone());
    }
    url = url.authority(server);
    let url = url.build().unwrap_or_default().to_string();
    let mut request = reqwest::Client::new().request(req.method().clone(), &url);
    for (name, value) in req.headers() {
        let val = match value.to_str() {
            Ok(v) => v,
            Err(_) => continue
        };
        let name = name.as_str();
        log::info!("Header: {}", name);
        if name == "Host" {
            continue
        }
        request = request.header(name, val);
    }
    request
}


async fn proxy_request_wrapper(
    req : HttpRequest, stream : web::Payload, state : Data<ServerState>
) -> ChaosResult<HttpResponseBuilder>{
    let start = now_milliseconds();
    let client = generate_client(&req, &state);
    let agent = state.services.agent_from_ip(req.connection_info().peer_addr().unwrap_or_default())?;
    let task = state.services.get_next_task_for_agent(&agent.id);
    let bytes = match stream.to_bytes().await {
        Ok(v) => v.as_ref().to_vec(),
        Err(_) => Vec::new()
    };
    if let Some(task) = task {
        if let TestActionType::HttpRequestInspect = task.action {
            let script_name : &str = match task.parameters.get("script") {
                Some(v) => v.try_into().unwrap_or_default(),
                None => return Err("Cannot find HttpRequestInspect script parameter".to_string().into())
            };
            let script = state.services.get_sever_script(&script_name)?;
            if let Some(result) = run_script_when_request(&req, &script, &bytes) {
                state.services.set_task_as_executed(AgentTaskResult {
                    scene_id : task.scene_id,
                    id : task.id,
                    action : task.action,
                    agent : agent.id.clone(),
                    start,
                    retries : 1,
                    end : now_milliseconds(),
                    limit : task.limit,
                    parameters : task.parameters,
                    result
                });
            }
        }
    }
    let client = client.body(bytes);
    let response = match client.send().await {
        Ok(v) => v,
        Err(e) => {
            if let Some(task) = state.services.get_next_task_for_agent(&agent.id){
                if let TestActionType::HttpResponseInspect = task.action {
                    state.services.set_task_as_executed(AgentTaskResult {
                        scene_id : task.scene_id,
                        id : task.id,
                        action : task.action,
                        agent : agent.id.clone(),
                        start,
                        retries : 1,
                        end : now_milliseconds(),
                        limit : task.limit,
                        parameters : task.parameters,
                        result : Err(ChaosError::Other(e.to_string()))
                    });
                }
            }
            return Err(format!("Cannot proxy http request: {}", e).into())
        }
    };
    let status = response.status();
    if let Some(task) = state.services.get_next_task_for_agent(&agent.id){
        if let TestActionType::HttpResponseInspect = task.action {
            let script_name : &str = match task.parameters.get("script") {
                Some(v) => v.try_into().unwrap_or_default(),
                None => return Err("Cannot find HttpResponseInspect script parameter".to_string().into())
            };
            let script = state.services.get_sever_script(&script_name)?;
            if let Some(res) = run_script_when_response(&req, &script, response).await {
                let (result, ret) = match res {
                    Ok(v) => (Ok(()), v),
                    Err(e) => (Err(e), HttpResponse::InternalServerError())
                };
                state.services.set_task_as_executed(AgentTaskResult {
                    scene_id : task.id,
                    id : task.id,
                    action : task.action,
                    agent : agent.id.clone(),
                    start,
                    retries : 1,
                    end : now_milliseconds(),
                    limit : task.limit,
                    parameters : task.parameters,
                    result
                });
                return Ok(ret)
            }
        }
    }
    Ok(HttpResponseBuilder::new(status))
}

fn run_script_when_request(req : &HttpRequest, script : &str, body : &[u8]) -> Option<ChaosResult<()>>{
    let content_type = req.content_type();
    let engine = rhai::Engine::new();
    let mut scope = Scope::new();
    if content_type == "application/json" {
        let body : serde_json::Value = serde_json::from_slice(body).unwrap_or_default();
        scope.set_value("body", body);
    }
    let mut headers = BTreeMap::new();
    for (n, v) in req.headers() {
        headers.insert(n.to_string(), v.to_str().unwrap_or_default().to_string());
    }
    scope.set_value("headers", headers);
    scope.set_value("full_url", req.full_url().as_str().to_string());
    eval_with_scope(&engine, &mut scope, script, ())
}

async fn run_script_when_response(req : &HttpRequest, script : &str, response : reqwest::Response) -> Option<ChaosResult<HttpResponseBuilder>> {
    let mut ret = HttpResponseBuilder::new(response.status());
    let content_type : String = response.headers().get("Content-Type").map(|v| v.to_str().unwrap_or_default().to_string()).unwrap_or_default();
    let engine = rhai::Engine::new();
    let mut scope = Scope::new();
    let mut headers = BTreeMap::new();
    for (n, v) in response.headers() {
        headers.insert(n.to_string(), v.to_str().unwrap_or_default().to_string());
        ret.insert_header((n.as_str(), v.to_str().unwrap_or_default()));
    }
    scope.set_value("headers", headers);
    scope.set_value("status_code", response.status().as_u16());
    scope.set_value("full_url", req.full_url().as_str().to_string());
    let body = response.bytes().await.unwrap_or_default().as_ref().to_vec();
    scope.set_value("content_type", content_type.clone());
    if &content_type == "application/json" {
        let body : serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
        scope.set_value("body", body);
    }
    ret.body(body);
    match eval_with_scope(&engine, &mut scope, script, ret)? {
        Ok(mut v) => {
            let (body, status_code, headers) : (serde_json::Value, u16, BTreeMap<String, String>) = match (scope.get_value("body"), scope.get_value("status_code"), scope.get_value("headers")) {
                (Some(v1), Some(v2), Some(v3)) => (v1, v2, v3),
                _ => return Some(Ok(v))
            };
            v.body(serde_json::to_vec(&body).unwrap_or_default());
            v.status(StatusCode::from_u16(status_code).unwrap_or_default());
            for (header, value) in headers {
                v.insert_header((header, value));
            }
            Some(Ok(v))
        },
        Err(e) => Some(Err(e))
    }
}

fn eval_with_scope<T>(engine: &Engine, scope : &mut Scope, script : &str, ret : T) -> Option<ChaosResult<T>> {
    match engine.eval_with_scope(scope, &script) {
        Ok(v) => {
            let v : bool = v;
            if v {
                Some(Ok(ret))
            }else {
                None
            }
        },
        Err(e) => Some(Err(ChaosError::Other(e.to_string())))
    }
}