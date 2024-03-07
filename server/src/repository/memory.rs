use std::{collections::{BTreeMap, BTreeSet}, sync::{Arc, Mutex}};

use chaos_core::{scenario::TestScenario, tasks::{AgentTask, AgentTaskResult}};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct MemoryRepository {
    pub db : Arc<Mutex<Database>>,
    pub scenarios : Arc<Vec<TestScenario>> 
}

impl MemoryRepository {
    pub fn new(db : &Arc<Mutex<Database>>, scenarios : &Arc<Vec<TestScenario>> ) -> Self {
        Self {
            db : db.clone(),
            scenarios : scenarios.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Database {
    pub agents : BTreeSet<String>,
    /// Actual scenario in execution
    pub scenario : Option<String>,
    pub scenarios : BTreeMap<String, TestScenario>,
    /// Resultado de la ejecución en cada equipo y de cada fase del escenario actual
    pub state : BTreeMap<String, AgentSceneState>
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AgentSceneState {
    pub last_task : Option<u32>,
    pub results : BTreeMap<u32, AgentTaskResult>
}

impl Database {
    pub fn load() -> Database {
        let content = std::fs::read_to_string("./database.db").unwrap_or_default();
        let database : Database = serde_json::from_str(&content).unwrap_or_default();
        database
    }
    pub fn save(&self) {
        let database = serde_json::to_string(&self).unwrap_or_default();
        std::fs::write("./database.db", database.as_bytes()).unwrap_or_default();
    }
    pub fn save_as(&self, name : &str) {
        let database = serde_json::to_string(&self).unwrap_or_default();
        let pth = format!("./{}.db",name.replace('.', ""));
        std::fs::write(pth, database.as_bytes()).unwrap_or_default();
    }

    pub fn set_task(&mut self, task : AgentTaskResult) {
        let mut entry = self.state.entry(task.agent.clone()).or_insert(AgentSceneState::default());
        entry.last_task = Some(task.id);
        entry.results.insert(task.id, task);
    }
}