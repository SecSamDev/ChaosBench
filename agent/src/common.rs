use std::{path::PathBuf, time::{UNIX_EPOCH, SystemTime}};

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
    std::env::set_current_dir(get_home()).expect("Must configure current dir for agent");
}