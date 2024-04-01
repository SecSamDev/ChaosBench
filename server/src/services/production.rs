use std::collections::{BTreeMap, BTreeSet};

use crate::{repository::memory::MemoryRepository, utils::now_milliseconds};

use super::ServerServices;
use chaos_core::{
    action::TestActionType,
    api::{agent::ConnectAgent, TestingReport},
    common::hash_params_and_actions,
    err::{ChaosError, ChaosResult},
    scenario::TestScenario,
    tasks::{AgentTask, AgentTaskResult},
};

pub struct ProductionService {
    repo: MemoryRepository,
}
impl ProductionService {
    pub fn new(repo: MemoryRepository) -> Self {
        Self { repo }
    }
}
impl ServerServices for ProductionService {
    fn backup_db(&self, location: &str) -> ChaosResult<()> {
        let db = self.repo.db.lock().unwrap();
        db.save_as(location);
        Ok(())
    }

    fn register_new_agent(&self, info: ConnectAgent) {
        let mut db = self.repo.db.lock().unwrap();
        db.agents.insert(info.id.clone(), info);
    }
    fn update_agent_task(&self, task: AgentTask) {}

    fn total_tasks(&self) -> u32 {
        let db = self.repo.db.lock().unwrap();
        db.scenario.as_ref().map(|v| v.tasks.len() as u32).unwrap_or(u32::MAX).saturating_sub(1)
    }

    fn get_next_task_for_agent(&self, agent: &str) -> Option<AgentTask> {
        let db = self.repo.db.lock().unwrap();
        let scenario = db.scenario.as_ref()?;
        let next_task = match db.state.get(agent) {
            Some(v) => match v.last_task {
                Some(v) => v + 1,
                None => 0,
            },
            None => 0,
        };
        let mut task = scenario.tasks.get(next_task as usize).map(|v| v.clone())?;
        match task.action {
            // All server actions
            TestActionType::HttpRequest | TestActionType::HttpResponse => return None,
            _ => {}
        }
        task.agent = agent.to_string();
        Some(task)
    }

    fn upload_artifact(&self, name: &str, location: &str) {

    }

    fn hash_state(&self) -> u64 {
        let db = self.repo.db.lock().unwrap();
        let scenario = match &db.scenario {
            Some(v) => v,
            None => return u64::MAX,
        };
        let scenario = match db.scenarios.get(&scenario.name) {
            Some(v) => v,
            None => return u64::MAX,
        };
        hash_params_and_actions(&scenario.parameters, &scenario.actions, &scenario.variables)
    }

    fn agent_log(&self, agent: String, file: String, log: String) {
        log::info!("{agent} - {file} - {log}");
    }

    fn execute_testing_scenario(&self, scenario: String) -> ChaosResult<()> {
        let mut db = self.repo.db.lock().unwrap();
        match &db.scenario {
            Some(v) => {
                return Err(ChaosError::Other(format!(
                    "There is alredy a scenario in execution: {}",
                    v.name
                )))
            }
            None => {}
        };
        let scenario = db.scenarios.get(&scenario).ok_or(ChaosError::Unknown)?;
        db.scenario = Some(scenario.into());
        Ok(())
    }

    fn stop_testing_scenario(&self) -> ChaosResult<()> {
        let mut db = self.repo.db.lock().unwrap();
        db.scenario = None;
        db.state = BTreeMap::new();
        Ok(())
    }
    fn create_testing_scenario(&self, id: String, scenario: &str) -> ChaosResult<()> {
        let mut db = self.repo.db.lock().unwrap();
        if db.scenarios.contains_key(&id) {
            return Err(ChaosError::Other(format!("Scenario {} alredy exists", id)));
        }
        let mut scenario_base = self.get_scenario(scenario)?;
        scenario_base.name = id.to_string();
        db.scenarios.insert(id, scenario_base);
        Ok(())
    }
    fn get_testing_scenario(&self, id: &str) -> ChaosResult<TestScenario> {
        let db = self.repo.db.lock().unwrap();
        match db.scenarios.get(id) {
            Some(v) => Ok(v.clone()),
            None => Err(ChaosError::Unknown),
        }
    }

    fn get_scenario(&self, id: &str) -> ChaosResult<TestScenario> {
        for scenario in self.repo.scenarios.iter() {
            if scenario.name == id {
                return Ok(scenario.clone());
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
            None => return Err(ChaosError::Unknown),
        };
        let scenario = match db.scenarios.get(&scenario.name) {
            Some(v) => v,
            None => return Err(ChaosError::Unknown),
        };
        Ok(scenario.clone())
    }

    fn set_task_as_executed(&self, task: AgentTaskResult) {
        log::info!("Task completed: {}-{}", task.agent, task.id);
        let mut db = self.repo.db.lock().unwrap();
        db.set_task(task);
    }

    fn remote_server(&self) -> ChaosResult<String> {
        let db = self.repo.db.lock().unwrap();

        let scenario = db
            .scenario
            .as_ref()
            .ok_or(ChaosError::Other("No active scenario".into()))?;
        let rs = match &scenario.remote_server {
            Some(v) => v.clone(),
            None => return Err(ChaosError::Unknown),
        };
        Ok(rs)
    }

    fn agent_from_ip(&self, ip: &str) -> ChaosResult<ConnectAgent> {
        let db = self.repo.db.lock().unwrap();
        for (_, agent) in &db.agents {
            if agent.ip == ip {
                return Ok(agent.clone());
            }
        }
        Err(ChaosError::Unknown)
    }

    fn generate_report(&self) -> ChaosResult<TestingReport> {
        let db = self.repo.db.lock().unwrap();
        let scenario = db.scenario.as_ref().ok_or(ChaosError::Unknown)?;
        let mut ret = TestingReport {
            date: now_milliseconds(),
            id: scenario.name.clone(),
            report: String::with_capacity(4096),
        };
        ret.add_h1(&scenario.name);
        ret.add_content("");
        let mut id = 0;
        let mut last_scene: i32 = -1;
        let agents_total = db.state.len();
        let mut scene_ok = BTreeSet::new();
        for agent in db.state.keys() {
            scene_ok.insert(agent);
        }
        for task in &scenario.tasks {
            let task_type: &str = (&task.action).into();
            let task_id = task.id.to_string();
            if task.scene_id as i32 != last_scene {
                if last_scene >= 0 {
                    ret.add_content("\n</details>\n");
                    ret.add_content(&format!("**Resume {}/{}**", scene_ok.len(), agents_total));
                }
                last_scene = task.scene_id as i32;
                match scenario.scenes.get(&(last_scene as u32)) {
                    Some(v) => ret.add_h2(&v),
                    None => ret.add_h2("Unknown scene"),
                };
                ret.add_content("\n<details>\n<summary>Show test</summary>\n");
                ret.add_table_header(&["ID", "State", "Action", "Agent", "Hostname", "Error"]);
                scene_ok = BTreeSet::new();
                for agent in db.state.keys() {
                    scene_ok.insert(agent);
                }
            }
            for (agent, state) in db.state.iter() {
                let hostname = db
                    .agents
                    .get(agent.as_str())
                    .map(|v| v.hostname.clone())
                    .unwrap_or_default();
                let (state, msg) = match state.results.get(&id).map(|v| v.result.clone()) {
                    Some(v) => match v {
                        Ok(_) => ("✅", String::new()),
                        Err(e) => {
                            scene_ok.remove(agent);
                            ("❌", e.to_string())
                        }
                    },
                    None => {
                        scene_ok.remove(agent);
                        ("🕔", "Execution Pending".into())
                    }
                };
                ret.add_table_row(&[
                    task_id.as_str(),
                    state,
                    task_type,
                    agent.as_str(),
                    &hostname,
                    msg.as_str(),
                ]);
                id += 1;
            }
        }
        ret.add_content("\n</details>\n");
        ret.add_content(&format!("**Resume {}/{} {}**", scene_ok.len(), agents_total, if scene_ok.len() == agents_total {"✅"} else {"❌"}));
        Ok(ret)
    }

    fn list_agents(&self) -> Vec<String> {
        let db = self.repo.db.lock().unwrap();
        db.agents.keys().map(|v| v.to_string()).collect()
    }
}
