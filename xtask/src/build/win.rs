use std::path::PathBuf;

use signtool::{signtool::SignTool, params::{SignParams, ThumbprintParams, FileCertParams, TimestampUrl}};

use crate::params::BuildParameters;

use super::wix_file;

static AGENT_BINARIES : [&'static str; 1] = ["agent.exe"];
static SERVER_BINARIES : [&'static str; 1] = ["server.exe"];
static USER_BINARIES : [&'static str; 1] = ["user.exe"];


pub fn build_full(params : BuildParameters) {
    let mut a_params = params.clone();
    a_params.target_dir = PathBuf::from(&params.target_dir).join("Agent").to_string_lossy().to_string();
    build_agent(a_params);
    let mut msi_params = params.clone();
    msi_params.target_dir = PathBuf::from(&params.target_dir).join("Agent").to_string_lossy().to_string();
    build_installer(msi_params);
    let mut s_params = params.clone();
    s_params.target_dir = PathBuf::from(&params.target_dir).join("Server").to_string_lossy().to_string();
    build_server(s_params.clone());
    let mut u_params = params.clone();
    u_params.target_dir = PathBuf::from(&params.target_dir).join("User").to_string_lossy().to_string();
    build_user(u_params.clone());
}

pub fn build_server(params : BuildParameters) {
    let target_dir = std::path::PathBuf::from(&params.target_dir);
    super::valid_directory(&target_dir);
    // Build in two phases: Compiling each individual exe sign them and wix them

    for binary in SERVER_BINARIES {
        // Clean up first old binaries. Necessary for signing
        let executable = super::executable_path(binary, &params);
        if executable.exists() {
            let _ = std::fs::remove_file(&executable);
        }
        let executable = super::executable_path_release(binary, &params);
        if executable.exists() {
            let _ = std::fs::remove_file(&executable);
        }
    }

    let project_dir = super::server_dir();
    let args = super::server_args(&params);
    let cargo_build = super::cargo_command(&params, &project_dir, &args);
    assert!(cargo_build.success());
    if !params.sign {
        return
    }
    let signtool : SignTool = SignTool::new().expect("Cannot locate SignTool");
    let sign_params = sign_parameters(&params).expect("Invalid parameters");
    for binary in AGENT_BINARIES {
        let executable = super::executable_path(binary, &params);
        if !executable.exists() {
            panic!("Executable {} not located", binary);
        }
        signtool.sign(&executable, &sign_params).expect("Cannot sign executable");
    }
}

pub fn build_installer(params : BuildParameters) {
    let target_dir = std::path::PathBuf::from(&params.target_dir);
    super::valid_directory(&target_dir);
    // Copy to release
    for binary in AGENT_BINARIES {
        let from = super::executable_path(binary, &params);
        let to = super::executable_path_release(binary, &params);
        if from.exists() {
            let _ = std::fs::copy(from, to);
        }
    }

    let project_dir = super::agent_dir();
    let args = super::installer_args(&params);
    let cargo_build = super::cargo_command(&params, &project_dir, &args);
    assert!(cargo_build.success());
    if !params.sign {
        return
    }
    let signtool : SignTool = SignTool::new().expect("Cannot locate SignTool");
    let sign_params = sign_parameters(&params).expect("Invalid parameters");
    let package_version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let installer_name = format!("agent-{}-{}.msi", package_version, params.architecture);
    let installer = super::msi_path(&installer_name, &params);
    signtool.sign(&installer, &sign_params).expect("Cannot sign executable");
}

pub fn build_agent(params : BuildParameters) {
    let target_dir = std::path::PathBuf::from(&params.target_dir);
    super::valid_directory(&target_dir);
    // Build in two phases: Compiling each individual exe sign them and wix them

    for binary in AGENT_BINARIES {
        // Clean up first old binaries. Necessary for signing
        let executable = super::executable_path(binary, &params);
        if executable.exists() {
            let _ = std::fs::remove_file(&executable);
        }
        let executable = super::executable_path_release(binary, &params);
        if executable.exists() {
            let _ = std::fs::remove_file(&executable);
        }
    }

    let project_dir = super::agent_dir();
    let args = super::agent_args(&params);
    let cargo_build = super::cargo_command(&params, &project_dir, &args);
    assert!(cargo_build.success());
    if !params.sign {
        return
    }
    let signtool : SignTool = SignTool::new().expect("Cannot locate SignTool");
    let sign_params = sign_parameters(&params).expect("Invalid parameters");
    for binary in AGENT_BINARIES {
        let executable = super::executable_path(binary, &params);
        if !executable.exists() {
            panic!("Executable {} not located", binary);
        }
        signtool.sign(&executable, &sign_params).expect("Cannot sign executable");
    }
}

pub fn build_user(params : BuildParameters) {
    let target_dir = std::path::PathBuf::from(&params.target_dir);
    super::valid_directory(&target_dir);
    // Build in two phases: Compiling each individual exe sign them and wix them

    for binary in USER_BINARIES {
        // Clean up first old binaries. Necessary for signing
        let executable = super::executable_path(binary, &params);
        if executable.exists() {
            let _ = std::fs::remove_file(&executable);
        }
        let executable = super::executable_path_release(binary, &params);
        if executable.exists() {
            let _ = std::fs::remove_file(&executable);
        }
    }

    let project_dir = super::user_dir();
    let args = super::user_args(&params);
    let cargo_build = super::cargo_command(&params, &project_dir, &args);
    assert!(cargo_build.success());
    if !params.sign {
        return
    }
    let signtool : SignTool = SignTool::new().expect("Cannot locate SignTool");
    let sign_params = sign_parameters(&params).expect("Invalid parameters");
    for binary in AGENT_BINARIES {
        let executable = super::executable_path(binary, &params);
        if !executable.exists() {
            panic!("Executable {} not located", binary);
        }
        signtool.sign(&executable, &sign_params).expect("Cannot sign executable");
    }
}

pub fn sign_parameters(params : &BuildParameters) -> Result<SignParams, anyhow::Error> {
    let timestamp_url = params.timestamp_url.as_ref().and_then(|v| Some(v.clone().into())).unwrap_or_else(|| TimestampUrl::Comodo);
    if let Some(thumbprint) = &params.certificate_thumbprint {
        return Ok(SignParams::Thumbprint(ThumbprintParams {
            certificate_thumbprint : thumbprint.to_string(),
            digest_algorithm : signtool::params::SignAlgorithm::Sha256,
            timestamp_url
        }))
    }
    if let Some(location) = &params.certificate_location {
        return Ok(SignParams::File(FileCertParams {
            certificate_location : location.clone(),
            certificate_password : params.certificate_password.clone(),
            digest_algorithm : signtool::params::SignAlgorithm::Sha256,
            timestamp_url
        }))
    }
    return Err(anyhow::Error::msg("Cannot parse signing parameters"))
}