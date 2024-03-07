use std::{path::Path, time::Duration};

use chaos_core::{
    api::agent::{NotifyCompletedTaskReq, NotifyCompletedTaskRes, NextTaskForAgentReq, NextTaskForAgentRes},
    err::{ChaosError, ChaosResult},
    tasks::AgentTask,
};
use ureq::Agent;

use crate::{db::Database, sys_info::{get_hostname, get_system_uuid}};

pub const AGENT_TASK_COLLECTION: &str = "agent_tasks";
pub const GLOBAL_PARAMETERS: &str = "parameters";
pub const CUSTOM_ACTIONS: &str = "custom_actions";
pub const CURRENT_TASK: &str = "current_task";

pub const SERVER_ADDRESS : &str = env!("AGENT_SERVER_ADDRESS");

/// Save the state of the agent in the database
pub struct AgentState {
    pub db: Database,
    task_tries: u32,
    client : Agent
}

impl AgentState {
    pub fn new() -> Self {
        let db = Database::load();
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs_f32(20.0)).user_agent("chaos/1.0.0").build();
        Self {
            db,
            task_tries: 0,
            client : agent
        }
    }

    pub fn increase_task_try(&mut self) -> u32 {
        self.task_tries += 1;
        self.task_tries
    }

    pub fn notify_completed_task(&self, task: &AgentTask) -> ChaosResult<()> {
        let req = NotifyCompletedTaskReq { task };
        let url = format!("http://{}/agent/task", SERVER_ADDRESS);
        let _response: NotifyCompletedTaskRes = self.client.post(&url)
            .send_json(&req)
            .map_err(|e| ChaosError::Other(format!("{:?}", e)))?
            .into_json()
            .map_err(|e| ChaosError::Other(format!("{:?}", e)))?;
        Ok(())
    }

    pub fn get_next_task(&self) -> Option<AgentTask> {
        let url = format!("http://{}/agent/next_task", SERVER_ADDRESS);
        let req = NextTaskForAgentReq {
            hostname: get_hostname().unwrap_or_else(|_| "InvalidHostname".into()),
            agent_id: get_system_uuid().unwrap_or_else(|_| "InvalidUuid".into()),
        };
        let response: NextTaskForAgentRes = self.client.post(&url)
            .send_json(&req)
            .map_err(|e| ChaosError::Other(format!("{:?}", e))).ok()?
            .into_json()
            .map_err(|e| ChaosError::Other(format!("{:?}", e))).ok()?;

        response.task
    }
    
}