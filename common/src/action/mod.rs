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

pub mod dns;
pub mod download;
pub mod execute;
pub mod install;
pub mod metrics;
pub mod names;
pub mod service;
pub mod upload;
pub mod wait;
pub mod watchlog;

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub enum TestActionType {
    Package(PackageActionType),
    Service(ServiceActionType),
    Execute(ExecutionActionType),
    Metrics(MetricActionType),
    Http(HttpActionType),
    Log(LogActionType),
    Artifact(ArtifactActionType),
    Dns(DnsActionType),
    RestartHost,
    /// Wait some time
    Wait,
    /// Cleans the temporal folder associated with this test, not the real TMP folder
    CleanTmpFolder,
    CleanAppFolder,
    SetAppEnvVars,
    SetEnvVar,
    DeleteEnvVar,
    ResetAppEnvVars,
    StartUserSession,
    CloseUserSession,
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
            TestActionType::Package(v) => v.into(),
            TestActionType::Dns(v) => v.into(),
            TestActionType::Wait => "Wait",
            TestActionType::RestartHost => "RestartHost",
            TestActionType::Execute(v) => v.into(),
            TestActionType::CleanTmpFolder => "CleanTmpFolder",
            TestActionType::CleanAppFolder => "CleanAppFolder",
            TestActionType::SetAppEnvVars => "SetAppEnvVars",
            TestActionType::SetEnvVar => "SetEnvVar",
            TestActionType::DeleteEnvVar => "DeleteEnvVar",
            TestActionType::ResetAppEnvVars => "ResetAppEnvVars",
            TestActionType::StartUserSession => "StartUserSession",
            TestActionType::CloseUserSession => "CloseUserSession",
            TestActionType::Artifact(v) => v.into(),
            TestActionType::Null => "Null",
            TestActionType::Http(_) => "Http",
            TestActionType::Custom(v) => v.as_str(),
            TestActionType::Service(v) => v.into(),
            TestActionType::Metrics(v) => v.into(),
            TestActionType::Log(v) => v.into(),
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
            "RestartHost" => TestActionType::RestartHost,
            "CleanTmpFolder" => TestActionType::CleanTmpFolder,
            "CleanAppFolder" => TestActionType::CleanAppFolder,
            "SetAppEnvVars" => TestActionType::SetAppEnvVars,
            "SetEnvVar" => TestActionType::SetEnvVar,
            "DeleteEnvVar" => TestActionType::DeleteEnvVar,
            "ResetAppEnvVars" => TestActionType::ResetAppEnvVars,
            "StartUserSession" => TestActionType::StartUserSession,
            "CloseUserSession" => TestActionType::CloseUserSession,
            "Null" => TestActionType::Null,
            "Wait" => TestActionType::Wait,
            _ => from_str_to_test_action_type(value),
        }
    }
}

pub fn from_str_to_test_action_type(value: &str) -> TestActionType {
    match value {
        "RestartHost" => TestActionType::RestartHost,
        "CleanTmpFolder" => TestActionType::CleanTmpFolder,
        "CleanAppFolder" => TestActionType::CleanAppFolder,
        "SetAppEnvVars" => TestActionType::SetAppEnvVars,
        "SetEnvVar" => TestActionType::SetEnvVar,
        "DeleteEnvVar" => TestActionType::DeleteEnvVar,
        "ResetAppEnvVars" => TestActionType::ResetAppEnvVars,
        "StartUserSession" => TestActionType::StartUserSession,
        "CloseUserSession" => TestActionType::CloseUserSession,
        "Null" => TestActionType::Null,
        "Wait" => TestActionType::Wait,
        _ => from_str_to_test_action_type_subpart(value)
            .unwrap_or_else(|| TestActionType::Custom(value.into())),
    }
}

fn from_str_to_test_action_type_subpart(value: &str) -> Option<TestActionType> {
    let mut split_iter = value.split("::");
    let splited = split_iter.next()?;
    Some(match splited {
        "Package" => TestActionType::Package(value.try_into().ok()?),
        "Service" => TestActionType::Service(value.try_into().ok()?),
        "Execute" => TestActionType::Execute(value.try_into().ok()?),
        "Metrics" => TestActionType::Metrics(value.try_into().ok()?),
        "Http" => TestActionType::Http(value.try_into().ok()?),
        "Log" => TestActionType::Log(value.try_into().ok()?),
        "Artifact" => TestActionType::Artifact(value.try_into().ok()?),
        "Dns" => TestActionType::Dns(value.try_into().ok()?),
        _ => return None,
    })
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

pub fn get_obj_field(
    parameters: &TestParameters,
    field: &str,
) -> ChaosResult<BTreeMap<String, TestParameter>> {
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
        matches!(
            self,
            TestActionType::Execute(ExecutionActionType::ServerCommand)
                | TestActionType::Execute(ExecutionActionType::ServerScript)
        )
    }
    /// Action to be performed by the agent
    pub fn is_agent(&self) -> bool {
        !self.is_server()
    }

    /// Action that should be undone. Ex: Uninstall after install
    pub fn undonable(&self) -> bool {
        matches!(
            self,
            TestActionType::Dns(DnsActionType::Add)
                | TestActionType::Package(PackageActionType::Install)
                | TestActionType::Log(LogActionType::Watch)
                | TestActionType::Metrics(MetricActionType::StartMetricsForProcess)
                | TestActionType::Metrics(MetricActionType::StartMetricsForService)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum PackageActionType {
    /// Install the application
    Install,
    /// Uninstall the application
    Uninstall,
    /// Try to install the application, but cant be done
    InstallWithError,
    /// Check that the application is installed
    IsInstalled,
    /// Check that the application is not installed
    IsNotInstalled,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum ServiceActionType {
    /// Restart the application service
    Restart,
    /// Stops the application service
    Stop,
    /// Starts the application service
    Start,
    /// Checks that the service is running
    IsRunning,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum ExecutionActionType {
    /// Execute a command/executable
    Command,
    /// Execute a command/executable in the server
    ServerCommand,
    /// Execute a script
    Script,
    /// Execute a script in the server
    ServerScript,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum MetricActionType {
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
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum HttpActionType {
    /// Wait for the agent to make an HTTP request and apply a script to the request
    Request,
    /// Wait for the agent to make an HTTP request and apply a script to the response
    Response,
    /// Wait to receive a hook
    Hook,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum LogActionType {
    /// Uploads all new lines of a text file
    Watch,
    /// Stops listening for changes in a text file    
    StopWatch,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum ArtifactActionType {
    /// Downloads a file into the agent
    Download,
    /// Uploads a file from the agent to the server
    Upload,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum DnsActionType {
    /// Add a DNS entry in /etc/hosts file
    Add,
    /// Remove a DNS entry in /etc/hosts file
    Remove,
}

impl<'a> From<&'a ArtifactActionType> for &'a str {
    fn from(value: &ArtifactActionType) -> &str {
        match value {
            ArtifactActionType::Download => "Artifact::Download",
            ArtifactActionType::Upload => "Artifact::Upload",
        }
    }
}
impl TryFrom<&str> for ArtifactActionType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Artifact::Download" => ArtifactActionType::Download,
            "Artifact::Upload" => ArtifactActionType::Upload,
            _ => return Err("Invalid Artifact action type"),
        })
    }
}

impl<'a> From<&'a LogActionType> for &'a str {
    fn from(value: &LogActionType) -> &str {
        match value {
            LogActionType::Watch => "Log::WatchLog",
            LogActionType::StopWatch => "Log::StopWatchLog",
        }
    }
}
impl TryFrom<&str> for LogActionType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Log::WatchLog" => LogActionType::Watch,
            "Log::StopWatchLog" => LogActionType::StopWatch,
            _ => return Err("Invalid Log action type"),
        })
    }
}

impl<'a> From<&'a HttpActionType> for &'a str {
    fn from(value: &HttpActionType) -> &str {
        match value {
            HttpActionType::Hook => "Http::Hook",
            HttpActionType::Request => "Http::Request",
            HttpActionType::Response => "Http::Response",
        }
    }
}
impl TryFrom<&str> for HttpActionType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Http::Hook" => HttpActionType::Hook,
            "Http::Request" => HttpActionType::Request,
            "Http::Response" => HttpActionType::Response,
            _ => return Err("Invalid Http action type"),
        })
    }
}

impl<'a> From<&'a MetricActionType> for &'a str {
    fn from(value: &MetricActionType) -> &str {
        match value {
            MetricActionType::StartMetricsForProcess => "Metric::StartMetricsForProcess",
            MetricActionType::StopMetricsForProcess => "Metric::StopMetricsForProcess",
            MetricActionType::UploadProcessMetrics => "Metric::UploadProcessMetrics",
            MetricActionType::StartMetricsForService => "Metric::StartMetricsForService",
            MetricActionType::StopMetricsForService => "Metric::StopMetricsForService",
            MetricActionType::UploadServiceMetrics => "Metric::UploadServiceMetrics",
        }
    }
}
impl TryFrom<&str> for MetricActionType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Metric::StartMetricsForProcess" => MetricActionType::StartMetricsForProcess,
            "Metric::StopMetricsForProcess" => MetricActionType::StopMetricsForProcess,
            "Metric::UploadProcessMetrics" => MetricActionType::UploadProcessMetrics,
            "Metric::StartMetricsForService" => MetricActionType::StartMetricsForService,
            "Metric::StopMetricsForService" => MetricActionType::StopMetricsForService,
            "Metric::UploadServiceMetrics" => MetricActionType::UploadServiceMetrics,
            _ => return Err("Invalid Metric action type"),
        })
    }
}

impl<'a> From<&'a ServiceActionType> for &'a str {
    fn from(value: &ServiceActionType) -> &str {
        match value {
            ServiceActionType::Restart => "Service:Restart",
            ServiceActionType::Stop => "Service:Stop",
            ServiceActionType::Start => "Service:Start",
            ServiceActionType::IsRunning => "Service:IsRunning",
        }
    }
}
impl TryFrom<&str> for ServiceActionType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Service:Restart" => ServiceActionType::Restart,
            "Service:Stop" => ServiceActionType::Stop,
            "Service:Start" => ServiceActionType::Start,
            "Service:IsRunning" => ServiceActionType::IsRunning,
            _ => return Err("Invalid Service action type"),
        })
    }
}

impl<'a> From<&'a ExecutionActionType> for &'a str {
    fn from(value: &ExecutionActionType) -> &str {
        match value {
            ExecutionActionType::Command => "Execute::Command",
            ExecutionActionType::ServerCommand => "Execute::ServerCommand",
            ExecutionActionType::Script => "Execute::Script",
            ExecutionActionType::ServerScript => "Execute::ServerScript",
        }
    }
}
impl TryFrom<&str> for ExecutionActionType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Execute::Command" => ExecutionActionType::Command,
            "Execute::ServerCommand" => ExecutionActionType::ServerCommand,
            "Execute::Script" => ExecutionActionType::Script,
            "Execute::ServerScript" => ExecutionActionType::ServerScript,
            _ => return Err("Invalid Execute action type"),
        })
    }
}

impl<'a> From<&'a PackageActionType> for &'a str {
    fn from(value: &PackageActionType) -> &str {
        match value {
            PackageActionType::Install => "Package::Install",
            PackageActionType::Uninstall => "Package::Uninstall",
            PackageActionType::InstallWithError => "Package::InstallWithError",
            PackageActionType::IsInstalled => "Package::IsInstalled",
            PackageActionType::IsNotInstalled => "Package::IsNotInstalled",
        }
    }
}
impl TryFrom<&str> for PackageActionType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Package::Install" => PackageActionType::Install,
            "Package::Uninstall" => PackageActionType::Uninstall,
            "Package::InstallWithError" => PackageActionType::InstallWithError,
            "Package::IsInstalled" => PackageActionType::IsInstalled,
            "Package::IsNotInstalled" => PackageActionType::IsNotInstalled,
            _ => return Err("Invalid Package action type"),
        })
    }
}

impl<'a> From<&'a DnsActionType> for &'a str {
    fn from(value: &DnsActionType) -> &str {
        match value {
            DnsActionType::Add => "Dns::Add",
            DnsActionType::Remove => "Dns::Remove",
        }
    }
}
impl TryFrom<&str> for DnsActionType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Dns::Add" => DnsActionType::Add,
            "Dns::Remove" => DnsActionType::Remove,
            _ => return Err("Invalid Dns action type"),
        })
    }
}
