use std::time::Duration;

use actix::{
    Actor, Handler
};
use chaos_core::{action::{execute::ExecutionParameters, TestActionType}, err::{ChaosError, ChaosResult}, parameters::TestParameters, tasks::{AgentTask, AgentTaskResult}};

use crate::{domains::server::ServerTask, state::ServerState, utils::now_milliseconds};

pub struct ServerActuator {
    pub(crate) state : ServerState
}

impl Actor for ServerActuator {
    type Context = actix::Context<Self>;
}


impl ServerActuator {

}

impl Handler<ServerTask> for ServerActuator {
    type Result = ();

    fn handle(&mut self, msg: ServerTask, _ctx: &mut Self::Context) -> Self::Result {
        let ServerTask { task } = msg;
        let start = now_milliseconds();
        let result = match task.action {
            TestActionType::ExecuteServer => execute_server_action(&task),
            _ => return
        };
        self.state.services.set_task_as_executed(AgentTaskResult {
            id : task.id,
            action : task.action,
            agent : task.agent,
            end : now_milliseconds(),
            limit : task.limit,
            start,
            result,
            retries : task.retries,
            scene_id : task.scene_id,
            parameters : TestParameters::default()
        });
    }
}

pub fn execute_server_action(task : &AgentTask) -> ChaosResult<()> {
    let parameters : ExecutionParameters = (&task.parameters).try_into()?;
    let mut command = std::process::Command::new(&parameters.executable);
    for param in parameters.parameters {
        command.arg(param);
    }
    let mut child = command.spawn().map_err(|e| ChaosError::Other(format!("Cannot execute command {}: {}", parameters.executable, e)))?;
    let max_duration_millis = parameters.timeout.as_millis() as i64;
    let end = now_milliseconds() + max_duration_millis;
    
    loop {
        let now = now_milliseconds();
        if now > end {
            let _ = child.kill();
            return Err(ChaosError::Other(format!("Execution error: Timeout reached")));
        }
        let ex_res = match child.try_wait() {
            Ok(v) => v,
            Err(e) => return Err(ChaosError::Other(format!("Cannot execute command {}: {}", task.id, e)))
        };
        let res = match ex_res {
            Some(v) => v,
            None => {
                // TODO: improve...
                std::thread::sleep(Duration::from_millis(10));
                continue
            }
        };
        if res.success() {
            return Ok(())
        }else {
            return Err(ChaosError::Other(format!("Execution error: exit_status={}", res.code().unwrap_or_default())))
        }
    }
}