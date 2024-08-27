use chaos_core::err::ChaosResult;
use nix::unistd::gethostname;

pub fn get_system_uuid() -> ChaosResult<String> {
    if let Ok(v) = std::fs::read_to_string("/sys/class/dmi/id/product_uuid") {
        return Ok(v)
    }
    if let Ok(v) = std::fs::read_to_string("/sys/class/dmi/id/board_serial") {
        return Ok(v)
    }
    if let Ok(v) = std::fs::read_to_string("/etc/machine-id") {
        return Ok(v)
    }
    if let Ok(v) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        return Ok(v)
    }
    Err(chaos_core::err::ChaosError::Other(format!(
        "Cannot find product_uuid"
    )))
}

pub fn get_hostname() -> ChaosResult<String> {
    if let Ok(v) = gethostname() {
        return Ok(v.to_string_lossy().to_string())
    }
    if let Ok(v) = std::env::var("COMPUTERNAME") {
        return Ok(v)
    }
    Err(chaos_core::err::ChaosError::Other(format!(
        "Cannot get hostname"
    )))
}