use serde::{Deserialize, Serialize};

use crate::{parameters::TestParameters, err::ChaosError};

use super::get_string_field;

pub const ARTIFACT_LOCATION : &str = "artifact_location";
pub const ARTIFACT_NAME : &str = "artifact_name";

/// Installation parameters: installer msi in windows or package in linux and parameters to be passed to the installer program
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UploadArtifactParameters {
    /// Location of the artifact to be uploaded
    pub location: String,
    /// Name of the artifact to be uploaded to the server
    pub name: String
}

impl TryFrom<&TestParameters> for UploadArtifactParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let location = get_string_field(params, ARTIFACT_LOCATION)?;
        let name = get_string_field(params, ARTIFACT_NAME)?;
        Ok(UploadArtifactParameters {
            location,
            name,
        })
    }
}
impl TryFrom<TestParameters> for UploadArtifactParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}