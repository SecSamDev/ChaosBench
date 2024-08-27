use std::path::PathBuf;

use crate::params::BuildParameters;
mod deb;

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

pub fn build_agent(params : BuildParameters) {
    let target_dir = std::path::PathBuf::from(&params.target_dir);
    super::valid_directory(&target_dir);

    let project_dir = super::agent_dir();
    let args = super::agent_args(&params);
    let cargo_build = super::cargo_command(&params, &project_dir, &args);
    assert!(cargo_build.success());
}

pub fn build_server(params : BuildParameters) {
    let target_dir = std::path::PathBuf::from(&params.target_dir);
    super::valid_directory(&target_dir);
    let project_dir = super::server_dir();
    let args = super::server_args(&params);
    let cargo_build = super::cargo_command(&params, &project_dir, &args);
    assert!(cargo_build.success());
}

pub fn build_user(params : BuildParameters) {
    let target_dir = std::path::PathBuf::from(&params.target_dir);
    super::valid_directory(&target_dir);
    let project_dir = super::user_dir();
    let args = super::user_args(&params);
    let cargo_build = super::cargo_command(&params, &project_dir, &args);
    assert!(cargo_build.success());
}

pub fn build_installer(params : BuildParameters) {
    // TODO: build RPM and DEB
    if std::process::Command::new("dpkg-deb").arg("--version").status().is_ok() {
        deb::build_deb_installer(params)
    }
}