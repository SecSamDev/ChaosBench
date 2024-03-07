use std::collections::BTreeMap;

use chaos_core::{action::CustomAction, parameters::TestParameters, tasks::AgentTask};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Database {
    pub current_task : Option<AgentTask>,
    pub parameters : TestParameters,
    pub commands : Vec<CustomAction>
}

impl Database {
    pub fn load() -> Database {
        let content = std::fs::read_to_string("./state.db").unwrap_or_default();
        let database : Database = serde_json::from_str(&content).unwrap_or_default();
        database
    }
    pub fn save(&self) {
        let database = serde_json::to_string(&self).unwrap_or_default();
        std::fs::write("./state.db", database.as_bytes()).unwrap_or_default();
    }
    pub fn get_current_task(&self) -> Option<&AgentTask> {
        self.current_task.as_ref()
    }
    pub fn set_current_task(&mut self, task: Option<AgentTask>) {
        self.current_task = task;
    }
    pub fn get_global_parameters(&self) -> &TestParameters {
        &self.parameters
    }
    pub fn set_commands(&mut self, commands: Vec<CustomAction>) {
        self.commands = commands;
    }
    pub fn get_commands(&self) -> &[CustomAction] {
        &self.commands
    }
    pub fn clean_current_task(&mut self) {
        self.current_task = None;
    }
}