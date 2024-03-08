use std::collections::BTreeMap;

use crate::repository::memory::MemoryRepository;

use super::ServerServices;
use actix_files::NamedFile;
use chaos_core::{common::hash_params_and_actions, err::{ChaosError, ChaosResult}, scenario::TestScenario, tasks::{AgentTask, AgentTaskResult}};

pub struct ProductionService {
    repo : MemoryRepository
}
impl ProductionService {
    pub fn new(repo : MemoryRepository) -> Self {
        Self {repo}
    }
}
impl ServerServices for ProductionService {

    fn backup_db(&self, location : &str) -> ChaosResult<()> {
        let db = self.repo.db.lock().unwrap();
        db.save_as(location);
        Ok(())
    }

    fn register_new_agent(&self) {
        todo!()
    }
    fn update_agent_task(&self, task : AgentTask) {

    }

    fn get_next_task_for_agent(&self, agent : &str) -> Option<AgentTask> {
        let mut db = self.repo.db.lock().unwrap();
        db.agents.insert(agent.to_string());
        let scenario = db.scenario.as_ref()?;
        let next_task = match db.state.get(agent) {
            Some(v) => match v.last_task {
                Some(v) => v + 1,
                None => 0
            },
            None => 0
        };
        let mut task = scenario.tasks.get(next_task as usize).map(|v| v.clone())?;
        task.agent = agent.to_string();
        Some(task)
    }

    fn upload_artifact(&self, name : &str, location : &str) {

    }
    fn hash_state(&self) -> u64 {
        let db = self.repo.db.lock().unwrap();
        let scenario = match &db.scenario {
            Some(v) => v,
            None => return u64::MAX
        };
        let scenario = match db.scenarios.get(&scenario.name) {
            Some(v) => v,
            None => return u64::MAX
        };
        hash_params_and_actions(&scenario.parameters, &scenario.actions, &scenario.variables)
    }

    fn agent_log(&self, agent : String, file : String, log : String) {
        log::info!("{agent} - {file} - {log}");
    }

    fn execute_testing_scenario(&self, scenario : String) -> ChaosResult<()> {
        let mut db = self.repo.db.lock().unwrap();
        match &db.scenario {
            Some(v) => return Err(ChaosError::Other(format!("There is alredy a scenario in execution: {}", v.name))),
            None => {}
        };
        let scenario = db.scenarios.get(&scenario).ok_or(ChaosError::Unknown)?;
        db.scenario = Some(scenario.into());
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

    fn current_scenario(&self) -> ChaosResult<TestScenario> {
        let db = self.repo.db.lock().unwrap();
        let scenario = match &db.scenario {
            Some(v) => v,
            None => return Err(ChaosError::Unknown)
        };
        let scenario = match db.scenarios.get(&scenario.name) {
            Some(v) => v,
            None => return Err(ChaosError::Unknown)
        };
        Ok(scenario.clone())
    }

    fn set_task_as_executed(&self, task : AgentTaskResult) {
        log::info!("Task completed: {}-{}", task.agent, task.id);
        let mut db = self.repo.db.lock().unwrap();
        db.set_task(task);
    }
}