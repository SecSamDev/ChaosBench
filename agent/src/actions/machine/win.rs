use chaos_core::{parameters::TestParameters, err::ChaosResult};
use windows::{Win32::{System::Shutdown::{InitiateShutdownA, SHUTDOWN_RESTART, SHUTDOWN_FORCE_OTHERS, SHUTDOWN_FORCE_SELF, SHTDN_REASON_MINOR_RECONFIG}, Foundation::ERROR_SUCCESS}, core::PCSTR};


pub fn restart_host(_parameters : &TestParameters) -> ChaosResult<()> {
    let ret = unsafe { InitiateShutdownA(PCSTR::null(), PCSTR("Testing with CHAOS\0".as_ptr()), 30, SHUTDOWN_RESTART |SHUTDOWN_FORCE_OTHERS | SHUTDOWN_FORCE_SELF, SHTDN_REASON_MINOR_RECONFIG) };
    if ret == ERROR_SUCCESS.0 {
        return Ok(())
    }
    Err(chaos_core::err::ChaosError::Other(format!("Cannot shutdown system: {}", ret)))
}