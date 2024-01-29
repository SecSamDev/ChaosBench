#[cfg(target_os="linux")]
mod linux;
#[cfg(target_os="windows")]
mod win;

#[cfg(target_os="windows")]
use win::run as run_servivce;

#[cfg(target_os="linux")]
use linux::run as run_servivce;

/// Runs the service
pub fn run() {
    run_servivce().unwrap();
}