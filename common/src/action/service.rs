use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{parameters::TestParameters, err::ChaosError};

use super::{get_string_field, get_timeout_field, names::*};

/// Service parameters to be executed
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServiceCommand {
    /// Name of the service
    pub name: String,
    pub timeout : Duration
}


impl TryFrom<&TestParameters> for ServiceCommand {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let name = get_string_field(params, APP_SERVICE_NAME)?;
        let timeout = get_timeout_field(params).unwrap_or_else(|_| Duration::from_secs(30));
        Ok(ServiceCommand {
            name,
            timeout
        })
    }
}