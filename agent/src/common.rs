use std::{path::PathBuf, time::{UNIX_EPOCH, SystemTime}};

use chaos_core::{action::TestActionType, err::ChaosError, parameters::TestParameters, tasks::AgentTaskResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AgentTaskInternal {
    pub id : u32,
    pub scene_id : u32,
    pub agent : String,
    pub limit : i64,
    pub start : i64,
    pub end : Option<i64>,
    pub action : TestActionType,
    pub parameters : TestParameters,
    pub result : Option<Result<(), ChaosError>>,
    pub retries : u32,
}

pub enum StopCommand {
    Shutdown,
    Stop
}

pub fn now_milliseconds() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

#[cfg(target_os="windows")]
pub fn get_home() -> PathBuf {
    PathBuf::from(r"C:\ProgramData\ChaosBench")
}

pub fn set_home() {
    let home = get_home();
    if !home.exists() {
        std::fs::create_dir_all(&home).unwrap();
    }
    std::env::set_current_dir(home).expect("Must configure current dir for agent");
}
/// Creates a new file in the workspace
pub fn create_file_path_in_workspace(filename : &str) -> PathBuf {
    std::env::current_dir().unwrap().join(filename)
}
/// Creates a new file in the temp workspace
pub fn create_file_in_temp(filename : &str) -> PathBuf {
    std::env::current_dir().unwrap().join("temp").join(filename)
}
/// Creates a new file in the APP temp workspace
pub fn create_file_in_app_temp(filename : &str) -> PathBuf {
    std::env::current_dir().unwrap().join("app_temp").join(filename)
}
/// Creates a new file in the User temp workspace
pub fn create_file_in_user_temp(filename : &str) -> PathBuf {
    std::env::current_dir().unwrap().join("user_temp").join(filename)
}

impl From<AgentTaskInternal> for AgentTaskResult {
    fn from(v: AgentTaskInternal) -> Self {
        AgentTaskResult {
            scene_id : v.scene_id,
            action : v.action,
            agent : v.agent,
            end : v.end.unwrap_or_default(),
            id : v.id,
            limit : v.limit,
            parameters : v.parameters,
            result : v.result.unwrap_or_else(|| Ok(())),
            start : v.start,
            retries : v.retries,
        }
    }
}

impl From<&AgentTaskInternal> for AgentTaskResult {
    fn from(v: &AgentTaskInternal) -> Self {
        AgentTaskResult {
            scene_id : v.id,
            action : v.action.clone(),
            agent : v.agent.clone(),
            end : v.end.unwrap_or_default(),
            id : v.id,
            limit : v.limit,
            parameters : v.parameters.clone(),
            result : v.result.clone().unwrap_or_else(|| Ok(())),
            start : v.start,
            retries : v.retries
        }
    }
}