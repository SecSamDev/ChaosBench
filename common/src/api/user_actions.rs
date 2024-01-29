use std::default;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum UserAction {
    TestScenario(String),
    StopTest(String),
    #[default]
    None
}

#[test]
#[ignore]
fn should_serialize_and_deserialize() {
    let action = UserAction::TestScenario("scenario 1".into());
    let txt = serde_json::to_string(&action).unwrap();
    println!("{}",txt);
    // {"TestScenario":"scenario 1"}
}