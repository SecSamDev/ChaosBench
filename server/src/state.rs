use std::{sync::Arc, rc::Rc};

use chaos_core::scenario::TestScenario;

use crate::services::ServerServices;

#[derive(Clone)]
pub struct ServerState {
    pub services : Rc<dyn ServerServices>,
    pub scenarios : Arc<Vec<TestScenario>>
    
}