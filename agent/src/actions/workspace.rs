use chaos_core::err::ChaosResult;

/// This modifies the env vars for the application that is going to be tested.
/// It modifies the TEMP and TMP to be executed on the agent test temp folder
pub fn inyect_testing_env_vars(_env_vars : &[(&str, &str)]) -> ChaosResult<()> {
    // TODO: Save old environment key from HKLM\SYSTEM\CurrentControlSet\Services.
    Ok(())
}

pub fn clean_testing_env_vars(_old_vars : &[(&str, &str)]) -> ChaosResult<()> {
    // removes or sets the old environment key
    Ok(())
}