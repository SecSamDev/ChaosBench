use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{parameters::TestParameters, err::ChaosError};

use super::{get_duration_field, get_f32_field, get_string_field, get_u64_field, names::*};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StartMetricsForService {
    /// Name of the service 
    pub service_name: String,
    /// Frecuency to take samples
    pub sampling_frequency : Duration
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StopMetricsForService {
    /// Name of the service 
    pub service_name: String,
    /// Max CPU permitted for the service in % (Max 1.0 for 1 core)
    pub max_average_cpu : f32,
    /// Max PhysicalMemory permitted for the service in bytes
    pub max_average_ram : u64
}

impl TryFrom<&TestParameters> for StopMetricsForService {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let service_name = get_string_field(params, APP_SERVICE_NAME)?;
        let max_average_ram = get_u64_field(params, "metric_max_avg_ram")?;
        let max_average_cpu = get_f32_field(params, "metric_max_avg_cpu")?;
        Ok(Self {
            service_name,
            max_average_cpu,
            max_average_ram,
        })
    }
}
impl TryFrom<TestParameters> for StopMetricsForService {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}

impl TryFrom<&TestParameters> for StartMetricsForService {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let service_name = get_string_field(params, APP_SERVICE_NAME)?;
        let sampling_frequency = get_duration_field(params, "metric_sample_freq")?;
        Ok(Self {
            service_name,
            sampling_frequency,
        })
    }
}
impl TryFrom<TestParameters> for StartMetricsForService {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}