use std::collections::BTreeMap;

use chaos_core::{action::{names::TASK_RETRIES, TestActionType}, parameters::{TestParameters, REMOTE_SERVER}, scenario::{ScenePreparationActions, TestScenario, TestScene}, tasks::AgentTask};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CalculatedScenario {
    /// Name of the scenario and identificator
    pub name : String,
    pub remote_server : Option<String>,
    pub scenes : BTreeMap<u32, String>,
    pub tasks : Vec<AgentTask>,
    pub scenario : TestScenario
}

impl From<&TestScenario> for CalculatedScenario {
    fn from(test: &TestScenario) -> Self {
        let remote_server : Option<String> = match test.parameters.global.get(REMOTE_SERVER) {
            Some(v) =>  Some(v.try_into().unwrap_or_default()),
            None => None
        };
        let mut tasks = Vec::with_capacity(test.scenes.len() * 32);
        let mut scenes = BTreeMap::new();
        let mut i = 0;
        for scene in &test.scenes {
            scenes.insert(i, scene.name.clone());
            scene_to_tasks(scene, i, test, &mut tasks);
            i += 1;
        }

        let default_retry = test.parameters.global.get(TASK_RETRIES).map(|v| v.try_into().unwrap_or(1u32)).unwrap_or(1u32);
        for action in &test.scene_preparation.cleanup.actions {
            let retries = if action_is_wait(action, test) {
                u32::MAX
            }else {
                default_retry
            };
            tasks.push(AgentTask {
                scene_id : test.scenes.len() as u32 - 1,
                action : action.clone(),
                agent : String::new(),
                id : tasks.len() as u32,
                preparation : true,
                limit : test.scene_preparation.phase_timeout.as_millis() as i64,
                parameters : TestParameters::new(),
                retries
            })
        }
        Self {
            scenes,
            scenario : test.clone(),
            name : test.name.to_string(),
            remote_server,
            tasks
        }
    }
}

fn scene_to_tasks(scene : &TestScene, scene_i : u32, scenario : &TestScenario, tasks : &mut Vec<AgentTask>) {
    let mut i = 0;
    scene_preparation(&scenario.scene_preparation.before, scene_i, scene, scenario, tasks);
    for phase in &scene.phases {
        scene_preparation(&scenario.scene_preparation.before_phase, scene_i, scene, scenario, tasks);
        if i == scene.phases.len() as u32 - 1 {
            scene_preparation(&scenario.scene_preparation.before_last, scene_i, scene, scenario, tasks);
        }
        phase_to_tasks(phase, scene_i, scene, scenario, tasks);
        if i == 0 {
            scene_preparation(&scenario.scene_preparation.after_first, scene_i, scene, scenario, tasks);
        }
        scene_preparation(&scenario.scene_preparation.after_phase, scene_i, scene, scenario, tasks);
        i += 1;
    }
    scene_preparation(&scenario.scene_preparation.after, scene_i, scene, scenario, tasks);
}
fn phase_to_tasks(action : &TestActionType, scene_id : u32, scene : &TestScene, scenario : &TestScenario, tasks : &mut Vec<AgentTask>) {
    let retries = if action_is_wait(action, scenario) {
        u32::MAX
    }else {
        scenario.parameters.global.get(TASK_RETRIES).map(|v| v.try_into().unwrap_or(1u32)).unwrap_or(1u32)
    };
    tasks.push(AgentTask {
        scene_id,
        action : action.clone(),
        agent : String::new(),
        id : tasks.len() as u32,
        preparation : false,
        limit : scene.phase_timeout.as_millis() as i64,
        parameters : TestParameters::new(),
        retries
    });
}

fn scene_preparation(preps : &ScenePreparationActions, scene_id : u32, scene : &TestScene, scenario : &TestScenario, tasks : &mut Vec<AgentTask>) {
    let default_retry = scenario.parameters.global.get(TASK_RETRIES).map(|v| v.try_into().unwrap_or(1u32)).unwrap_or(1u32);
    for action in &preps.actions {
        let retries = if action_is_wait(action, scenario) {
            u32::MAX
        }else {
            default_retry
        };
        tasks.push(AgentTask {
            scene_id,
            action : action.clone(),
            agent : String::new(),
            id : tasks.len() as u32,
            preparation : true,
            limit : scene.phase_timeout.as_millis() as i64,
            parameters : TestParameters::new(),
            retries
        })
    }
}

fn action_is_wait(action : &TestActionType, scenario : &TestScenario) -> bool {
   if action == &TestActionType::Wait {
        return true
    }
    if let TestActionType::Custom(c) = action {
        for act in &scenario.actions {
            if &act.name == c {
                if act.action == TestActionType::Wait {
                    return true
                }
            }
        }
    }
    false
}