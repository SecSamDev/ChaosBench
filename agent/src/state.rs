use std::path::Path;

use chaos_core::{
    action::CustomAction,
    api::agent::{NotifyCompletedTaskReq, NotifyCompletedTaskRes, NextTaskForAgentReq, NextTaskForAgentRes},
    err::{ChaosError, ChaosResult},
    parameters::TestParameters,
    tasks::AgentTask,
};
use sled::Db;
use ureq::Agent;

use crate::sys_info::{get_system_uuid, get_hostname};

pub const AGENT_TASK_COLLECTION: &str = "agent_tasks";
pub const GLOBAL_PARAMETERS: &str = "parameters";
pub const CUSTOM_ACTIONS: &str = "custom_actions";
pub const CURRENT_TASK: &str = "current_task";

const SERVER_ADDRESS : &str = env!("AGENT_SERVER_ADDRESS");

/// Save the state of the agent in the database
pub struct AgentState {
    db: Db,
    commands: Vec<CustomAction>,
    actual_task: Option<AgentTask>,
    task_tries: u32,
    client : Agent
}

impl AgentState {
    pub fn new<P>(file: P) -> Self
    where
        P: AsRef<Path>,
    {
        let db = sled::open(file).unwrap();
        let agent = ureq::AgentBuilder::new().user_agent("chaos/1.0.0").build();
        let mut slf = Self {
            db,
            commands: Vec::new(),
            actual_task: None,
            task_tries: 0,
            client : agent
        };
        let commands = slf.get_commands_from_db();
        slf.set_commands(commands);
        let task = slf.get_current_task_from_db();
        slf.set_current_task(task);
        slf
    }

    pub fn set_current_task(&mut self, task: Option<AgentTask>) {
        self.actual_task = task;
        self.task_tries = 0;
        if let Some(task) = &self.actual_task {
            let _ = self
                .db
                .open_tree(AGENT_TASK_COLLECTION)
                .and_then(|v| v.insert(CURRENT_TASK, serde_json::to_vec(task).unwrap_or_default()));
        }
    }

    pub fn get_current_task(&self) -> Option<&AgentTask> {
        self.actual_task.as_ref()
    }

    pub fn get_current_task_from_db(&self) -> Option<AgentTask> {
        let dat = self
            .db
            .open_tree(AGENT_TASK_COLLECTION)
            .ok()?
            .get(CURRENT_TASK)
            .ok()??;
        serde_json::from_slice(&dat).ok()
    }

    pub fn get_global_parameters(&self) -> TestParameters {
        let mut params = TestParameters::new();
        match self.db.open_tree(GLOBAL_PARAMETERS) {
            Ok(v) => {
                for dat in v.iter() {
                    let (key, value) = match dat {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    if let Ok(v) = serde_json::from_slice(&value) {
                        params.insert(&String::from_utf8_lossy(&key), v);
                    }
                }
            }
            Err(_) => return TestParameters::default(),
        };
        params
    }

    pub fn set_commands(&mut self, commands: Vec<CustomAction>) {
        self.commands = commands;
        let db = match self.db.open_tree(CUSTOM_ACTIONS) {
            Ok(v) => v,
            Err(_) => return,
        };
        for cmd in &self.commands {
            if let Ok(v) = serde_json::to_vec(&cmd) {
                let _ = db.insert(&cmd.name, v);
            }
        }
    }

    pub fn get_commands(&self) -> &[CustomAction] {
        &self.commands
    }

    pub fn get_commands_from_db(&self) -> Vec<CustomAction> {
        let mut params = Vec::with_capacity(32);
        match self.db.open_tree(CUSTOM_ACTIONS) {
            Ok(v) => {
                for dat in v.iter() {
                    let (_key, value) = match dat {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    if let Ok(v) = serde_json::from_slice(&value) {
                        params.push(v);
                    }
                }
            }
            Err(_) => return Vec::new(),
        };
        params
    }

    pub fn increase_task_try(&mut self) -> u32 {
        self.task_tries += 1;
        self.task_tries
    }

    pub fn clean_current_task(&mut self) {
        self.actual_task = None;
        self.task_tries = 0;
        let _ = self
            .db
            .open_tree(AGENT_TASK_COLLECTION)
            .and_then(|v| v.remove(CURRENT_TASK));
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
