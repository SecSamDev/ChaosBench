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
/// Creates a new file in the workspace
pub fn create_file_path_in_workspace(filename : &str) -> PathBuf {
    std::env::current_dir().unwrap().join(filename)
}
/// Creates a new file in the temp workspace
pub fn create_file_path_in_temp(filename : &str) -> PathBuf {
    std::env::current_dir().unwrap().join("temp").join(filename)
}
/// Creates a new file in the APP temp workspace
pub fn create_file_path_in_app_temp(filename : &str) -> PathBuf {
    std::env::current_dir().unwrap().join("app_temp").join(filename)
}
/// Creates a new file in the User temp workspace
pub fn create_file_path_in_user_temp(filename : &str) -> PathBuf {
    std::env::current_dir().unwrap().join("user_temp").join(filename)
}