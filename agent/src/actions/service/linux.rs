use std::{process::{Child, Command}, time::Duration};

use chaos_core::{action::service::ServiceCommand, err::{ChaosError, ChaosResult}, parameters::TestParameters};

use crate::common::{now_milliseconds, spawn_child_and_check_return_code};

pub fn stop_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: ServiceCommand = parameters.try_into()?;
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("stop").arg(parameters.name);
    spawn_child_and_check_return_code(cmd, parameters.timeout, "Cannot stop service using systemctl")
}

pub fn start_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: ServiceCommand = parameters.try_into()?;
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("start").arg(parameters.name);
    spawn_child_and_check_return_code(cmd, parameters.timeout, "Cannot start service using systemctl")
}

pub fn restart_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: ServiceCommand = parameters.try_into()?;
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("restart").arg(parameters.name);
    spawn_child_and_check_return_code(cmd, parameters.timeout, "Cannot restart service using systemctl")
}

pub fn service_is_running(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: ServiceCommand = parameters.try_into()?;
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("is-active").arg("--quiet").arg(parameters.name);
    spawn_child_and_check_return_code(cmd, parameters.timeout, "Cannot test service using systemctl")
}