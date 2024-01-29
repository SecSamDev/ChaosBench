pub mod production;
use actix_files::NamedFile;
use async_trait::async_trait;

use crate::domains::tasks::AgentTask;

#[async_trait]
pub trait ServerServices {
    async fn register_new_agent(&self);

    async fn update_agent_task(&self, task : AgentTask);

    async fn get_next_task_for_agent(&self, agent : &str) -> Option<AgentTask>;


    async fn upload_artifact(&self, name : &str, location : &str);

    async fn agent_log(&self, agent : &str, file : String, log : String);

    async fn execute_testing_scenario(&self, scenario : &str);

    async fn download_file(&self, filename : &str) -> Option<NamedFile>;
}