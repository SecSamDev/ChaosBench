use std::path::PathBuf;
use std::process::ExitStatus;

use crate::params::BuildParameters;
#[cfg(target_os="linux")]
pub mod linux;
#[cfg(target_os="linux")]
use linux::{build_full as build_full_os, build_agent as build_agent_os, build_server as build_server_os, build_installer as build_installer_os, build_user as build_user_os};

#[cfg(target_os="windows")]
pub mod win;
#[cfg(target_os="windows")]
use win::{build_server as build_server_os, build_full as build_full_os, build_agent as build_agent_os, build_installer as build_installer_os, build_user as build_user_os};

pub fn build_full(params : BuildParameters) {
    println!("Building full project");
    build_full_os(params);
}

pub fn build_server(params : BuildParameters) {
    build_server_os(params);
}

pub fn build_agent(params : BuildParameters) {
    build_agent_os(params);
}

pub fn build_installer(params : BuildParameters) {
    build_installer_os(params);
}

pub fn build_user(params : BuildParameters) {
    build_user_os(params);
}

pub fn valid_directory(dir : &PathBuf) -> bool {
    dir.has_root() && dir.is_dir()
}

pub fn agent_version(params : &BuildParameters) -> String {
    if let Some(version) = &params.version {
        std::env::set_var("CARGO_PKG_VERSION", version);
    }
    std::env::var("CARGO_PKG_VERSION").unwrap()
}

pub fn cargo_target(params : &BuildParameters) -> &'static str {
    match params.target_os {
        crate::params::TargetOs::Windows => cargo_target_win(params),
        crate::params::TargetOs::Linux => cargo_target_linux(params),
        crate::params::TargetOs::MacOS => todo!(),
    }
}

pub fn rust_toolchain(params : &BuildParameters) -> &'static str {
    match params.target_os {
        crate::params::TargetOs::Windows => rust_toolchain_win(params),
        crate::params::TargetOs::Linux => rust_toolchain_linux(params),
        crate::params::TargetOs::MacOS => todo!(),
    }
}

pub fn cargo_target_win(params : &BuildParameters) -> &'static str {
    match params.architecture {
        crate::params::Architecture::X64 => "x86_64-pc-windows-msvc",
        crate::params::Architecture::X86 => "i686-pc-windows-msvc",
        crate::params::Architecture::ARM64 => todo!(),
    }
}

pub fn cargo_target_linux(params : &BuildParameters) -> &'static str {
    match params.architecture {
        crate::params::Architecture::X64 => "x86_64-unknown-linux-gnu",
        crate::params::Architecture::X86 => "i686-unknown-linux-gnu",
        crate::params::Architecture::ARM64 => "arm64-unknown-linux-gnu",
    }
}

pub fn rust_toolchain_linux(params : &BuildParameters) -> &'static str {
    match params.architecture {
        crate::params::Architecture::X64 => "stable-x86_64-unknown-linux-gnu",
        crate::params::Architecture::X86 => "stable-i686-unknown-linux-gnu",
        crate::params::Architecture::ARM64 => "stable-arm64-unknown-linux-gnu",
    }
}

pub fn rust_toolchain_win(params : &BuildParameters) -> &'static str {
    match params.architecture {
        crate::params::Architecture::X64 => "stable-x86_64-pc-windows-msvc",
        crate::params::Architecture::X86 => "stable-i686-pc-windows-msvc",
        crate::params::Architecture::ARM64 => todo!(),
    }
}

pub fn build_rustflags(params : &BuildParameters) -> String {
    let mut rustflags : Vec<String> = vec![
        "-C",
        "target-feature=+crt-static",
        "-Ctarget-feature=+crt-static",
    ].iter().map(|v| v.to_string()).collect();
    if params.no_service {
        rustflags.push("-C".into());
        rustflags.push("link-args=-mwindows".into());
    }
    rustflags.join(" ")
}

pub fn cargo_command(params : &BuildParameters, dir : &PathBuf, args: &[String]) -> ExitStatus {
    let rust_toolchain = rust_toolchain(&params);
    let cargo_target = cargo_target(&params);
    let package_version = agent_version(&params);
    let rustflags = build_rustflags(&params);
    let mut cmd = std::process::Command::new("cargo");
    cmd.current_dir(&dir);
    cmd.env("CARGO_PKG_VERSION",&package_version);
    cmd.env("RUSTUP_TOOLCHAIN", &rust_toolchain);
    cmd.env("CARGO_BUILD_TARGET", &cargo_target);
    cmd.env("CARGO_TARGET_DIR", &params.target_dir);
    cmd.env("RUSTFLAGS", &rustflags);
    cmd.args(args);
    cmd.status().expect("Failed to run cargo command")
}
pub fn user_dir() -> PathBuf {
    std::env::current_dir().unwrap().join("user")
}
pub fn agent_dir() -> PathBuf {
    std::env::current_dir().unwrap().join("agent")
}
pub fn server_dir() -> PathBuf {
    std::env::current_dir().unwrap().join("server")
}

#[allow(unused)]
pub fn executable_path(exec : &str, params : &BuildParameters) -> PathBuf {
    PathBuf::from(&params.target_dir).join(cargo_target(params)).join("release").join(exec)
}
#[allow(unused)]
pub fn msi_path(exec : &str, params : &BuildParameters) -> PathBuf {
    PathBuf::from(&params.target_dir).join(cargo_target(params)).join("wix").join(exec)
}
#[allow(unused)]
pub fn executable_path_release(exec : &str, params : &BuildParameters) -> PathBuf {
    PathBuf::from(&params.target_dir).join("release").join(exec)
}

pub fn wix_file() -> PathBuf {
    std::env::current_dir().unwrap().join("agent").join("wix").join("main.wxs")
}

pub fn agent_args(params : &BuildParameters) -> Vec<String> {
    let mut features : Vec<String> = Vec::with_capacity(64);
    if params.no_service {
        features.push("no_service".into());
    } 
    let mut res : Vec<String> = ["build", "--release", "--features"].into_iter().map(|v| v.to_string()).collect();
    res.push(features.join(","));
    res
}

#[allow(unused_variables)]
pub fn user_args(params : &BuildParameters) -> Vec<String> {
    let res : Vec<String> = ["build", "--release"].into_iter().map(|v| v.to_string()).collect();
    res
}

#[allow(unused)]
pub fn installer_args(params : &BuildParameters) -> Vec<String> {
    let wxs = wix_file();
    let res : Vec<String> = ["wix", "--no-build", "--package=agent", "--nocapture"].into_iter().map(|v| v.to_string()).collect();
    res
}
#[allow(unused)]
pub fn deb_args(params : &BuildParameters) -> Vec<String> {
    let wxs = wix_file();
    let res : Vec<String> = ["wix", "--no-build", "--package=agent", "--nocapture"].into_iter().map(|v| v.to_string()).collect();
    res
}
#[allow(unused)]
pub fn rpm_args(params : &BuildParameters) -> Vec<String> {
    let wxs = wix_file();
    let res : Vec<String> = ["wix", "--no-build", "--package=agent", "--nocapture"].into_iter().map(|v| v.to_string()).collect();
    res
}
#[allow(unused_variables)]
pub fn server_args(params : &BuildParameters) -> Vec<String> {
    let res : Vec<String> = ["build", "--release"].into_iter().map(|v| v.to_string()).collect();
    res
}
#[allow(unused)]
pub fn out_directories(params : &BuildParameters) -> (PathBuf, PathBuf, PathBuf) {
    let x64 = PathBuf::from(&params.target_dir).join("x64");
    let x86 = PathBuf::from(&params.target_dir).join("x86");
    let arm64: PathBuf = PathBuf::from(&params.target_dir).join("arm64");
    (x86, x64, arm64)
}