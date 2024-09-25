use std::panic::catch_unwind;

use crate::stopper::wait_for_service_signal;

#[cfg(target_os="linux")]
mod linux;
#[cfg(target_os="windows")]
mod win;

#[cfg(target_os="windows")]
use win::{run as run_servivce, stop_handler_win_service, set_service_status_as_starting, set_service_status_as_stopping, set_service_status_as_running, set_service_status_as_stopped};

#[cfg(target_os="linux")]
use chaos_core::err::ChaosError as Error;
#[cfg(target_os="windows")]
use windows_service::Error as Error;

/// Runs the service
#[cfg(target_os = "windows")]
pub fn run() {
    run_servivce().unwrap();
}

#[cfg(target_os = "linux")]
pub fn run() {
    run_generic().unwrap();
}

fn run_generic() -> Result<(), Error> {
    let (stop_s, stop_r) = std::sync::mpsc::sync_channel(32);
    let signal_s = stop_s.clone();
    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.

    #[cfg(all(not(feature = "no_service"), target_os="windows"))]
    let status_handle = stop_handler_win_service(stop_s)?;

    // Tell the system that service is starting
    #[cfg(all(not(feature = "no_service"), target_os="windows"))]
    set_service_status_as_starting(&status_handle)?;

    // Register system service event handler
    #[cfg(all(not(feature = "no_service"), target_os="windows"))]
    set_service_status_as_starting(&status_handle)?;
    #[cfg(all(not(feature = "no_service"), target_os="windows"))]
    set_service_status_as_running(&status_handle)?;
    let panic = catch_unwind(||  {
        wait_for_service_signal(signal_s, stop_r);
    });
    if panic.is_err() {
        log::error!("Service execution panicked");
    }
    #[cfg(all(not(feature = "no_service"), target_os="windows"))]
    set_service_status_as_stopping(&status_handle)?;
    #[cfg(all(not(feature = "no_service"), target_os="windows"))]
    set_service_status_as_stopped(&status_handle)?;
    Ok(())
}