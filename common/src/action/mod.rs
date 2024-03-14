use std::{fmt, time::Duration};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{err::ChaosResult, parameters::{ScenarioParameters, TestParameters}};

use self::names::TASK_TIMEOUT;

pub mod install;
pub mod names;
pub mod service;

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub enum TestActionType {
    /// Install the application
    Install,
    /// Uninstall the application
    Uninstall,
    /// Try to install the application, but cant be done
    InstallWithError,
    /// Restart the application service
    RestartService,
    /// Stops the application service
    StopService,
    StartService,
    /// Checks that the service is running
    ServiceIsRunning,
    RestartHost,
    Execute,
    /// Cleans the temporal folder associated with this test, not the real TMP folder
    CleanTmpFolder,
    CleanAppFolder,
    SetAppEnvVars,
    SetEnvVar,
    DeleteEnvVar,
    ResetAppEnvVars,
    StartUserSession,
    CloseUserSession,
    /// Wait for the agent to make an HTTP request and apply a script to the request
    HttpRequest,
    /// Wait for the agent to make an HTTP request and apply a script to the response
    HttpResponse,
    /// Downloads a file
    Download,
    #[default]
    Null,
    Custom(String),
}
impl Serialize for TestActionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.serialize_str(self.into())
    }
}

impl<'a> From<&'a TestActionType> for &'a str {
    fn from(value: &TestActionType) -> &str {
        match value {
            TestActionType::Install => "Install",
            TestActionType::Uninstall => "Uninstall",
            TestActionType::InstallWithError => "InstallWithError",
            TestActionType::RestartService => "RestartService",
            TestActionType::StopService => "StopService",
            TestActionType::StartService => "StartService",
            TestActionType::ServiceIsRunning => "ServiceIsRunning",
            TestActionType::RestartHost => "RestartHost",
            TestActionType::Execute => "Execute",
            TestActionType::CleanTmpFolder => "CleanTmpFolder",
            TestActionType::CleanAppFolder => "CleanAppFolder",
            TestActionType::SetAppEnvVars => "SetAppEnvVars",
            TestActionType::SetEnvVar => "SetEnvVar",
            TestActionType::DeleteEnvVar => "DeleteEnvVar",
            TestActionType::ResetAppEnvVars => "ResetAppEnvVars",
            TestActionType::StartUserSession => "StartUserSession",
            TestActionType::CloseUserSession => "CloseUserSession",
            TestActionType::Download => "Download",
            TestActionType::Null => "Null",
            TestActionType::HttpRequest => "HttpRequest",
            TestActionType::HttpResponse => "HttpResponse",
            TestActionType::Custom(v) => v.as_str(),
        }
    }
}
struct TestActionTypeVisitor;

impl<'de> Visitor<'de> for TestActionTypeVisitor {
    type Value = TestActionType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid parameter type")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.into())
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.as_str().into())
    }
}

impl<'de> Deserialize<'de> for TestActionType {
    fn deserialize<D>(deserializer: D) -> Result<TestActionType, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(TestActionTypeVisitor)
    }
}

impl From<&str> for TestActionType {
    fn from(value: &str) -> Self {
        match value {
            "Install" => TestActionType::Install,
            "Uninstall" => TestActionType::Uninstall,
            "InstallWithError" => TestActionType::InstallWithError,
            "RestartService" => TestActionType::RestartService,
            "StopService" => TestActionType::StopService,
            "StartService" => TestActionType::StartService,
            "RestartHost" => TestActionType::RestartHost,
            "Execute" => TestActionType::Execute,
            "CleanTmpFolder" => TestActionType::CleanTmpFolder,
            "CleanAppFolder" => TestActionType::CleanAppFolder,
            "SetAppEnvVars" => TestActionType::SetAppEnvVars,
            "SetEnvVar" => TestActionType::SetEnvVar,
            "DeleteEnvVar" => TestActionType::DeleteEnvVar,
            "ResetAppEnvVars" => TestActionType::ResetAppEnvVars,
            "StartUserSession" => TestActionType::StartUserSession,
            "CloseUserSession" => TestActionType::CloseUserSession,
            "Download" => TestActionType::Download,
            "Null" => TestActionType::Null,
            _ => TestActionType::Custom(value.to_string()),
        }
    }
}


#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct CustomAction {
    pub name : String,
    pub action : TestActionType,
    pub parameters : ScenarioParameters
}


pub fn get_timeout_field(parameters : &TestParameters) -> ChaosResult<Duration> {
    Ok(parameters
    .get(TASK_TIMEOUT)
    .ok_or(format!("Install parameter {:?} not found", TASK_TIMEOUT))?
    .try_into()
    .map_err(|_| "Invalid parameter type, expected String".to_string())?)
}