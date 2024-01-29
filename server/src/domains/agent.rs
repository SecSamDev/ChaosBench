pub struct AgentSchema {
    pub id : String,
    pub hostname : String,
    pub os : Os,
    pub arch : Arch
}

pub enum Os {
    Windows,
    Linux,
    Mac
}

pub enum Arch {
    X64,
    X86,
    ARM64
}