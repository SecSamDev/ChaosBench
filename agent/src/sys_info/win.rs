use chaos_core::err::{ChaosError, ChaosResult};
use uuid::Uuid;
use windows::{
    core::PWSTR,
    Win32::{Foundation::FILETIME, System::{
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX}, Services::{QueryServiceStatusEx, SC_STATUS_PROCESS_INFO, SERVICE_STATUS_PROCESS}, SystemInformation::{GetSystemFirmwareTable, GetSystemInfo, GlobalMemoryStatusEx, MEMORYSTATUSEX, RSMB, SYSTEM_INFO}, Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_INFORMATION}, WindowsProgramming::{GetComputerNameW, MAX_COMPUTERNAME_LENGTH}
    }},
};

use crate::actions::service::open_service;

pub fn get_hostname() -> ChaosResult<String> {
    let mut buffer = [0u16; MAX_COMPUTERNAME_LENGTH as usize + 1];
    let mut size = buffer.len() as u32;
    if let Err(e) = unsafe { GetComputerNameW(PWSTR(buffer.as_mut_ptr()), &mut size) } {
        return Err(ChaosError::Other(e.to_string()));
    }
    if size == 0 {
        return Err(ChaosError::Unknown);
    }
    Ok(String::from_utf16_lossy(&buffer[0..size as usize]))
}

pub fn get_system_uuid() -> ChaosResult<String> {
    //https://gist.github.com/vadimpiven/618b720324e9f54c01075fcb8675f2c4
    let size = unsafe { GetSystemFirmwareTable(RSMB, 0, None) };
    let mut buffer = vec![0; size as usize];
    let writed = unsafe { GetSystemFirmwareTable(RSMB, 0, Some(&mut buffer)) };
    let buffer = &buffer[0..writed as usize];
    let length = u32::from_le_bytes(buffer[4..8].try_into().unwrap());
    let table_data = &buffer[8..8 + length as usize];
    let mut pos = 0;
    while pos < table_data.len() {
        let header_type = buffer[pos];
        let struct_size = buffer[pos + 1];
        if header_type == 0x1 {
            //SystemInformation
            let uuid = &buffer[pos + 8..pos + 24];
            let id = Uuid::from_bytes_le(uuid.try_into().unwrap());
            return Ok(id.to_string());
        }
        let termination_location = match (&buffer[pos + 4 + struct_size as usize..])
            .windows(2)
            .position(|subslice| subslice == [0, 0])
        {
            Some(v) => v,
            None => break,
        };
        pos += struct_size as usize + 6 + termination_location;
    }
    Err(chaos_core::err::ChaosError::Other(format!(
        "Cannot find ProductId"
    )))
}



#[test]
fn system_uuid_must_be_obtained() {
    let systemuuid = get_system_uuid().unwrap();
    assert!(!systemuuid.is_empty());
}
