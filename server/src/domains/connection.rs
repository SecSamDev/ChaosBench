use actix::{Message as ActixMessage, Recipient};
use chaos_core::api::Log;
use serde::Serialize;

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Connect {
    pub id : String,
    pub addr: Recipient<AgentLog>,
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id : String
}

#[derive(Serialize, ActixMessage)]
#[rtype(result = "()")]
pub struct AgentLog(pub Log);