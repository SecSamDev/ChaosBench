use std::{fs::create_dir_all, path::PathBuf, rc::Rc, sync::{Arc, Mutex}};

use actix::{Actor, Addr};
use actix_web::{HttpServer, App, web::Data, middleware::Logger};
use actors::logs::LogServer;
use chaos_core::scenario::TestScenario;
use repository::memory::{Database, MemoryRepository};
use rustls::{pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs1KeyDer, PrivatePkcs8KeyDer}, server::{NoClientAuth, ServerConfig}};
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
pub mod utils;

const DEFAULT_SERVER_PORT : u16 = 8080;
pub const SERVER_CERTIFICATE : &[u8] = include_bytes!(env!("SERVER_CERTIFICATE"));
pub const SERVER_KEY : &[u8] = include_bytes!(env!("SERVER_KEY"));
pub const SERVER_PORT : &str = env!("SERVER_PORT");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging();
    let cert_chain = match rustls_pemfile::read_one_from_slice(SERVER_CERTIFICATE).unwrap().unwrap().0 {
        rustls_pemfile::Item::X509Certificate(v) => vec![v],
        _ => panic!("Server certificate invalid"),
    };
    let key_der = match rustls_pemfile::read_one_from_slice(SERVER_KEY).unwrap().unwrap().0 {
        rustls_pemfile::Item::Pkcs1Key(v) => v.into(),
        rustls_pemfile::Item::Pkcs8Key(v) => v.into(),
        rustls_pemfile::Item::Sec1Key(v) => v.into(),
        _ => panic!("Private key format not supported"),
    };
    let config = ServerConfig::builder().with_no_client_auth().with_single_cert(cert_chain, key_der).unwrap();
    let (address, port) = listening_parameters();
    log::info!("Listening on: {}:{}", address, port);
    let scenarios = Arc::new(read_test_scenarios());
    log::info!("Loaded {} scenarios", scenarios.len());
    let database = read_database();
    let db = database.clone();
    let log_server = LogServer::new().start();
    log::info!("Started logserver");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(create_server_state(&scenarios, &database, &log_server)))
            .wrap(Logger::default())
            .configure(controllers::config)
    });
    let _ = server.bind_rustls_0_22((address, port), config)?.run().await;
    //let _ = server.bind((address, port))?.run().await;
    db.lock().unwrap().save();
    Ok(())
}

pub fn listening_parameters() -> (String, u16) {
    let address = std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0".into());
    let port = SERVER_PORT.parse::<u16>().unwrap_or(DEFAULT_SERVER_PORT);
    (address, port)
}

pub fn create_server_state(scenarios : &Arc<Vec<TestScenario>>, db : &Arc<Mutex<Database>>, log_server : &Addr<LogServer>) -> ServerState {
    ServerState {
        services: Rc::new(ProductionService::new(MemoryRepository::new(db, scenarios))),
        scenarios : scenarios.clone(),
        log_server : log_server.clone()
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
                Err(e) => {
                    log::info!("Cannot parse Scenario {}: {}", entry.file_name().to_str().unwrap_or_default(), e);
                    continue;
                }
            };
            ret.push(scenario);
        }
    }
    ret
}

pub fn read_database() -> Arc<Mutex<Database>> {
    Arc::new(Mutex::new(Database::load()))
}