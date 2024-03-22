use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{err::ChaosError, parameters::TestParameters};

use super::get_duration_field;

/// Required parameters:
/// duration: Sleep duration
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WaitParameters {
    /// Watch the file periodically each x seconds
    pub duration: Duration,
}


impl TryFrom<&TestParameters> for WaitParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let wait_duration = get_duration_field(params, "wait_duration")?;
        Ok(Self {
            duration : wait_duration
        })
    }
}

impl TryFrom<TestParameters> for WaitParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}