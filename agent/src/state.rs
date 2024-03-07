use std::{path::Path, sync::mpsc::{sync_channel, Receiver, SyncSender}, time::Duration};

use chaos_core::
    common::hash_params_and_actions
;

use crate::{common::StopCommand, db::Database};
pub const SERVER_ADDRESS : &str = env!("AGENT_SERVER_ADDRESS");

/// Save the state of the agent in the database
pub struct AgentState {
    pub db: Database,
    task_tries: u32,
    logs : Receiver<String>,
    stopper : SyncSender<StopCommand>,
    log : Option<String>
}

impl AgentState {
    pub fn new(stopper : SyncSender<StopCommand>) -> Self {
        let db = Database::load();
        let (_, logs) = sync_channel(1);
        Self {
            db,
            task_tries: 0,
            logs,
            stopper,
            log : None
        }
    }
    pub fn set_log_receiver(&mut self, logs : Receiver<String>) {
        self.logs = logs;
    }
    pub fn try_recv_log(&mut self) -> Option<String> {
        let log = match self.logs.try_recv() {
            Ok(v) => v,
            Err(_) => return None
        };
        if self.log.is_none() && log.contains("\n") {
            return Some(log)
        }
        if self.log.is_none() {
            self.log = Some(log);
            return None
        }
        let slog = self.log.as_mut()?;
        slog.push_str(&log);
        if slog.contains("\n") {
            drop(slog);
            let mut tk = None;
            std::mem::swap(&mut self.log, &mut tk);
            return tk;
        }
        None
    }

    pub fn increase_task_try(&mut self) -> u32 {
        self.task_tries += 1;
        self.task_tries
    }

    pub fn signal_shutdown(&self, signal : StopCommand) {
        let _ = self.stopper.send(signal);
    }
    pub fn state_hash(&self) -> u64 {
        hash_params_and_actions(&self.db.parameters, &self.db.commands)
    }
    
}