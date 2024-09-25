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
    Test
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum Architecture {
    #[default]
    X64,
    X86,
    ARM64
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TargetOs {
    Windows,
    Linux,
    MacOS
}

impl Default for TargetOs {
    fn default() -> Self {
        default_target_os()
    }
}

fn default_target_os() -> TargetOs {
    #[cfg(target_os="windows")]
    {
        return TargetOs::Windows
    }
    #[cfg(target_os="linux")]
    {
        return TargetOs::Linux
    }
    #[cfg(target_os="macos")]
    {
        return TargetOs::MacOS
    }
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
    #[clap(default_value_t, long)]
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
    #[clap(default_value = "false", long)]
    pub support_win7 : bool,
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

impl Command {
    pub fn support_win7(&self) -> bool {
        match self {
            Command::BuildAgent(v) => v.support_win7,
            Command::BuildFull(v) => v.support_win7,
            Command::BuildServer(v) => v.support_win7,
            Command::BuildInstaller(v) => v.support_win7,
            Command::BuildUser(v) => v.support_win7,
            Command::Test => false
        }
    }
}