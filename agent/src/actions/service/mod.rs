#[cfg(target_os="windows")]
pub mod win;
use chaos_core::{action::ServiceActionType, err::ChaosResult, parameters::TestParameters};
#[cfg(target_os="windows")]
pub use win::*;

#[cfg(target_os="linux")]
pub mod linux;
#[cfg(target_os="linux")]
pub use linux::*;


pub fn service_action(action : &ServiceActionType, parameters: &TestParameters) -> ChaosResult<()> {
    match action {
        ServiceActionType::Restart => restart_service(parameters),
        ServiceActionType::Stop => stop_service(parameters),
        ServiceActionType::Start => start_service(parameters),
        ServiceActionType::IsRunning => service_is_running(parameters)
    }
}