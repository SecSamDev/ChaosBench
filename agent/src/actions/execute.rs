use std::{cell::RefCell, process::Child, time::Duration};

use chaos_core::{action::{execute::ExecutionParameters, ExecutionActionType}, err::{ChaosError, ChaosResult}, parameters::TestParameters};

use crate::common::now_milliseconds;

thread_local! {
    pub static CLIENT: RefCell<Option<(u32, i64, Child)>> = const { RefCell::new(None) };
}

pub fn command_execution_action(action : &ExecutionActionType, task_id : u32, parameters : &TestParameters) -> Option<ChaosResult<()>> {
    match action {
        ExecutionActionType::Command => execute_command(task_id, parameters),
        ExecutionActionType::ServerCommand => Some(Ok(())),
        ExecutionActionType::Script => Some(Ok(())),
        ExecutionActionType::ServerScript => Some(Ok(())),
    }
}

pub fn execute_command(task_id : u32, parameters : &TestParameters) -> Option<ChaosResult<()>> {
    let parameters : ExecutionParameters = match parameters.try_into() {
        Ok(v) => v,
        Err(e) => return Some(Err(e))
    };
    if let Some(id) = get_actual_task_id() {
        if task_id == id {
            return try_wait_task(parameters.timeout);
        } else {
            stop_actual_task();
        }
    }
    let mut command = std::process::Command::new(&parameters.executable);
    for param in parameters.parameters {
        command.arg(param);
    }
    let child = match command.spawn().map_err(|e| ChaosError::Other(format!("Cannot execute command {}: {}", parameters.executable, e))) {
        Ok(v) => v,
        Err(e) => return Some(Err(e))
    };
    store_execution_task(task_id, child);
    Some(Ok(()))
}

fn store_execution_task(task_id : u32, child : Child) {
    CLIENT.with_borrow_mut(|v| {
        *v = Some((task_id, now_milliseconds(), child));
    });
}

fn get_actual_task_id() -> Option<u32> {
    CLIENT.with_borrow(|v| {
        v.as_ref().map(|v| v.0)
    })
}

fn stop_actual_task() {
    let (_, _, mut child) = match CLIENT.replace(None) {
        Some(v) => v,
        None => return
    };
    let _ = child.kill();
}

fn try_wait_task(max_duration : Duration) -> Option<ChaosResult<()>> {
    let now = now_milliseconds();
    let max_duration_millis = max_duration.as_millis() as i64;
    CLIENT.with_borrow_mut(|v| {
        let (id, start, child) = v.as_mut()?;
        if *start + max_duration_millis > now {
            let _ = child.kill();
            return Some(Err(ChaosError::Other(format!("Timeout reached executing command {}", id))))
        }
        let res = match child.try_wait().map_err(|e| ChaosError::Other(format!("Execution error: {}", e))) {
            Ok(v) => v?,
            Err(e) => return Some(Err(e))
        };
        if res.success() {
            return Some(Ok(()))
        }
        Some(Err(ChaosError::Other(format!("Execution error: exit_status={}", res.code().unwrap_or_default()))))
    })
}