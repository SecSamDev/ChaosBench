use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{err::ChaosError, parameters::TestParameters};

use super::{get_duration_field, get_string_field};

/// Required parameters:
/// watchlog_step: Sleep duration to watch changes of file
/// watchlog_file: Name of the file
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WatchLogParameters {
    /// Location of the file to be watched
    pub file: String,
    /// Watch the file periodically each x seconds
    pub step: Duration,
}


impl TryFrom<&TestParameters> for WatchLogParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let file = get_string_field(params, "watchlog_file")?;
        let step = get_duration_field(params, "watchlog_step").unwrap_or_else(|_| Duration::from_secs(60));

        Ok(Self {
            file,
            step,
        })
    }
}

impl TryFrom<TestParameters> for WatchLogParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}