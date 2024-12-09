use chaos_core::{action::CustomAction, parameters::ScenarioParameters, tasks::AgentTask, variables::{ScenarioVariables, TestVariables}};
use serde::{Deserialize, Serialize};

use crate::common::AgentTaskInternal;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Database {
    pub current_task : Option<AgentTaskInternal>,
    pub parameters : ScenarioParameters,
    pub commands : Vec<CustomAction>,
    pub g_variables : ScenarioVariables,
    pub variables : TestVariables
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
    pub fn get_current_task(&self) -> Option<&AgentTaskInternal> {
        self.current_task.as_ref()
    }
    pub fn update_current_task(&mut self, task : AgentTaskInternal) {
        self.current_task = Some(task);
    }
    pub fn set_current_task(&mut self, task: Option<AgentTask>) {
        self.current_task = task.map(|v| AgentTaskInternal {
            scene_id : v.scene_id,
            action : v.action,
            agent : v.agent,
            end : None,
            id : v.id,
            limit : v.limit,
            parameters : v.parameters,
            result : None,
            start : 0,
            retries : v.retries
        });
    }
    pub fn set_global_parameters(&mut self, params : ScenarioParameters) {
        self.parameters = params;
    }
    pub fn get_global_parameters(&self) -> &ScenarioParameters {
        &self.parameters
    }
    pub fn set_global_variables(&mut self, params : ScenarioVariables) {
        self.variables = (&params).into();
        self.g_variables = params;
    }
    pub fn get_global_variables(&self) -> &ScenarioVariables {
        &self.g_variables
    }
    pub fn get_variables(&self) -> &TestVariables {
        &self.variables
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