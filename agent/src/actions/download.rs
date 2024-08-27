use chaos_core::{action::download::DownloadFileParameters, err::ChaosResult, parameters::TestParameters};

pub fn download_file(
    parameters: &TestParameters
) -> ChaosResult<()> {
    let parameters : DownloadFileParameters = parameters.try_into()?;
    crate::api::download_file_to(&parameters.name, std::path::PathBuf::from(&parameters.location))?;
    Ok(())
}