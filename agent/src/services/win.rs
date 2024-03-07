use std::{ffi::OsString, time::Duration};
use windows_service::{service_dispatcher, define_windows_service, service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle}, service::{ServiceControl, ServiceStatus, ServiceType, ServiceState, ServiceControlAccept, ServiceExitCode}};

use crate::{common::StopCommand, stopper::wait_for_service_signal};

#[cfg(not(feature="no_service"))]
define_windows_service!(ffi_service_main, service_main);


pub fn run() -> Result<(), windows_service::Error> {
    // Register generated `ffi_service_main` with the system and start the service, blocking
    // this thread until the service is stopped.
    #[cfg(not(feature="no_service"))]
    run_as_service()?;
    #[cfg(feature="no_service")]
    run_as_executable()?;
    Ok(())
}
#[cfg(not(feature="no_service"))]
fn run_as_service() -> Result<(), windows_service::Error> {
    log::info!("Running ChaosAgent as service");
    service_dispatcher::start("myservice", ffi_service_main)
}

#[cfg(feature="no_service")]
fn run_as_executable() -> Result<(), windows_service::Error> {
    log::info!("Running ChaosAgent as executable");
    run_service(Vec::new())
}

fn service_main(arguments: Vec<OsString>) {
    if let Err(_e) = run_service(arguments) {
        // Handle errors in some way.
    }
}

fn run_service(_arguments: Vec<OsString>) -> Result<(), windows_service::Error> {
    let (shutdown_s, shutdown_r) = std::sync::mpsc::sync_channel(32);
    let signal_s = shutdown_s.clone();
    #[cfg(not(feature="no_service"))]
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // Handle stop event and return control back to the system.
                shutdown_s.send(StopCommand::Stop).unwrap();
                ServiceControlHandlerResult::NoError
            },
            ServiceControl::Shutdown => {
                shutdown_s.send(StopCommand::Shutdown).unwrap();
                ServiceControlHandlerResult::NoError
            },
            // All services must accept Interrogate even if it's a no-op.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };
    #[cfg(not(feature="no_service"))]
    let status_handle = service_control_handler::register("myservice", event_handler)?;

    // Register system service event handler
    #[cfg(not(feature="no_service"))]
    set_service_status_as_starting(&status_handle)?;
    #[cfg(not(feature="no_service"))]
    set_service_status_as_running(&status_handle)?;
    wait_for_service_signal(signal_s, shutdown_r);
    #[cfg(not(feature="no_service"))]
    set_service_status_as_stopping(&status_handle)?;
    #[cfg(not(feature="no_service"))]
    set_service_status_as_stopped(&status_handle)?;
    Ok(())
}

fn set_service_status_as_starting(handler : &ServiceStatusHandle) -> Result<(), windows_service::Error> {
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
fn set_service_status_as_stopped(handler : &ServiceStatusHandle) -> Result<(), windows_service::Error> {
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
fn set_service_status_as_stopping(handler : &ServiceStatusHandle) -> Result<(), windows_service::Error> {
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
fn set_service_status_as_running(handler : &ServiceStatusHandle) -> Result<(), windows_service::Error> {
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