use serde::{Deserialize, Serialize};

use crate::{parameters::TestParameters, err::ChaosError};

use super::get_string_field;

pub const FILE_LOCATION : &str = "upload_fil_location";
pub const FILE_NAME : &str = "upload_file_name";

/// Download a file from the server
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DownloadFileParameters {
    /// Full path of the file download location
    pub location: String,
    /// Name of the file to be downloaded from the server
    pub name: String
}

impl TryFrom<&TestParameters> for DownloadFileParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let location = get_string_field(params, FILE_LOCATION)?;
        let name = get_string_field(params, FILE_NAME)?;
        Ok(Self {
            location,
            name,
        })
    }
}
impl TryFrom<TestParameters> for DownloadFileParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}