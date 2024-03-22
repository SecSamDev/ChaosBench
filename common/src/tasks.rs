use crate::{action::TestActionType, err::ChaosError, parameters::TestParameters};

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AgentTask {
    pub id : u32,
    pub scene_id : u32,
    pub agent : String,
    pub limit : i64,
    pub preparation : bool,
    pub action : TestActionType,
    pub parameters : TestParameters
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTaskResult {
    pub id : u32,
    pub scene_id : u32,
    pub agent : String,
    pub start : i64,
    pub end : i64,
    pub limit : i64,
    pub action : TestActionType,
    pub parameters : TestParameters,
    pub result : Result<(), ChaosError>
}

impl From<AgentTask> for AgentTaskResult {
    fn from(v: AgentTask) -> Self {
        AgentTaskResult {
            scene_id : v.scene_id,
            id : v.id,
            action : v.action,
            agent : v.agent,
            end : 0,
            start : 0,
            limit : v.limit,
            parameters : v.parameters,
            result : Ok(())
        }
    }
}