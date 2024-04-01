use chaos_core::{action::upload::UploadArtifactParameters, err::ChaosResult, parameters::TestParameters};

use crate::api::upload_file;

pub fn upload_artifact(
    parameters: &TestParameters
) -> ChaosResult<()> {
    let parameters : UploadArtifactParameters = parameters.try_into()?;
    upload_file(&parameters.name, std::path::PathBuf::from(&parameters.location))?;
    Ok(())
}