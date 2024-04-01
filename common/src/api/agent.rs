use serde::{Serialize, Deserialize};

use crate::{action::CustomAction, parameters::ScenarioParameters, tasks::{AgentTask, AgentTaskResult}, variables::ScenarioVariables};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectAgent {
    pub id : String,
    pub hostname : String,
    pub os : Os,
    pub arch : Arch,
    pub ip : String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Os {
    Windows,
    Linux,
    Mac
}
impl Default for Os {
    fn default() -> Self {
        #[cfg(target_os="windows")]
        {
            Self::Windows
        }
        #[cfg(target_os="linux")]
        {
            Self::Linux
        }
        #[cfg(target_os="macos")]
        {
            Self::Mac
        }
    }
}
impl From<&str> for Os {
    fn from(value : &str) -> Self {
        match value {
            "Windows" => Os::Windows,
            "Linux" => Os::Linux,
            "Mac" => Os::Mac,
            _ => Os::Windows
        }
    }
}
impl From<Os> for &str {
    fn from(value: Os) -> Self {
        match value {
            Os::Windows => "Windows",
            Os::Linux => "Linux",
            Os::Mac => "Mac",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Arch {
    X64,
    X86,
    ARM64
}
impl Default for Arch {
    fn default() -> Self {
        #[cfg(target_arch="x86_64")]
        {
            Self::X64
        }
        #[cfg(target_arch="x86")]
        {
            Self::X86
        }
        #[cfg(target_arch="aarch64")]
        {
            Self::ARM64
        }
    }
}

impl From<&str> for Arch {
    fn from(value : &str) -> Self {
        match value {
            "X64" => Arch::X64,
            "X86" => Arch::X86,
            "ARM64" => Arch::ARM64,
            _ => Arch::X64
        }
    }
}
impl From<Arch> for &str {
    fn from(value: Arch) -> Self {
        match value {
            Arch::X64 => "X64",
            Arch::X86 => "X86",
            Arch::ARM64 => "ARM64",
        }
    }
}

pub fn native_arch_str() -> &'static str {
    #[cfg(target_arch="x86_64")]
    {
        "x86_64"
    }
    #[cfg(target_arch="x86")]
    {
        "x86"
    }
    #[cfg(target_arch="aarch64")]
    {
        "aarch64"
    }
        
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadFileRequest {
    pub filename : String
}

#[derive(Debug, Clone, Serialize)]
pub struct NotifyCompletedTaskReq<'a> {
    pub task : &'a AgentTask
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyCompletedTaskReqOwn {
    pub task : AgentTask
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotifyCompletedTaskRes {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextTaskForAgentReq {
    pub hostname : String,
    pub agent_id : String
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NextTaskForAgentRes {
    pub task : Option<AgentTask>
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppLog {
    pub msg : String,
    pub file : String,
    pub agent : String
}

/// Request from agent to server
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AgentRequest {
    /// Agent uploads a log
    Log(String),
    AppLog(AppLog),
    /// Agent asks for the next task. Requires the hash of the parameters + customactions
    NextTask(u64),
    /// Agent completes a task
    CompleteTask(AgentTaskResult),
    #[default]
    HeartBeat
}
/// Response from server to agent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AgentResponse {
    NextTask(AgentTask),
    CleanTask,
    CustomActions(Vec<CustomAction>),
    Parameters(ScenarioParameters),
    Variables(ScenarioVariables),
    Stop,
    #[default]
    Wait
}