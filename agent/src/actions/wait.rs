use chaos_core::{
    action::wait::WaitParameters, err::ChaosResult,
    parameters::TestParameters,
};

pub fn wait_agent(
    parameters: &TestParameters
) -> ChaosResult<()> {
    let parameters: WaitParameters = parameters.try_into()?;
    std::thread::sleep(parameters.duration);
    Ok(())
}