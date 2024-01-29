use std::{path::PathBuf, fs::create_dir_all, sync::Arc, rc::Rc};

use actix_web::{HttpServer, App, web::Data, middleware::Logger};
use chaos_core::scenario::TestScenario;
use services::production::ProductionService;
use state::ServerState;
use telemetry::init_logging;
pub mod controllers;
pub mod domains;
pub mod state;
pub mod telemetry;
pub mod services;
pub mod repository;
pub mod actors;

const DEFAULT_SERVER_PORT: u16 = 8080;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    let (address, port) = listening_parameters();
    log::info!("Listening on: {}:{}", address, port);
    let scenarios = Arc::new(read_test_scenarios());
    log::info!("Loaded {} scenarios", scenarios.len());
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(create_server_state(&scenarios)))
            .wrap(Logger::default())
            .configure(controllers::config)
    });
    let _ = server.bind((address, port))?.run().await;
    Ok(())
}

pub fn listening_parameters() -> (String, u16) {
    let address = std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("SERVER_PORT")
        .map(|v| v.parse::<u16>().unwrap_or(DEFAULT_SERVER_PORT))
        .unwrap_or_else(|_| DEFAULT_SERVER_PORT);
    (address, port)
}

pub fn create_server_state(scenarions : &Arc<Vec<TestScenario>>) -> ServerState {
    ServerState {
        services: Rc::new(ProductionService::new()),
        scenarios : scenarions.clone()
    }
}

pub fn read_test_scenarios() -> Vec<TestScenario> {
    let location = match std::env::var("SCENARIOS") {
        Ok(v) => PathBuf::from(v),
        Err(_) => std::env::current_dir().unwrap().join("scenarios")
    };
    if !location.exists() {
        std::fs::create_dir_all(&location).expect("Scenario location must be created");
    }
    let mut ret = Vec::with_capacity(32);
    for entry in location.read_dir().expect("Must have read permissions over scenarios folder") {
        if let Ok(entry) = entry {
            let content = std::fs::read_to_string(entry.path()).expect("Scenario must be readable");
            let scenario = match serde_yaml::from_str(&content) {
                Ok(v) => v,
                Err(_) => {
                    println!("Cannot parse Scenario {}", entry.file_name().to_str().unwrap_or_default());
                    continue;
                }
            };
            ret.push(scenario);
        }
    }
    ret
}