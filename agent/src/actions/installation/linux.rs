use chaos_core::{action::install::{InstallParameters, InstallWithErrorParameters}, parameters::TestParameters, err::ChaosResult};

use crate::{api::download_file, common::create_file_path_in_workspace};


pub fn execute_install(parameters: &TestParameters) -> ChaosResult<()> {
    log::info!("Executing install");
    let parameters: InstallParameters = parameters.try_into()?;
    let mut command = std::process::Command::new(r"dpkg");
    let file_location = download_file(&parameters.installer)?;
    command
        .arg("-i")
        .arg(&file_location.to_string_lossy()[..]);
    for (param, value) in &parameters.parameters {
        command.arg(format!("{}={}", param, value));
    }
    let output = match process.output() {
        Ok(v) => v,
        Err(_) => {
            return Err(ChaosResult::Other(format!(
                "Cannot install {}", &parameters.installer
            )))
        }
    };
    let exit_code = output.status.code().map(|v| v as i32).unwrap_or(-1);
    if output.status.success() {
        if output.status.success() {
            log::info!("Installed {}", parameters.installer);
            return Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            log::error!("Error installing (stdout):\n{}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr[..]);
            log::error!("Error installing (stderr):\n{}", stderr);
        }
    }
    Err(ChaosResult::Other(format!(
        "Cannot install {}_ exit_status={}", &parameters.installer, exit_code
    )))
}

pub fn execute_uninstall(parameters: &TestParameters)-> ChaosResult<()> {
    log::info!("Executing uninstall");
    let parameters: InstallParameters = parameters.try_into()?;
    let mut command = std::process::Command::new(r"dpkg");
    let file_location = download_file(&parameters.installer)?;
    command
        .arg("-r")
        .arg(&file_location.to_string_lossy()[..]);
    for (param, value) in &parameters.parameters {
        command.arg(format!("{}={}", param, value));
    }
    let output = match process.output() {
        Ok(v) => v,
        Err(_) => {
            return Err(ChaosResult::Other(format!(
                "Cannot uninstall {}", &parameters.installer
            )))
        }
    };
    let exit_code = output.status.code().map(|v| v as i32).unwrap_or(-1);
    if output.status.success() {
        if output.status.success() {
            log::info!("Uninstalled {}", parameters.installer);
            return Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout[..]);
            log::error!("Error uninstalling (stdout):\n{}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr[..]);
            log::error!("Error uninstalling (stderr):\n{}", stderr);
        }
    }
    Err(ChaosResult::Other(format!(
        "Cannot uninstall {}_ exit_status={}", &parameters.installer, exit_code
    )))
}