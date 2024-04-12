use actix::Message as ActixMessage;
use chaos_core::tasks::AgentTask;

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct ServerTask {
    pub task : AgentTask
}