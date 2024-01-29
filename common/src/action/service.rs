use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{parameters::TestParameters, err::{ChaosResult, ChaosError}};

use super::{names::*, get_timeout_field};

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
        let name = get_service_name(params)?;
        let timeout = get_timeout_field(params).unwrap_or_else(|_| Duration::from_secs(30));
        Ok(ServiceCommand {
            name,
            timeout
        })
    }
}

pub fn get_service_name(parameters : &TestParameters) -> ChaosResult<String> {
    Ok(parameters
        .get(APP_SERVICE_NAME)
        .ok_or(format!(
            "Service name {:?} not found",
            APP_SERVICE_NAME
        ))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected String".to_string())?)
}