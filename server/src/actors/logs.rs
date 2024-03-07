use std::collections::HashMap;

use actix::{Actor, Context, Handler, Recipient};
use chaos_core::api::Log;

use crate::domains::connection::{Connect, Disconnect, AgentLog};

pub struct LogServer {
    sessions: HashMap<String, Recipient<AgentLog>>
}

impl LogServer {
    pub fn new() -> Self {
        LogServer {
            sessions: HashMap::with_capacity(64),
        }
    }

    pub fn send_log(&self, log : Log) {
        self.sessions.iter().for_each(|(_id, addr)| {
            addr.do_send(AgentLog(log.clone()))
        });
    }

    pub fn unsubscribe(&mut self, id : &str) {
        self.sessions.remove(id);
    }
}

impl Actor for LogServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        let Connect { id, addr } = msg;
        self.sessions.insert(id, addr);
    }
}

impl Handler<Disconnect> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        self.unsubscribe(&msg.id);
    }
}

impl Handler<AgentLog> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: AgentLog, _ctx: &mut Self::Context) -> Self::Result {
        self.send_log(msg.0);
    }
}