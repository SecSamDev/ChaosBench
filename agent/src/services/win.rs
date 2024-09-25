use std::{ffi::OsString, sync::mpsc::SyncSender, time::Duration};
use chaos_core::err::ChaosError;
use windows_service::{service_dispatcher, define_windows_service, service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle}, service::{ServiceControl, ServiceStatus, ServiceType, ServiceState, ServiceControlAccept, ServiceExitCode}};

use crate::common::StopCommand;

#[cfg(not(feature="no_service"))]
define_windows_service!(ffi_service_main, service_main);


#[cfg(not(feature="no_service"))]
pub fn run() -> Result<(), windows_service::Error> {
    log::info!("Running ChaosAgent as service");
    service_dispatcher::start("chaosbench", ffi_service_main)
}

#[cfg(feature="no_service")]
pub fn run() -> Result<(), windows_service::Error> {
    log::info!("Running ChaosAgent as executable");
    service_main(Vec::new())
}

fn service_main(_arguments: Vec<OsString>) {
    if let Err(_e) = super::run_generic() {
        // Handle errors in some way.
    }
}

pub fn stop_handler_win_service(
    stop_handler: SyncSender<StopCommand>,
) -> Result<ServiceStatusHandle, windows_service::Error> {
    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::Preshutdown => {
                log::debug!("Preshutdown received");
                let _ = stop_handler.send(StopCommand::Shutdown);
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Shutdown => {
                log::debug!("Shutdown received");
                let _ = stop_handler.send(StopCommand::Shutdown);
                ServiceControlHandlerResult::NoError
            }
            // Handle stop
            ServiceControl::Stop => {
                log::debug!("Stop received");
                let _ = stop_handler.send(StopCommand::Stop);
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NoError,
        }
    };
    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    service_control_handler::register("chaosbench", event_handler)
}

pub fn set_service_status_as_starting(handler : &ServiceStatusHandle) -> Result<(), windows_service::Error> {
    handler.set_service_status(ServiceStatus {
        service_type : ServiceType::OWN_PROCESS,
        current_state : ServiceState::StartPending,
        checkpoint : 0,
        controls_accepted : ServiceControlAccept::STOP,
        exit_code : ServiceExitCode::NO_ERROR,
        wait_hint : Duration::from_secs_f32(5.0),
        process_id : None
    })
}
pub fn set_service_status_as_stopped(handler : &ServiceStatusHandle) -> Result<(), windows_service::Error> {
    handler.set_service_status(ServiceStatus {
        service_type : ServiceType::OWN_PROCESS,
        current_state : ServiceState::Stopped,
        checkpoint : 0,
        controls_accepted : ServiceControlAccept::empty(),
        exit_code : ServiceExitCode::NO_ERROR,
        wait_hint : Duration::from_secs_f32(5.0),
        process_id : None
    })
}
pub fn set_service_status_as_stopping(handler : &ServiceStatusHandle) -> Result<(), windows_service::Error> {
    handler.set_service_status(ServiceStatus {
        service_type : ServiceType::OWN_PROCESS,
        current_state : ServiceState::StopPending,
        checkpoint : 0,
        controls_accepted : ServiceControlAccept::empty(),
        exit_code : ServiceExitCode::NO_ERROR,
        wait_hint : Duration::from_secs_f32(5.0),
        process_id : None
    })
}
pub fn set_service_status_as_running(handler : &ServiceStatusHandle) -> Result<(), windows_service::Error> {
    handler.set_service_status(ServiceStatus {
        service_type : ServiceType::OWN_PROCESS,
        current_state : ServiceState::Running,
        checkpoint : 0,
        controls_accepted : ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN | ServiceControlAccept::PRESHUTDOWN,
        exit_code : ServiceExitCode::NO_ERROR,
        wait_hint : Duration::from_secs_f32(5.0),
        process_id : None
    })
}