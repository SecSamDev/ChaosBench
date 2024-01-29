use crate::domains::tasks::AgentTask;

use super::ServerServices;
use actix_files::NamedFile;
use async_trait::async_trait;

pub struct ProductionService {

}
impl ProductionService {
    pub fn new() -> Self {
        Self {}
    }
}
#[async_trait]
impl ServerServices for ProductionService {
    async fn register_new_agent(&self) {
        todo!()
    }
    async fn update_agent_task(&self, task : AgentTask) {

    }

    async fn get_next_task_for_agent(&self, agent : &str) -> Option<AgentTask> {
        None
    }


    async fn upload_artifact(&self, name : &str, location : &str) {

    }

    async fn agent_log(&self, agent : &str, file : String, log : String) {

    }

    async fn execute_testing_scenario(&self, scenario : &str) {
        
    }
    async fn download_file(&self, filename : &str) -> Option<NamedFile> {
        log::info!("Downloading file: {}", filename);
        let file_path = std::env::current_dir().unwrap().join("workspace").join(filename);

        let file = actix_files::NamedFile::open_async(file_path).await.ok()?;
        Some(file)
    }
}