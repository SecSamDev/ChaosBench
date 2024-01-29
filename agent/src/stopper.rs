use std::{sync::mpsc::{Receiver, RecvTimeoutError}, time::{Duration, UNIX_EPOCH, SystemTime}};

use chaos_core::action::TestActionType;

use crate::{common::{StopCommand, now_milliseconds}, state::AgentState, actions::execute_action};


pub fn wait_for_service_signal(signal : Receiver<StopCommand>) {
    let mut state = AgentState::new("./state.db");
    on_start_service(&mut state);
    loop {
        let shutdown = match signal.recv_timeout(Duration::from_secs_f32(1.0)) {
            Ok(v) => v,
            Err(e) => {
                if let RecvTimeoutError::Disconnected = e {
                    break;
                }
                do_actual_work(&mut state);
                continue
            }
        };
        if let StopCommand::Shutdown = shutdown {
            signal_agent_shutdown(&mut state);
        }
        log::info!("Stopping ChaosAgent");
    }
    
}
fn on_start_service(state : &mut AgentState) {
    if let Some(task) = state.get_current_task() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        if TestActionType::RestartHost == task.action {
            let mut task = task.to_owned();
            task.end = Some(now);
            task.result = Some(Ok(()));
            state.clean_current_task();
            if let Err(err) = state.notify_completed_task(&task) {
                log::error!("Cannot notify of completed task: {:?}", err);
            }
        }
    }
}

fn do_actual_work(state : &mut AgentState) {
    // Do things while waiting for the service stop signal
    let actual_task = match state.get_current_task() {
        None => {
            let next_task = state.get_next_task();
            if let Some(next_task) = &next_task {
                log::info!("Received new task: {:?}", next_task);
            }
            state.set_current_task(next_task);
            state.get_current_task()
        },
        Some(v) => Some(v),
    };
    let mut actual_task = match actual_task {
        Some(v) => v.clone(),
        None => return,
    };

    if actual_task.start == 0 {
        //Execute task
        let now = now_milliseconds();
        actual_task.start = now;
        match execute_action(actual_task.action.clone(), state, &mut actual_task) {
            Ok(_) => {},
            Err(err) => {
                log::info!("Error executing task {}: {:?}", actual_task.id, err);
                let tries = state.increase_task_try();
                if tries > 5 {
                    actual_task.end = Some(now_milliseconds());
                    actual_task.result = Some(Err(format!("Error executing task {}: {:?}", actual_task.id, err)));
                    state.clean_current_task();
                    if let Err(err) = state.notify_completed_task(&actual_task) {
                        log::error!("Cannot notify of completed task: {:?}", err);
                    }
                }
                return
            }
        };
    }
    if actual_task.end.is_some() && actual_task.result.is_some() {
        state.clean_current_task();
        if let Err(err) = state.notify_completed_task(&actual_task) {
            log::error!("Cannot notify of completed task: {:?}", err);
        }
    }
}

fn signal_agent_shutdown(_state : &mut AgentState) {
    // The agent was signaled to shutdown -> Create file to manage that
}