use serde::{Deserialize, Serialize};

pub mod user_actions;
pub mod agent;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Log {
    pub agent : String,
    pub file : String,
    pub msg : String
}