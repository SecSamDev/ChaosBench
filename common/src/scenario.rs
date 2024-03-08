use std::time::Duration;

use serde::{Serialize, Deserialize, Deserializer};

use crate::{phase::TestPhase, parameters::TestParameters, variables::TestVariables, common::*, action::{TestActionType, CustomAction}};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TestScene {
    pub name : String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub description : String,
    pub phases : Vec<TestActionType>,
    #[serde(default = "default_timeout", deserialize_with = "deserialize_null_default")]
    pub timeout : Duration,
    #[serde(default = "default_timeout", deserialize_with = "deserialize_duration", serialize_with = "serialize_duration")]
    pub phase_timeout : Duration
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TestScenario {
    /// Name of the scenario and identificator
    pub name : String,
    /// Friendly description of the scenario
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub description : String,
    /// Variables to auto replace text when parsing a scenario file
    pub variables : TestVariables,
    /// Execution parameters
    pub parameters : TestParameters,
    /// List of scenes
    pub scenes : Vec<TestScene>,
    /// Custom actions with parameter overriding
    pub actions : Vec<CustomAction>,
    /// Actions to be performed for each scene
    pub scene_preparation : ScenePreparation,
    /// List of required files to be download before the testing begins
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub files : Vec<String>
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScenePreparation {
    #[serde(default = "default_timeout", deserialize_with = "deserialize_duration", serialize_with = "serialize_duration")]
    pub phase_timeout : Duration,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub cleanup : ScenePreparationActions,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub before : ScenePreparationActions,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub after_first : ScenePreparationActions,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub before_last : ScenePreparationActions,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub after : ScenePreparationActions,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub before_phase : ScenePreparationActions,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub after_phase : ScenePreparationActions
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ScenePreparationActions {
    pub actions : Vec<TestActionType>
}

#[cfg(test)]
mod tst {
    use super::*;

    #[test]
    pub fn should_parse_basic_scenario() {
        let file_content = std::fs::read_to_string("./src/basic_scenario.yaml").unwrap();
        let basic_scene : TestScenario = serde_yaml::from_str(&file_content).unwrap();
        assert_eq!(Duration::from_secs(10), basic_scene.scene_preparation.phase_timeout);
    }
}