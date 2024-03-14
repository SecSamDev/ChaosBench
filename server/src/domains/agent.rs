use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AgentSchema {
    pub id : String,
    pub hostname : String,
    pub os : Os,
    pub arch : Arch
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum Os {
    #[default]
    Windows,
    Linux,
    Mac
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum Arch {
    #[default]
    X64,
    X86,
    ARM64
}