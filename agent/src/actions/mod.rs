use chaos_core::{parameters::TestParameters, err::{ChaosResult, ChaosError}, action::TestActionType, tasks::AgentTask};

use crate::{common::{now_milliseconds, AgentTaskInternal}, state::AgentState};

pub mod installation;
pub mod service;
pub mod machine;
pub mod workspace;
pub mod watchlog;
pub mod wait;

/// Ejecutar una acción que viene desde el servidor, la idea es que esto produzca un TaskResult que se pueda enviar de vuelta al servidor
/// Además es necesario guardar el estado de la operación en una bbdd local, así como también la sobreescritura de acciones.
pub fn execute_action(origin_action : TestActionType, state : &mut AgentState, task : &mut AgentTaskInternal) -> ChaosResult<()> {
    let global_parameters = state.db.get_global_parameters();
    let commands = state.db.get_commands();
    let mut parameters: TestParameters = global_parameters.into();
    let mut action = origin_action.clone();
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
        TestActionType::Install => installation::execute_install(&parameters),
        TestActionType::Uninstall => installation::execute_uninstall(&parameters),
        TestActionType::InstallWithError => installation::execute_install_with_error(&parameters),
        TestActionType::RestartService => service::restart_service(&parameters),
        TestActionType::StopService => service::stop_service(&parameters),
        TestActionType::StartService => service::start_service(&parameters),
        TestActionType::ServiceIsRunning => service::service_is_running(&parameters),
        TestActionType::RestartHost => machine::restart_host(&parameters),
        TestActionType::Execute => Ok(()),
        TestActionType::CleanTmpFolder => Ok(()),
        TestActionType::CleanAppFolder => Ok(()),
        TestActionType::SetAppEnvVars => Ok(()),
        TestActionType::SetEnvVar => Ok(()),
        TestActionType::DeleteEnvVar => Ok(()),
        TestActionType::ResetAppEnvVars => Ok(()),
        TestActionType::StartUserSession => Ok(()),
        TestActionType::CloseUserSession => Ok(()),
        TestActionType::Download => Ok(()),
        TestActionType::Null => Ok(()),
        TestActionType::HttpRequest => Ok(()),
        TestActionType::HttpResponse => Ok(()),
        TestActionType::Wait => wait::wait_agent(&parameters),
        TestActionType::WatchLog => watchlog::start_listening_to_file_changes(&parameters, state),
        TestActionType::WatchLogStop => watchlog::stop_listening_to_file_changes(&parameters),
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