use serde::{Serialize, Deserialize};

use crate::tasks::AgentTask;

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