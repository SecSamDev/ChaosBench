use chaos_core::api::agent::{Arch, Os};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AgentSchema {
    pub id : String,
    pub hostname : String,
    pub os : Os,
    pub arch : Arch
}