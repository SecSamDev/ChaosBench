use std::collections::{BTreeMap, BTreeSet};

use tokio::sync::Mutex;

pub struct MemoryRepository {
    pub db : Mutex<Database>    
}

#[derive(Clone, Debug)]
struct Database {
    pub agents : BTreeSet<String>,
    pub scenario : String,
    /// Resultado de la ejecución en cada equipo y de cada fase del escenario actual
    pub state : BTreeMap<String, BTreeMap<String, Result<(), String>>>
}