use std::{collections::BTreeMap, fmt, time::Duration};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{
    err::ChaosResult,
    parameters::{ScenarioParameters, TestParameter, TestParameters},
};

use self::names::TASK_TIMEOUT;

pub mod install;
pub mod names;
pub mod service;
pub mod wait;
pub mod watchlog;
pub mod upload;
pub mod download;
pub mod metrics;
pub mod execute;

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub enum TestActionType {
    /// Install the application
    Install,
    /// Uninstall the application
    Uninstall,
    /// Try to install the application, but cant be done
    InstallWithError,
    /// Check that the application is installed
    CheckInstalled,
    /// Check that the application is not installed
    CheckNotInstalled,
    /// Restart the application service
    RestartService,
    /// Stops the application service
    StopService,
    StartService,
    /// Checks that the service is running
    ServiceIsRunning,
    RestartHost,
    /// Wait some time
    Wait,
    /// Execute a command in the agent
    Execute,
    /// Execute a command in the server
    ExecuteServer,
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
    HttpRequestInspect,
    /// Wait for the agent to make an HTTP request and apply a script to the response
    HttpResponseInspect,
    /// Downloads a file
    Download,
    /// Uploads all new lines of a text file
    WatchLog,
    /// Stops listening for changes in a text file    
    StopWatchLog,
    /// Uploads a file from the agent to the server
    UploadArtifact,
    /// Starts taking CPU and RAM usage of a process by name
    StartMetricsForProcess,
    /// Stops taking CPU and RAM usage of a process by name
    StopMetricsForProcess,
    /// Uploads metrics of a process
    UploadProcessMetrics,
    /// Starts taking CPU and RAM usage of a service
    StartMetricsForService,
    /// Stops taking CPU and RAM usage of a service
    StopMetricsForService,
    /// Uploads metrics of a service
    UploadServiceMetrics,
    #[default]
    Null,
    Custom(String),
}
impl Serialize for TestActionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.into())
    }
}

impl<'a> From<&'a TestActionType> for &'a str {
    fn from(value: &TestActionType) -> &str {
        match value {
            TestActionType::Install => "Install",
            TestActionType::Wait => "Wait",
            TestActionType::Uninstall => "Uninstall",
            TestActionType::InstallWithError => "InstallWithError",
            TestActionType::CheckInstalled => "CheckInstalled",
            TestActionType::CheckNotInstalled => "CheckNotInstalled",
            TestActionType::RestartService => "RestartService",
            TestActionType::StopService => "StopService",
            TestActionType::StartService => "StartService",
            TestActionType::ServiceIsRunning => "ServiceIsRunning",
            TestActionType::RestartHost => "RestartHost",
            TestActionType::Execute => "Execute",
            TestActionType::ExecuteServer => "ExecuteServer",
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
            TestActionType::HttpRequestInspect => "HttpRequestInspect",
            TestActionType::HttpResponseInspect => "HttpResponseInspect",
            TestActionType::WatchLog => "WatchLog",
            TestActionType::StopWatchLog => "StopWatchLog",
            TestActionType::UploadArtifact => "UploadArtifact",
            TestActionType::StartMetricsForProcess => "StartMetricsForProcess",
            TestActionType::StopMetricsForProcess => "StopMetricsForProcess",
            TestActionType::UploadProcessMetrics => "UploadProcessMetrics",
            TestActionType::StartMetricsForService => "StartMetricsForService",
            TestActionType::StopMetricsForService => "StopMetricsForService",
            TestActionType::UploadServiceMetrics => "UploadServiceMetrics",
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
            "ExecuteServer" => TestActionType::ExecuteServer,
            "CleanTmpFolder" => TestActionType::CleanTmpFolder,
            "CleanAppFolder" => TestActionType::CleanAppFolder,
            "SetAppEnvVars" => TestActionType::SetAppEnvVars,
            "SetEnvVar" => TestActionType::SetEnvVar,
            "DeleteEnvVar" => TestActionType::DeleteEnvVar,
            "ResetAppEnvVars" => TestActionType::ResetAppEnvVars,
            "StartUserSession" => TestActionType::StartUserSession,
            "CloseUserSession" => TestActionType::CloseUserSession,
            "StartMetricsForProcess" => TestActionType::StartMetricsForProcess,
            "StopMetricsForProcess"=> TestActionType::StopMetricsForProcess,
            "UploadProcessMetrics" => TestActionType::UploadProcessMetrics,
            "StartMetricsForService"  => TestActionType::StartMetricsForService,
            "StopMetricsForService" =>   TestActionType::StopMetricsForService,
            "UploadServiceMetrics" => TestActionType::UploadServiceMetrics,
            "Download" => TestActionType::Download,
            "Null" => TestActionType::Null,
            "Wait" => TestActionType::Wait,
            "ServiceIsRunning" => TestActionType::ServiceIsRunning,
            "HttpRequestInspect" => TestActionType::HttpRequestInspect,
            "HttpResponseInspect" => TestActionType::HttpResponseInspect,
            "WatchLog" => TestActionType::WatchLog,
            "StopWatchLog" => TestActionType::StopWatchLog,
            "UploadArtifact" => TestActionType::UploadArtifact,
            _ => TestActionType::Custom(value.to_string()),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct CustomAction {
    pub name: String,
    pub action: TestActionType,
    pub parameters: ScenarioParameters,
}

pub fn get_timeout_field(parameters: &TestParameters) -> ChaosResult<Duration> {
    Ok(parameters
        .get(TASK_TIMEOUT)
        .ok_or(format!("Install parameter {:?} not found", TASK_TIMEOUT))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected String".to_string())?)
}

pub fn get_duration_field(parameters: &TestParameters, field: &str) -> ChaosResult<Duration> {
    Ok(parameters
        .get(field)
        .ok_or(format!("Parameter {:?} not found", field))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected duration string".to_string())?)
}

pub fn get_string_field(parameters: &TestParameters, field: &str) -> ChaosResult<String> {
    Ok(parameters
        .get(field)
        .ok_or(format!("Parameter {:?} not found", field))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected String".to_string())?)
}

pub fn get_obj_field(parameters: &TestParameters, field: &str) -> ChaosResult<BTreeMap<String, TestParameter>> {
    Ok(parameters
        .get(field)
        .ok_or(format!("Parameter {:?} not found", field))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected String".to_string())?)
}

pub fn get_vec_field(parameters: &TestParameters, field: &str) -> ChaosResult<Vec<TestParameter>> {
    Ok(parameters
        .get(field)
        .ok_or(format!("Parameter {:?} not found", field))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected Vec<Param>".to_string())?)
}
pub fn get_vec_string_field(parameters: &TestParameters, field: &str) -> ChaosResult<Vec<String>> {
    Ok(parameters
        .get(field)
        .ok_or(format!("Parameter {:?} not found", field))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected Vec<String>".to_string())?)
}

pub fn get_u64_field(parameters: &TestParameters, field: &str) -> ChaosResult<u64> {
    Ok(parameters
        .get(field)
        .ok_or(format!("Parameter {:?} not found", field))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected String".to_string())?)
}

pub fn get_f32_field(parameters: &TestParameters, field: &str) -> ChaosResult<f32> {
    Ok(parameters
        .get(field)
        .ok_or(format!("Parameter {:?} not found", field))?
        .try_into()
        .map_err(|_| "Invalid parameter type, expected String".to_string())?)
}


impl TestActionType {
    /// Action to be performed by the server
    pub fn is_server(&self) -> bool {
        matches!(self, TestActionType::HttpRequestInspect | TestActionType::HttpResponseInspect | TestActionType::ExecuteServer)
    }
    /// Action to be performed by the agent
    pub fn is_agent(&self) -> bool {
        !self.is_server()
    }
}