use chaos_core::{err::ChaosResult, parameters::TestParameters};
use nix::libc::{reboot, sync, RB_AUTOBOOT};

pub fn restart_host(_parameters : &TestParameters) -> ChaosResult<()> {
    unsafe { sync() };
    unsafe { reboot(RB_AUTOBOOT) };
    Ok(())
}