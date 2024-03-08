use chaos_core::{action::TestActionType, parameters::TestParameters, scenario::{ScenePreparationActions, TestScenario, TestScene}, tasks::AgentTask, variables::TestVariables};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CalculatedScenario {
    /// Name of the scenario and identificator
    pub name : String,
    pub tasks : Vec<AgentTask>
}

impl From<&TestScenario> for CalculatedScenario {
    fn from(test: &TestScenario) -> Self {
        let mut tasks = Vec::with_capacity(test.scenes.len() * 32);
        for scene in &test.scenes {
            scene_to_tasks(scene, test, &mut tasks);
        }
        for action in &test.scene_preparation.cleanup.actions {
            tasks.push(AgentTask {
                action : action.clone(),
                agent : String::new(),
                id : tasks.len() as u32,
                limit : test.scene_preparation.phase_timeout.as_millis() as i64,
                parameters : TestParameters::new()
            })
        }
        Self {
            name : test.name.to_string(),
            tasks
        }
    }
}

fn scene_to_tasks(scene : &TestScene, scenario : &TestScenario, tasks : &mut Vec<AgentTask>) {
    let mut i = 0;
    scene_preparation(&scenario.scene_preparation.before, scene, scenario, tasks);
    for phase in &scene.phases {
        scene_preparation(&scenario.scene_preparation.before_phase, scene, scenario, tasks);   
        phase_to_tasks(phase, scene, scenario, tasks);
        if i == 0 {
            scene_preparation(&scenario.scene_preparation.after_first, scene, scenario, tasks);
        }else if i == scene.phases.len() - 1 {
            scene_preparation(&scenario.scene_preparation.before_last, scene, scenario, tasks);
        }
        scene_preparation(&scenario.scene_preparation.after_phase, scene, scenario, tasks);
        i += 1;
    }
    scene_preparation(&scenario.scene_preparation.after, scene, scenario, tasks);
}
fn phase_to_tasks(phase : &TestActionType, scene : &TestScene, _scenario : &TestScenario, tasks : &mut Vec<AgentTask>) {
    tasks.push(AgentTask {
        action : phase.clone(),
        agent : String::new(),
        id : tasks.len() as u32,
        limit : scene.phase_timeout.as_millis() as i64,
        parameters : TestParameters::new()
    });
}

fn scene_preparation(preps : &ScenePreparationActions, scene : &TestScene, _scenario : &TestScenario, tasks : &mut Vec<AgentTask>) {
    for action in &preps.actions {
        tasks.push(AgentTask {
            action : action.clone(),
            agent : String::new(),
            id : tasks.len() as u32,
            limit : scene.phase_timeout.as_millis() as i64,
            parameters : TestParameters::new()
        })
    }
}