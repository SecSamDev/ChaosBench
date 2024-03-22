use actix::{Message as ActixMessage, Recipient};
use chaos_core::api::{agent::AppLog, Log};
use serde::Serialize;

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct ConnectLog {
    pub id : String,
    pub addr: Recipient<AgentLog>,
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct ConnectAppLog {
    pub id : String,
    pub addr: Recipient<AgentAppLog>,
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct DisconnectLog {
    pub id : String
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct DisconnectAppLog {
    pub id : String
}

#[derive(Serialize, ActixMessage)]
#[rtype(result = "()")]
pub struct AgentLog(pub Log);

#[derive(Serialize, ActixMessage)]
#[rtype(result = "()")]
pub struct AgentAppLog(pub AppLog);