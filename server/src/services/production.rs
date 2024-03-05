use std::collections::BTreeMap;

use crate::{domains::tasks::AgentTask, repository::memory::MemoryRepository};

use super::ServerServices;
use actix_files::NamedFile;
use async_trait::async_trait;
use chaos_core::{err::{ChaosError, ChaosResult}, scenario::TestScenario};

pub struct ProductionService {
    repo : MemoryRepository
}
impl ProductionService {
    pub fn new(repo : MemoryRepository) -> Self {
        Self {repo}
    }
}
#[async_trait]
impl ServerServices for ProductionService {

    fn backup_db(&self, location : &str) -> ChaosResult<()> {
        let db = self.repo.db.lock().unwrap();
        db.save_as(location);
        Ok(())
    }

    async fn register_new_agent(&self) {
        todo!()
    }
    async fn update_agent_task(&self, task : AgentTask) {

    }

    async fn get_next_task_for_agent(&self, agent : &str) -> Option<AgentTask> {
        None
    }

    async fn upload_artifact(&self, name : &str, location : &str) {

    }

    fn agent_log(&self, agent : &str, file : String, log : String) {
        log::info!("{agent} - {file} - {log}");
    }

    fn execute_testing_scenario(&self, scenario : String) -> ChaosResult<()> {
        let mut db = self.repo.db.lock().unwrap();
        match &db.scenario {
            Some(v) => return Err(ChaosError::Other(format!("There is alredy a scenario in execution: {}", v))),
            None => {}
        };
        db.scenario = Some(scenario);
        Ok(())
    }

    fn stop_testing_scenario(&self, id : String) -> ChaosResult<()> {
        let mut db = self.repo.db.lock().unwrap();
        db.scenario = None;
        db.state = BTreeMap::new();
        Ok(())
    }
    fn create_testing_scenario(&self, id : String, scenario : &str) -> ChaosResult<()> {
        let mut db = self.repo.db.lock().unwrap();
        if db.scenarios.contains_key(&id) {
            return Err(ChaosError::Other(format!("Scenario {} alredy exists", id)))
        }
        let mut scenario_base = self.get_scenario(scenario)?;
        scenario_base.name = id.to_string();
        db.scenarios.insert(id, scenario_base);
        Ok(())
    }
    fn get_testing_scenario(&self, id : &str) -> ChaosResult<TestScenario> {
        let db = self.repo.db.lock().unwrap();
        match db.scenarios.get(id) {
            Some(v) => Ok(v.clone()),
            None => Err(ChaosError::Unknown)
        }
    }

    fn get_scenario(&self, id : &str) -> ChaosResult<TestScenario> {
        for scenario in self.repo.scenarios.iter() {
            if scenario.name == id {
                return Ok(scenario.clone())
            }
        }
        Err(chaos_core::err::ChaosError::Unknown)
    }

    fn list_testing_scenarios(&self) -> Vec<String> {
        let db = self.repo.db.lock().unwrap();
        db.scenarios.keys().map(|v| v.clone()).collect()
    }

    fn list_scenarios(&self) -> Vec<String> {
        self.repo.scenarios.iter().map(|v| v.name.clone()).collect()
    }

    async fn download_file(&self, filename : &str) -> Option<NamedFile> {
        log::info!("Downloading file: {}", filename);
        let file_path = std::env::current_dir().unwrap().join("workspace").join(filename);

        let file = actix_files::NamedFile::open_async(file_path).await.ok()?;
        Some(file)
    }
}