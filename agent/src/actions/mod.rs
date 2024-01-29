use chaos_core::{parameters::TestParameters, err::{ChaosResult, ChaosError}, action::{TestActionType, CustomAction}, tasks::AgentTask};

use crate::{common::now_milliseconds, state::AgentState};

pub mod installation;
pub mod service;
pub mod machine;
pub mod workspace;

/// Ejecutar una acción que viene desde el servidor, la idea es que esto produzca un TaskResult que se pueda enviar de vuelta al servidor
/// Además es necesario guardar el estado de la operación en una bbdd local, así como también la sobreescritura de acciones.
pub fn execute_action(action : TestActionType, state : &mut AgentState, task : &mut AgentTask) -> ChaosResult<()> {
    let parameters = state.get_global_parameters();
    let commands = state.get_commands();
    let mut new_parameters = TestParameters::new();
    let mut new_action = TestActionType::Null;
    let (action, parameters) = if let TestActionType::Custom(ca) = action {
        for command in commands {
            if command.name == ca {
                new_action = command.action.to_owned();
                for (name, value) in parameters.inner() {
                    new_parameters.insert(name, value.clone());
                }
                // Override parameters with the ones from custom action
                for (name, value) in command.parameters.inner() {
                    new_parameters.insert(name, value.clone());
                }
                break
            }
        };
        if new_action == TestActionType::Null {
            return Err(ChaosError::Other(format!("Custom action {} not found", ca)))
        }
        (new_action, &new_parameters)
    }else {
        (action, &parameters)
    };
    task.start = now_milliseconds();
    let res = match &action {
        TestActionType::Install => installation::execute_install(parameters),
        TestActionType::Uninstall => installation::execute_uninstall(parameters),
        TestActionType::InstallWithError => installation::execute_install_with_error(parameters),
        TestActionType::RestartService => service::restart_service(parameters),
        TestActionType::StopService => service::stop_service(parameters),
        TestActionType::StartService => service::start_service(parameters),
        TestActionType::ServiceIsRunning => service::service_is_running(parameters),
        TestActionType::RestartHost => machine::restart_host(parameters),
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