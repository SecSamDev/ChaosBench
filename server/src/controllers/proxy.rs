use std::{collections::BTreeMap, time::{SystemTime, UNIX_EPOCH}};

use actix_web::{http::Uri, web::{self, Data}, HttpMessage, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};
use chaos_core::{action::TestActionType, err::ChaosError, tasks::AgentTaskResult};
use reqwest::RequestBuilder;
use rhai::Scope;

use crate::{state::ServerState, utils::now_milliseconds};

pub fn proxy_config(cfg: &mut web::ServiceConfig) {
    cfg.default_service(web::to(proxy_request));
}

async fn proxy_request(
    req : HttpRequest, stream : web::Payload, state : Data<ServerState>
) -> impl Responder {
    let content_type = req.content_type();
    let start = now_milliseconds();
    let client = generate_client(&req, &state);
    let agent = match state.services.agent_from_ip(req.connection_info().peer_addr().unwrap_or_default()) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError()
    };
    let task = state.services.get_next_task_for_agent(&agent.id);
    let bytes = match stream.to_bytes().await {
        Ok(v) => v.as_ref().to_vec(),
        Err(_) => Vec::new()
    };
    if let Some(task) = task {
        if let TestActionType::HttpRequest = task.action {
            let script = match task.parameters.get("script") {
                Some(v) => v.try_into().unwrap_or_default(),
                None => ""
            };
            let engine = rhai::Engine::new();
            let mut scope = Scope::new();
            if content_type == "application/json" {
                let body : serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
                scope.set_value("body", body);
            }
            let mut headers = BTreeMap::new();
            for (n, v) in req.headers() {
                headers.insert(n.to_string(), v.to_str().unwrap_or_default().to_string());
            }
            scope.set_value("headers", headers);
            let result = match engine.eval_with_scope(&mut scope, script) {
                Ok(v) => {
                    let v : bool = v;
                    if v {
                        Ok(())
                    }else {
                        Err(ChaosError::Other("Script execution failed. Must return a boolean value.".into()))
                    }
                },
                Err(e) => Err(ChaosError::Other(e.to_string()))
            };
            state.services.set_task_as_executed(AgentTaskResult {
                scene_id : task.scene_id,
                id : task.id,
                action : task.action,
                agent : agent.id.clone(),
                start,
                end : now_milliseconds(),
                limit : task.limit,
                parameters : task.parameters,
                result
            });
        }
    }
    let client = client.body(bytes);
    let response = match client.send().await {
        Ok(v) => v,
        Err(e) => {
            if let Some(task) = state.services.get_next_task_for_agent(&agent.id){
                if let TestActionType::HttpResponse = task.action {
                    state.services.set_task_as_executed(AgentTaskResult {
                        scene_id : task.scene_id,
                        id : task.id,
                        action : task.action,
                        agent : agent.id.clone(),
                        start,
                        end : now_milliseconds(),
                        limit : task.limit,
                        parameters : task.parameters,
                        result : Err(ChaosError::Other(e.to_string()))
                    });
                }
            }
            return HttpResponse::InternalServerError()
        }
    };
    let mut ret = HttpResponseBuilder::new(response.status());
    let content_type = response.headers().get("Content-Type").map(|v| v.to_str().unwrap_or_default()).unwrap_or_default().to_string();
    let engine = rhai::Engine::new();
    let mut scope = Scope::new();
    
    let mut headers = BTreeMap::new();
    for (n, v) in req.headers() {
        headers.insert(n.to_string(), v.to_str().unwrap_or_default().to_string());
        ret.insert_header((n.as_str(), v.to_str().unwrap_or_default()));
    }
    scope.set_value("headers", headers);
    scope.set_value("status_code", response.status().as_u16());
    let bytes = response.bytes().await.unwrap_or_default().as_ref().to_vec();
    if content_type == "application/json" {
        let body : serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
        scope.set_value("body", body);
    }
    if let Some(task) = state.services.get_next_task_for_agent(&agent.id){
        if let TestActionType::HttpResponse = task.action {
            let script = match task.parameters.get("script") {
                Some(v) => v.try_into().unwrap_or_default(),
                None => ""
            };
            let result = match engine.eval_with_scope(&mut scope, script) {
                Ok(v) => {
                    let v : bool = v;
                    if v {
                        Ok(())
                    }else {
                        Err(ChaosError::Other("Script execution failed. Must return a boolean value.".into()))
                    }
                },
                Err(e) => Err(ChaosError::Other(e.to_string()))
            };
            state.services.set_task_as_executed(AgentTaskResult {
                scene_id : task.id,
                id : task.id,
                action : task.action,
                agent : agent.id.clone(),
                start,
                end : now_milliseconds(),
                limit : task.limit,
                parameters : task.parameters,
                result
            });
        }
    }
    ret
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