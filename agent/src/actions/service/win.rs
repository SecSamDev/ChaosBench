use std::time::{Instant, Duration};

use chaos_core::{action::service::ServiceCommand, parameters::TestParameters, err::{ChaosResult, ChaosError}};
use windows::{Win32::{System::Services::{OpenServiceW, OpenSCManagerW, SC_MANAGER_CONNECT, SC_MANAGER_ENUMERATE_SERVICE, SERVICE_INTERROGATE, SERVICE_CONTROL_STOP, CloseServiceHandle, ControlService, SERVICE_STATUS, StartServiceW, SERVICE_START, QueryServiceStatus, SERVICE_STOPPED, SERVICE_START_PENDING}, Security::SC_HANDLE}, core::PCWSTR};

pub fn stop_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: ServiceCommand = parameters.try_into()?;
    stop_service_internal(&parameters)
}

fn stop_service_internal(parameters : &ServiceCommand) -> ChaosResult<()> {
    control_service(&parameters.name, SERVICE_CONTROL_STOP)
}

pub fn start_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: ServiceCommand = parameters.try_into()?;
    start_service_internal(&parameters)
}
fn start_service_internal(parameters: &ServiceCommand) -> ChaosResult<()> {
    let sc_manager = get_manager_handle()?;
    let service = match open_service_with_manager(&parameters.name, sc_manager) {
        Ok(v) => v,
        Err(e) => {
            close_service_handle(sc_manager);
            return Err(e)
        }
    };
    if let Err(_err) = unsafe { StartServiceW(service, None) } {
        close_service_handle(service);
        close_service_handle(sc_manager);
        return Err(ChaosError::Unknown)
    }
    Ok(())
}

pub fn service_is_running(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: ServiceCommand = parameters.try_into()?;
    let sc_manager = get_manager_handle()?;
    let service = match open_service_with_manager(&parameters.name, sc_manager) {
        Ok(v) => v,
        Err(e) => {
            close_service_handle(sc_manager);
            return Err(e)
        }
    };
    if let Err(err) = set_service(service, SERVICE_CONTROL_STOP) {
        close_service_handle(service);
        close_service_handle(sc_manager);
        return Err(err)
    }
    let start = Instant::now();
    let max_duration = parameters.timeout;
    let mut service_status = SERVICE_STATUS::default();
    let mut res = Ok(());
    loop {
        let now = Instant::now();
        if now > start + max_duration {
            res = Err(ChaosError::Other(format!("Timeout")));
            break
        }
        if let Err(_err) = unsafe { QueryServiceStatus(service, &mut service_status) } {
            res = Err(ChaosError::Other(format!("Cannot get service status")));
            break
        }
        if service_status.dwCurrentState == SERVICE_START_PENDING {
            continue
        }
        break
    }
    close_service_handle(service);
    close_service_handle(sc_manager);
    res
}

pub fn restart_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: ServiceCommand = parameters.try_into()?;
    let sc_manager = get_manager_handle()?;
    let service = match open_service_with_manager(&parameters.name, sc_manager) {
        Ok(v) => v,
        Err(e) => {
            close_service_handle(sc_manager);
            return Err(e)
        }
    };
    if let Err(err) = set_service(service, SERVICE_CONTROL_STOP) {
        close_service_handle(service);
        close_service_handle(sc_manager);
        return Err(err)
    }
    let start = Instant::now();
    let max_duration = Duration::from_secs(10);
    let mut service_status = SERVICE_STATUS::default();
    loop {
        let now = Instant::now();
        if now > start + max_duration {
            return Err(ChaosError::Unknown)
        }
        if let Err(_err) = unsafe { QueryServiceStatus(service, &mut service_status) } {
            close_service_handle(service);
            close_service_handle(sc_manager);
            return Err(ChaosError::Unknown)
        }
        if service_status.dwCurrentState == SERVICE_STOPPED {
            break;
        }
        
    }
    if let Err(_err) = unsafe { StartServiceW(service, None) } {
        close_service_handle(service);
        close_service_handle(sc_manager);
        return Err(ChaosError::Unknown)
    }
    Ok(())
}



fn control_service(name : &str, control : u32) -> ChaosResult<()> {
    let sc_manager = get_manager_handle()?;
    let service = match open_service_with_manager(name, sc_manager) {
        Ok(v) => v,
        Err(e) => {
            close_service_handle(sc_manager);
            return Err(e)
        }
    };
    if let Err(err) = set_service(service, control) {
        close_service_handle(service);
        close_service_handle(sc_manager);
        return Err(err)
    }
    Ok(())
}

fn set_service(handle : SC_HANDLE, control : u32) -> ChaosResult<()> {
    let mut status = SERVICE_STATUS::default();
    if let Err(_err) = unsafe { ControlService(handle, control, &mut status) } {
        return Err(ChaosError::Unknown)
    }
    Ok(())
}

fn close_service_handle(handle : SC_HANDLE) {
    let _ = unsafe { CloseServiceHandle(handle) };
}

pub fn open_service(name : &str) -> ChaosResult<SC_HANDLE> {
    let sc_manager = get_manager_handle()?;
    match open_service_with_manager(name, sc_manager) {
        Ok(v) => Ok(v),
        Err(_) => {
            let _ = unsafe { CloseServiceHandle(sc_manager) };
            Err(ChaosError::Unknown)
        }
    }
}

fn open_service_with_manager(name : &str, manager : SC_HANDLE) -> ChaosResult<SC_HANDLE> {
    let service_name :Vec<u16> = name.encode_utf16().into_iter().chain(std::iter::once(0)).collect();
        let service_handle = match unsafe { OpenServiceW(manager, PCWSTR(service_name.as_ptr()), SERVICE_INTERROGATE | SERVICE_CONTROL_STOP | SERVICE_START) } {
            Ok(v) => v,
            Err(_) => return Err(ChaosError::Unknown)
        };
        Ok(service_handle)
}

fn get_manager_handle() -> ChaosResult<SC_HANDLE> {
    let handle = match unsafe { OpenSCManagerW(None, None, SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE) } {
        Ok(v) => v,
        Err(_) => return Err(ChaosError::Unknown)
    };
    Ok(handle)
}