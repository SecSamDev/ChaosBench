use std::{sync::Arc, rc::Rc};

use actix::Addr;
use chaos_core::scenario::TestScenario;

use crate::{actors::logs::LogServer, services::ServerServices};

#[derive(Clone)]
pub struct ServerState {
    pub log_server: Addr<LogServer>,
    pub services : Rc<dyn ServerServices>,
    pub scenarios : Arc<Vec<TestScenario>>,    
}