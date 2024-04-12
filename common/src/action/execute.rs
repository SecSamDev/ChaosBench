use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{err::ChaosError, parameters::TestParameters};

use super::{get_duration_field, get_obj_field, get_string_field, get_timeout_field, get_vec_string_field};

pub const EXECUTABLE : &str = "executable";
pub const EXECUTION_PARAMETERS : &str = "parameters";
pub const EXECUTION_TIMEOUT : &str = "timeout";
pub const EXECUTION_OBJ : &str = "execution";

/// Installation parameters: installer msi in windows or package in linux and parameters to be passed to the installer program
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionParameters {
    /// Location of the aplication to be installed
    pub executable: String,
    /// List of parameters to pass to the installer
    pub parameters: Vec<String>,
    /// 60 seconds by default
    pub timeout: Duration,
}

impl TryFrom<&TestParameters> for ExecutionParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let obj = get_obj_field(params, EXECUTION_OBJ)?;
        let params = TestParameters(obj);
        let executable = get_string_field(&params, EXECUTABLE)?;
        let parameters = get_vec_string_field(&params, EXECUTION_PARAMETERS)?;
        let timeout = get_duration_field(&params, EXECUTION_TIMEOUT).unwrap_or(get_timeout_field(&params).unwrap_or(Duration::from_secs(30)));
        Ok(Self {
            executable,
            parameters,
            timeout
        })
    }
}
impl TryFrom<TestParameters> for ExecutionParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}