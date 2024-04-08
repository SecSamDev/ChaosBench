pub mod production;
use chaos_core::{action::metrics::MetricsArtifact, api::{agent::ConnectAgent, TestingReport}, err::ChaosResult, scenario::TestScenario, tasks::{AgentTask, AgentTaskResult}};

pub trait ServerServices {
    /// Gets the remote server when intercepting requests
    fn remote_server(&self) -> ChaosResult<String>;

    fn backup_db(&self, location : &str) -> ChaosResult<()>;

    /// Registers a new agent
    fn register_new_agent(&self, info : ConnectAgent);

    /// Get scenario configuration state
    fn hash_state(&self) -> u64;

    /// Updates an agent task
    fn update_agent_task(&self, task : AgentTask);

    /// Gets the next scenario task for an agent
    fn get_next_task_for_agent(&self, agent : &str) -> Option<AgentTask>;

    /// Uploads an agent artifact
    fn upload_artifact(&self, name : &str, location : &str);

    /// Uploads an agent log
    fn agent_log(&self, agent : String, file : String, log : String);

    /// Queues a scenario for testing
    fn execute_testing_scenario(&self, id : String) -> ChaosResult<()>;

    /// Remove a scenario from the queue
    fn stop_testing_scenario(&self) -> ChaosResult<()>;

    /// Creates a testing scenario based on a file
    fn create_testing_scenario(&self, id : String, scenario : &str) -> ChaosResult<()>;

    /// Gets a testing scenario: scenario created based on a file
    fn get_testing_scenario(&self, id : &str) -> ChaosResult<TestScenario>;

    /// Gets current testing scenario
    fn current_scenario(&self) -> ChaosResult<TestScenario>;

    /// Gets a scenario from a file
    fn get_scenario(&self, id : &str) -> ChaosResult<TestScenario>;

    /// List all testing scenarios
    fn list_testing_scenarios(&self) -> Vec<String>;

    /// List all file scenarios
    fn list_scenarios(&self) -> Vec<String>;
    /// List all agents
    fn list_agents(&self) -> Vec<String>;
    /// Sets a task as executed
    fn set_task_as_executed(&self, task : AgentTaskResult);

    /// Number of tasks in this scenario
    fn total_tasks(&self) -> u32;

    fn agent_from_ip(&self, ip : &str) -> ChaosResult<ConnectAgent>;

    fn generate_report(&self) -> ChaosResult<TestingReport>;

    fn set_metrics_for_agent(&self, agent : &str, metric_name : &str, metrics : MetricsArtifact) -> ChaosResult<()>;
}