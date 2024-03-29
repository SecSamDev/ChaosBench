use clap::Parser;

#[derive(Debug, Clone, Default, Parser)]
pub enum Command {
    /// Build only userland service and executables
    BuildAgent(BuildParameters),
    /// Build installer with all
    BuildFull(BuildParameters),
    BuildServer(BuildParameters),
    BuildInstaller(BuildParameters),
    BuildUser(BuildParameters),
    #[default]
    Testing
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum Architecture {
    #[default]
    X64,
    X86,
    ARM64
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum TargetOs {
    #[default]
    Windows,
    Linux,
    MacOS
}

impl std::str::FromStr for Architecture {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        Ok(match &s[..] {
            "x64" => Architecture::X64,
            "x86" => Architecture::X86,
            "arm64" => Architecture::ARM64,
            _ => return Err("Invalid Architecture param")
        })
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Architecture::ARM64 => "arm64",
                Architecture::X64 => "x64",
                Architecture::X86 => "x86"
            })
    }
}

impl std::str::FromStr for TargetOs {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        Ok(match &s[..] {
            "windows" => TargetOs::Windows,
            "linux" => TargetOs::Linux,
            "macos" => TargetOs::MacOS,
            _ => return Err("Invalid TargetOS param")
        })
    }
}
impl std::fmt::Display for TargetOs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                TargetOs::Windows => "Windows",
                TargetOs::MacOS => "MacOS",
                TargetOs::Linux => "Linux"
            })
    }
}

#[derive(Debug, Parser, Default, Clone)]
pub struct BuildParameters {
    #[clap(default_value = "x64", long)]
    pub architecture : Architecture,
    #[clap(default_value = "windows", long)]
    pub target_os : TargetOs,
    #[clap(long)]
    pub version : Option<String>,
    #[clap(long)]
    pub target_dir : String,
    #[clap(default_value = "es", long)]
    pub language : String,
    /// No windows service
    #[clap(default_value = "false", long)]
    pub no_service : bool,
    // ----------------- SIGN executable ------------------
    #[clap(long, default_value = "false")]
    pub sign : bool,
    #[clap(long)]
    pub certificate_thumbprint : Option<String>,
    #[clap(long)]
    pub timestamp_url : Option<String>,
    #[clap(long)]
    pub certificate_location : Option<String>,
    #[clap(long)]
    pub certificate_password : Option<String>,

}