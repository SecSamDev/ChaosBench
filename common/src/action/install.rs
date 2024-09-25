use std::{collections::BTreeMap, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{parameters::{TestParameter, TestParameters}, err::{ChaosError, ChaosResult}};

use super::{get_string_field, get_timeout_field, names::*};

const SKIP_FIELDS: [&str; 2] = [INSTALLER_LOCATION, INSTALL_ERROR];

/// Installation parameters: installer msi in windows or package in linux and parameters to be passed to the installer program
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InstallParameters {
    /// Location of the aplication to be installed
    pub installer: String,
    /// List of parameters to pass to the installer
    pub parameters: BTreeMap<String, String>,
    /// 60 seconds by default
    pub timeout: Duration,
}

/// Installation parameters: installer msi in windows or package in linux and parameters to be passed to the installer program
/// The installation must fail
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InstallWithErrorParameters {
    /// Location of the aplication to be installed
    pub installer: String,
    /// List of parameters to pass to the installer
    pub parameters: BTreeMap<String, String>,
    pub error: i32,
    /// 60 seconds by default
    pub timeout: Duration,
}

/// Installation check parameters: 
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InstallCheckParameters {
    /// Product Code GUID
    pub product_code: Option<String>,
    /// Name of the product
    pub product_name: String
}


impl TryFrom<&TestParameters> for InstallParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let installer = get_installer_field(params)?;
        let timeout = get_timeout_field(params).unwrap_or_else(|_| Duration::from_secs(60));
        let install_parameters = get_install_parameters_field(params)?;

        let mut parameters = BTreeMap::new();

        for (name, param) in install_parameters {
            if SKIP_FIELDS
                .into_iter()
                .find(|&v| v == name.as_str())
                .is_some()
            {
                continue;
            }
            let str_param: String = param
                .try_into()
                .map_err(|_| "Invalid installer parameter type, expected String".to_string())?;
            parameters.insert(name.clone(), str_param);
        }

        Ok(InstallParameters {
            installer,
            parameters,
            timeout,
        })
    }
}
impl TryFrom<TestParameters> for InstallWithErrorParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}

impl TryFrom<&TestParameters> for InstallWithErrorParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let installer = get_installer_field(params)?;
        let timeout = get_timeout_field(params).unwrap_or_else(|_| Duration::from_secs(60));
        let install_parameters = get_install_parameters_field(params)?;

        let error = get_install_error(params)?;
        
        let mut parameters = BTreeMap::new();

        for (name, param) in install_parameters {
            if SKIP_FIELDS
                .into_iter()
                .find(|&v| v == name.as_str())
                .is_some()
            {
                continue;
            }
            let str_param: String = param
                .try_into()
                .map_err(|_| "Invalid installer parameter type, expected String".to_string())?;
            parameters.insert(name.clone(), str_param);
        }

        Ok(InstallWithErrorParameters {
            installer,
            parameters,
            error,
            timeout,
        })
    }
}
impl TryFrom<TestParameters> for InstallParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}

impl TryFrom<&TestParameters> for InstallCheckParameters {
    type Error = ChaosError;
    fn try_from(params: &TestParameters) -> Result<Self, ChaosError> {
        let product_code = get_string_field(params, "product_code").ok();
        let product_name = get_string_field(params, "product_name")?;

        Ok(InstallCheckParameters {
            product_code,
            product_name
        })
    }
}

impl TryFrom<TestParameters> for InstallCheckParameters {
    type Error = ChaosError;
    fn try_from(value: TestParameters) -> Result<Self, ChaosError> {
        (&value).try_into()
    }
}


pub fn get_installer_field(parameters : &TestParameters) -> ChaosResult<String> {
    Ok(parameters
        .get(INSTALLER_LOCATION)
        .ok_or(format!(
            "Installer name {:?} not found",
            INSTALLER_LOCATION
        ))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected String".to_string())?)
}

pub fn get_install_parameters_field(parameters : &TestParameters) -> ChaosResult<&BTreeMap<String, TestParameter>> {
    Ok(parameters
        .get(INSTALL_PARAMETERS)
        .ok_or(format!(
            "Install parameter list {:?} not found",
            INSTALL_PARAMETERS
        ))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected Obj".to_string())?)
}

pub fn get_install_error(parameters : &TestParameters) -> ChaosResult<i32> {
    Ok(parameters
        .get(INSTALL_ERROR)
        .ok_or(format!(
            "Install error parameter {:?} not found",
            INSTALL_ERROR
        ))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected i32".to_string())?)
}