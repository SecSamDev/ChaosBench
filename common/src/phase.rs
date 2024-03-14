use std::time::Duration;

use serde::{Serialize, Deserialize};

use crate::{action::TestActionType, parameters::TestParameters, err::ChaosResult};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum PhaseActor {
    Server,
    #[default]
    Agent
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TestPhase {
    /// Max duration of this phase to be set as invalid
    pub timeout : Duration,
    /// The one who performs the action
    pub actor : PhaseActor,
    /// Action to be performed
    pub action : TestActionType,
    /// Parameters used by the action
    pub parameters : TestParameters
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestPhaseResult {
    pub data : TestParameters,
    pub result : ChaosResult<()>,
    pub duration : Duration
}