use std::{net::IpAddr, str::FromStr};

use serde::{Deserialize, Serialize};
use crate::{err::ChaosError, parameters::TestParameters};

use super::{get_string_field, names::*};

/// Dns parameters: domain and IP address
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DnsParameters {
    /// DNS domain
    pub domain: String,
    /// IP address
    pub ip : String
}

impl TryFrom<&TestParameters> for DnsParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let domain = get_string_field(params, SERVER_DOMAIN)?;
        let ip = get_string_field(params, SERVER_IP)?;
        if let Err(e) = IpAddr::from_str(&ip) {
            return Err(ChaosError::Other(format!("Invalid IP address: {e}")))
        }

        Ok(DnsParameters {
            domain,
            ip,
        })
    }
}
impl TryFrom<TestParameters> for DnsParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}