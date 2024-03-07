use serde::{Serialize, Deserialize};

use crate::{action::CustomAction, parameters::TestParameters, tasks::{AgentTask, AgentTaskResult}};

use super::Log;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectAgentRequest {

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
pub struct AgentLogReq {
    pub log : Log,
    pub agent : String
}

/// Request from agent to server
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AgentRequest {
    /// Agent uploads a log
    Log(String),
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
    Parameters(TestParameters),
    Stop,
    #[default]
    Wait
}