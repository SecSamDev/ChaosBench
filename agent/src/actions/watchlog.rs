use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::Read,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use chaos_core::{
    action::{watchlog::WatchLogParameters, LogActionType}, api::agent::AppLog, err::ChaosResult,
    parameters::TestParameters,
};

use crate::{state::AgentState, sys_info::get_system_uuid};

thread_local! {
    pub static FILE_HANDLES: RefCell<BTreeMap<String, Arc<AtomicBool>>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn watchlog_action(action : &LogActionType, parameters: &TestParameters, state: &mut AgentState,) -> ChaosResult<()> {
    match action {
        LogActionType::Watch => start_listening_to_file_changes(parameters, state),
        LogActionType::StopWatch => stop_listening_to_file_changes(parameters),
    }
}

pub fn start_listening_to_file_changes(
    parameters: &TestParameters,
    state: &mut AgentState,
) -> ChaosResult<()> {
    let parameters: WatchLogParameters = parameters.try_into()?;
    start_file_watcher(parameters, state);
    Ok(())
}

pub fn stop_listening_to_file_changes(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: WatchLogParameters = parameters.try_into()?;
    stop_file_watcher(parameters);
    Ok(())
}

fn start_file_watcher(parameters: WatchLogParameters, state: &mut AgentState) {
    let file_name = parameters.file.clone();
    let stopper = Arc::new(AtomicBool::new(true));
    let stpr = stopper.clone();
    let channel = state.app_log_sender();
    let agent = get_system_uuid().unwrap();
    std::thread::spawn(move || {
        let mut file = std::fs::File::open(&parameters.file).unwrap();
        let mut buffer = vec![0u8; 1024];
        let mut str_to_send = Vec::with_capacity(1024);
        loop {
            if !stopper.load(Ordering::Relaxed) {
                break;
            }
            let ln = match file.read(&mut buffer) {
                Ok(v) => v,
                Err(_) => return,
            };
            if ln == 0 {
                std::thread::sleep(parameters.step);
                continue;
            }
            let mut p_buffer = &buffer[0..ln];
            loop {
                let pos = match p_buffer.iter().position(|&v| v == b'\n') {
                    Some(pos) => {
                        for &v in p_buffer[0..pos + 1].iter() {
                            str_to_send.push(v);
                        }
                        let _ = channel.try_send(AppLog {
                            file: parameters.file.clone(),
                            msg: String::from_utf8_lossy(&str_to_send).into(),
                            agent : agent.clone()
                        });
                        str_to_send.clear();
                        pos
                    }
                    None => {
                        for &v in p_buffer.iter() {
                            str_to_send.push(v);
                        }
                        break;
                    }
                };
                p_buffer = &p_buffer[pos + 1..];
            }
        }
    });
    FILE_HANDLES.with_borrow_mut(|v| {
        v.insert(file_name, stpr);
    });
}

fn stop_file_watcher(parameters: WatchLogParameters) {
    FILE_HANDLES.with_borrow_mut(|v| {
        if let Some(v) = v.get(&parameters.file) { v.store(false, Ordering::Relaxed) }
        v.remove(&parameters.file);
    });
}
