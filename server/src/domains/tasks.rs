use chaos_core::{action::TestActionType, parameters::TestParameters};

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTask {
    pub id : u32,
    pub agent : String,
    pub start : i64,
    pub end : Option<i64>,
    pub limit : i64,
    pub action : TestActionType,
    pub parameters : TestParameters,
    pub result : Option<Result<(), String>>
}