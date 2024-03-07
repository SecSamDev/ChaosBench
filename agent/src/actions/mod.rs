use chaos_core::{parameters::TestParameters, err::{ChaosResult, ChaosError}, action::TestActionType, tasks::AgentTask};

use crate::{common::now_milliseconds, state::AgentState};

pub mod installation;
pub mod service;
pub mod machine;
pub mod workspace;

/// Ejecutar una acción que viene desde el servidor, la idea es que esto produzca un TaskResult que se pueda enviar de vuelta al servidor
/// Además es necesario guardar el estado de la operación en una bbdd local, así como también la sobreescritura de acciones.
pub fn execute_action(origin_action : TestActionType, state : &mut AgentState, task : &mut AgentTask) -> ChaosResult<()> {
    let global_parameters = state.db.get_global_parameters();
    let commands = state.db.get_commands();
    let mut parameters = global_parameters.clone();
    let mut action = origin_action.clone();
    if let TestActionType::Custom(ca) = origin_action {
        for command in commands {
            if command.name == ca {
                action = command.action.to_owned();
                // Override parameters with the ones from custom action
                for (name, value) in command.parameters.inner() {
                    parameters.insert(name, value.clone());
                }
                break
            }
        };
        if action == TestActionType::Null {
            return Err(ChaosError::Other(format!("Custom action {} not found", ca)))
        }
    }
    task.start = now_milliseconds();
    let res = match &action {
        TestActionType::Install => installation::execute_install(&parameters),
        TestActionType::Uninstall => installation::execute_uninstall(&parameters),
        TestActionType::InstallWithError => installation::execute_install_with_error(&parameters),
        TestActionType::RestartService => service::restart_service(&parameters),
        TestActionType::StopService => service::stop_service(&parameters),
        TestActionType::StartService => service::start_service(&parameters),
        TestActionType::ServiceIsRunning => service::service_is_running(&parameters),
        TestActionType::RestartHost => machine::restart_host(&parameters),
        TestActionType::Execute => todo!(),
        TestActionType::CleanTmpFolder => todo!(),
        TestActionType::CleanAppFolder => todo!(),
        TestActionType::SetAppEnvVars => todo!(),
        TestActionType::SetEnvVar => todo!(),
        TestActionType::DeleteEnvVar => todo!(),
        TestActionType::ResetAppEnvVars => todo!(),
        TestActionType::StartUserSession => todo!(),
        TestActionType::CloseUserSession => todo!(),
        TestActionType::Download => todo!(),
        TestActionType::Null => Ok(()),
        TestActionType::Custom(action) => Err(chaos_core::err::ChaosError::Other(format!("Custom action {} not found", action))),
    };
    task.result = Some(match res {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?}",e))
    });
    task.end = Some(now_milliseconds());
    if TestActionType::RestartHost == action {
        task.end = None;
        task.result = None;
    }
    Ok(())
}