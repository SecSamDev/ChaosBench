use std::{io::{Read, Write}, path::PathBuf};

use chaos_core::{action::metrics::MetricsArtifact, err::{ChaosError, ChaosResult}};
use reqwest::{header::{HeaderMap, HeaderValue, CONTENT_TYPE}, Certificate};

use crate::{state::{SERVER_ADDRESS, SERVER_PORT}, sys_info::get_system_uuid};


pub const SERVER_CERTIFICATE : &[u8] = include_bytes!(env!("CA_CERT"));

fn instance_client() -> ChaosResult<reqwest::blocking::Client> {
    let agent = reqwest::blocking::ClientBuilder::new().user_agent("chaos-agent/1.0.0").https_only(true).use_rustls_tls().add_root_certificate(Certificate::from_pem(SERVER_CERTIFICATE).map_err(|e| ChaosError::Other(format!("Invalid pem for ca.crt: {}", e)))?);
    let mut headers = HeaderMap::new();
    headers.insert("Agent-ID", HeaderValue::from_str(get_system_uuid()?.as_str()).unwrap());
    agent.default_headers(headers).build().map_err(|e| ChaosError::Other(e.to_string()))
}

pub fn download_file(file_name : &str) -> ChaosResult<PathBuf> {
    log::info!("Downloading {}", file_name);
    let destination = std::env::current_dir().unwrap_or_default().join(file_name);
    let file_url = format!("https://{}:{}/_agent/file/{}", SERVER_ADDRESS,SERVER_PORT, file_name);
    let client = instance_client()?;
    let res = client.get(&file_url).send().map_err(|e| ChaosError::Other(format!("Error getting file {}: {}", file_name, e)))?;
    let mut res = match res.error_for_status() {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Error getting file {}: {}", file_name, e)))
    };
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
pub fn download_file_to(file_name : &str, destination : PathBuf) -> ChaosResult<PathBuf> {
    log::info!("Downloading {}", file_name);
    let file_url = format!("https://{}:{}/_agent/file/{}", SERVER_ADDRESS,SERVER_PORT, file_name);
    let client = instance_client()?;
    let res = client.get(&file_url).send().map_err(|e| ChaosError::Other(format!("Error getting file {}: {}", file_name, e)))?;
    let mut res = match res.error_for_status() {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Error getting file {}: {}", file_name, e)))
    };
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

pub fn upload_file(file_name : &str, location : PathBuf) -> ChaosResult<()> {
    let file_url = format!("https://{}:{}/_agent/file/{}", SERVER_ADDRESS, SERVER_PORT, file_name);
    let mut file = std::fs::File::open(&location).map_err(|e| ChaosError::Other(format!("Cannot open file {}: {}", location.to_str().unwrap_or_default(), e)))?;
    let mut buffer = Vec::with_capacity(100_000);
    file.read_to_end(&mut buffer).map_err(|e| ChaosError::Other(format!("Cannot read file {}: {}", location.to_str().unwrap_or_default(), e)))?;
    let client = instance_client()?;
    let res = client.post(&file_url).body(buffer).send().map_err(|e| ChaosError::Other(format!("Error uploading file {}: {}", file_name, e)))?;
    let _res = match res.error_for_status() {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Error getting file {}: {}", file_name, e)))
    };
    Ok(())
}

pub fn upload_data(file_name : &str, content : Vec<u8>) -> ChaosResult<()> {
    let file_url = format!("https://{}:{}/_agent/file/{}", SERVER_ADDRESS, SERVER_PORT, file_name);
    let client = instance_client()?;
    let res = client.post(&file_url).body(content).send().map_err(|e| ChaosError::Other(format!("Error uploading data {}: {}", file_name, e)))?;
    let _res = match res.error_for_status() {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Error getting file {}: {}", file_name, e)))
    };
    Ok(())
}

pub fn upload_metric(metric_name : &str, content : &MetricsArtifact) -> ChaosResult<()> {
    let file_url = format!("https://{}:{}/_agent/metric/{}", SERVER_ADDRESS, SERVER_PORT, metric_name);
    let client = instance_client()?;
    let res = client.post(&file_url).header(CONTENT_TYPE, "application/json").body(serde_json::to_vec(content).unwrap_or_default()).send().map_err(|e| ChaosError::Other(format!("Error uploading metrics {}: {}", metric_name, e)))?;
    let _res = match res.error_for_status() {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Error uploading metrics {}: {}", metric_name, e)))
    };
    Ok(())
}