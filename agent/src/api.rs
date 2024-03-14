use std::{cell::RefCell, io::{Read, Write}, path::{Path, PathBuf}, sync::Arc};

use chaos_core::err::{ChaosError, ChaosResult};
use reqwest::Certificate;

use crate::state::{SERVER_ADDRESS, SERVER_PORT};


pub const SERVER_CERTIFICATE : &[u8] = include_bytes!(env!("CA_CERT"));

thread_local! {
    pub static CLIENT: RefCell<reqwest::blocking::Client> = RefCell::new(instance_client().unwrap());
}

fn instance_client() -> ChaosResult<reqwest::blocking::Client> {
    let agent = reqwest::blocking::ClientBuilder::new().user_agent("chaos-agent/1.0.0").https_only(true).use_rustls_tls().add_root_certificate(Certificate::from_pem(SERVER_CERTIFICATE).map_err(|e| ChaosError::Other(format!("Invalid pem for ca.crt: {}", e)))?);
    agent.build().map_err(|e| ChaosError::Other(e.to_string()))
}

pub fn download_file(file_name : &str) -> ChaosResult<PathBuf> {
    let destination = std::env::current_dir().unwrap_or_default().join(file_name);
    let file_url = format!("https://{}:{}/_agent/file/{}", SERVER_ADDRESS,SERVER_PORT, file_name);
    let mut res = CLIENT.with_borrow(|v| {
        v.get(&file_url).send()
    }).map_err(|e| ChaosError::Other(format!("Error getting file {}: {}", file_name, e)))?;
    let mut file = std::fs::File::create(&destination).map_err(|e| ChaosError::Other(format!("Cannot create file {}: {}", destination.to_str().unwrap_or_default(), e)))?;
    let mut buffer = vec![0; 1024];
    loop {
        let readed = res.read(&mut buffer).map_err(|e| ChaosError::Other(format!("Cannot read download file response: {}", e)))?;
        if readed == 0 {
            break
        }
        file.write(&buffer[0..readed]).map_err(|e| ChaosError::Other(format!("Cannot write to downloaded file: {}", e)))?;
    }
    Ok(destination)
}