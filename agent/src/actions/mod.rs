use std::time::Duration;

use chaos_core::{action::{wait::WaitParameters, ArtifactActionType, TestActionType}, err::{ChaosError, ChaosResult}, parameters::TestParameters};
use execute::command_execution_action;

use crate::{common::{now_milliseconds, AgentTaskInternal}, state::AgentState};

pub mod installation;
pub mod service;
pub mod machine;
pub mod workspace;
pub mod watchlog;
pub mod upload;
pub mod metrics;
pub mod download;
pub mod execute;
pub mod dns;

/// Ejecutar una acción que viene desde el servidor, la idea es que esto produzca un TaskResult que se pueda enviar de vuelta al servidor
/// Además es necesario guardar el estado de la operación en una bbdd local, así como también la sobreescritura de acciones.
pub fn execute_action(origin_action : TestActionType, state : &mut AgentState, task : &mut AgentTaskInternal) -> ChaosResult<()> {
    let global_parameters = state.db.get_global_parameters();
    let commands = state.db.get_commands();
    let mut parameters: TestParameters = global_parameters.into();
    let mut action = origin_action.clone();
    task.retries -= 1;
    if let TestActionType::Custom(ca) = origin_action {
        for command in commands {
            if command.name == ca {
                action = command.action.to_owned();
                // Override parameters with the ones from custom action
                let cmd_params : TestParameters = (&command.parameters).into();
                for (name, value) in cmd_params.inner() {
                    parameters.insert(name, value.clone());
                }
                break
            }
        };
        if action == TestActionType::Null {
            return Err(ChaosError::Other(format!("Custom action {} not found", ca)))
        }
    }
    parameters.replace_with_vars(state.db.get_variables());
    let res = match &action {
        TestActionType::Package(action) => installation::package_action(action, &parameters),
        TestActionType::Service(action) => service::service_action(action, &parameters),
        TestActionType::Metrics(action) => metrics::metric_action(action, &parameters),
        TestActionType::Log(action) => watchlog::watchlog_action(action, &parameters, state),
        TestActionType::Dns(action) => dns::dns_action(action, &parameters),
        TestActionType::Http(_) => Ok(()),
        TestActionType::Artifact(action) => match action {
            ArtifactActionType::Download => download::download_file(&parameters),
            ArtifactActionType::Upload => upload::upload_artifact(&parameters),
        },
        TestActionType::RestartHost => machine::restart_host(&parameters),
        TestActionType::Execute(action) => {
            // Return if task has not finished
            match command_execution_action(action, task.id, &parameters) {
                Some(v) => v,
                None => return Ok(())
            }
        },
        TestActionType::CleanTmpFolder => Ok(()),
        TestActionType::CleanAppFolder => Ok(()),
        TestActionType::SetAppEnvVars => Ok(()),
        TestActionType::SetEnvVar => Ok(()),
        TestActionType::DeleteEnvVar => Ok(()),
        TestActionType::ResetAppEnvVars => Ok(()),
        TestActionType::StartUserSession => Ok(()),
        TestActionType::CloseUserSession => Ok(()),
        TestActionType::Null => Ok(()),
        TestActionType::Wait => {
            task.retries += 1;
            let parameters: WaitParameters = parameters.try_into()?;
            let elapsed = (now_milliseconds() - task.start).max(0).abs();
            let duration_millis = parameters.duration.as_millis() as i64;
            let remaining = duration_millis - elapsed;
            if remaining > 0 {
                std::thread::sleep(Duration::from_millis(remaining.max(100) as u64));
                return Ok(())
            }
            Ok(())
        },
        TestActionType::Custom(action) => Err(chaos_core::err::ChaosError::Other(format!("Custom action {} not found", action))),
        
    };
    task.result = Some(res);
    task.end = Some(now_milliseconds());
    if TestActionType::RestartHost == action {
        task.end = None;
        task.result = None;
    }
    Ok(())
}