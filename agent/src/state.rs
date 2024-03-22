use std::{path::Path, sync::mpsc::{sync_channel, Receiver, SyncSender}, time::Duration};

use chaos_core::{api::agent::AppLog, 
    common::hash_params_and_actions}
;

use crate::{common::StopCommand, db::Database};
pub const SERVER_ADDRESS : &str = env!("SERVER_ADDRESS");
pub const SERVER_PORT : &str = env!("SERVER_PORT");

/// Save the state of the agent in the database
pub struct AgentState {
    pub db: Database,
    task_tries: u32,
    logs : Receiver<String>,
    app_logs : Receiver<AppLog>,
    app_logs_s : SyncSender<AppLog>,
    stopper : SyncSender<StopCommand>,
    log : Option<String>
}

impl AgentState {
    pub fn new(stopper : SyncSender<StopCommand>) -> Self {
        let db = Database::load();
        let (_, logs) = sync_channel(1);
        let (app_logs_s, app_logs) = sync_channel(1024);
        Self {
            db,
            task_tries: 0,
            logs,
            stopper,
            log : None,
            app_logs,
            app_logs_s
        }
    }
    pub fn set_log_receiver(&mut self, logs : Receiver<String>) {
        self.logs = logs;
    }
    pub fn try_recv_log(&mut self) -> Option<String> {
        loop {
            let log = self.logs.try_recv().ok()?;
            if self.log.is_none() && log.contains("\n") {
                return Some(log)
            }
            if self.log.is_none() {
                self.log = Some(log);
                continue
            }
            let slog = self.log.as_mut()?;
            slog.push_str(&log);
            if slog.contains("\n") {
                let mut tk = None;
                std::mem::swap(&mut self.log, &mut tk);
                return tk;
            }
        }
    }
    pub fn try_recv_app_log(&mut self) -> Option<AppLog> {
        self.app_logs.try_recv().ok()
    }
    pub fn app_log_sender(&mut self) -> SyncSender<AppLog> {
        self.app_logs_s.clone()
    }

    pub fn increase_task_try(&mut self) -> u32 {
        self.task_tries += 1;
        self.task_tries
    }

    pub fn signal_shutdown(&self, signal : StopCommand) {
        let _ = self.stopper.send(signal);
    }
    pub fn state_hash(&self) -> u64 {
        hash_params_and_actions(&self.db.parameters, &self.db.commands, &self.db.g_variables)
    }
    
}