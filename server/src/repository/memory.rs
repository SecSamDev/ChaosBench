use std::{collections::BTreeMap, sync::{Arc, Mutex}};

use chaos_core::{action::metrics::MetricsArtifact, api::agent::ConnectAgent, common::deserialize_null_default, scenario::TestScenario, tasks::AgentTaskResult};
use serde::{Deserialize, Serialize};

use crate::domains::scenario::CalculatedScenario;

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
    pub agents : BTreeMap<String, ConnectAgent>,
    /// Actual scenario in execution
    pub scenario : Option<CalculatedScenario>,
    pub scenarios : BTreeMap<String, TestScenario>,
    /// Resultado de la ejecución en cada equipo y de cada fase del escenario actual
    pub state : BTreeMap<String, AgentSceneState>
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AgentSceneState {
    pub last_task : Option<u32>,
    pub results : BTreeMap<u32, AgentTaskResult>,
    #[serde(deserialize_with="deserialize_null_default")]
    pub metric : BTreeMap<String , MetricsArtifact>
}

impl Database {
    pub fn load() -> Database {
        let content = std::fs::read_to_string("./database.db").unwrap_or_default();
        let database : Database = serde_json::from_str(&content).unwrap_or_default();
        log::info!("Loaded database with scenarios={} and executing={}", database.scenarios.len(), database.scenario.as_ref().map(|v| v.name.clone()).unwrap_or_default());
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
        let entry = self.state.entry(task.agent.clone()).or_insert(AgentSceneState::default());
        entry.last_task = Some(task.id);
        entry.results.insert(task.id, task);
    }
}