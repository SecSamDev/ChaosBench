use serde::{Serialize, Deserialize};

use crate::err::ChaosResult;

use super::{agent::AppLog, Log, TestingReport};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum UserAction {
    BackupDB(String),
    AgentLogsAll,
    StopAgentLogs,
    AgentLogs(LogSubscription),
    AppLogsAll,
    AppLogs(LogSubscription),
    StopAppLogs,
    NoLogs,
    StartScenario(String),
    StopScenario(String),
    CreateScenario(CreateScenario),
    EnumerateScenarios,
    EnumerateTestingScenarios,
    Report,
    #[default]
    None
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateScenario {
    pub base_id : String,
    pub id : String
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogSubscription {
    pub agent : String
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum UserActionResponse {
    AgentCompletion((u32,u32)),
    Logs(Log),
    AppLogs(AppLog),
    BackupDB(ChaosResult<()>),
    StartScenario(ChaosResult<()>),
    StopScenario(ChaosResult<()>),
    CreateScenario(ChaosResult<()>),
    EnumerateScenarios(Vec<String>),
    EnumerateTestingScenarios(Vec<String>),
    Report(TestingReport),
    #[default]
    None
}
#[test]
#[ignore]
fn should_serialize_and_deserialize() {
    let action = UserAction::StartScenario("scenario 1".into());
    let txt = serde_json::to_string(&action).unwrap();
    println!("{}",txt);
    // {"TestScenario":"scenario 1"}
}