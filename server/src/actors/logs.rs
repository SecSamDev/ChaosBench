use std::collections::HashMap;

use actix::{Actor, Context, Handler, Recipient};
use chaos_core::api::{agent::AppLog, Log};

use crate::domains::connection::{AgentAppLog, AgentLog, ConnectAppLog, ConnectLog, DisconnectAppLog, DisconnectLog};

pub struct LogServer {
    sessions: HashMap<String, Recipient<AgentLog>>,
    app_sessions: HashMap<String, Recipient<AgentAppLog>>
}

impl LogServer {
    pub fn new() -> Self {
        LogServer {
            sessions: HashMap::with_capacity(64),
            app_sessions: HashMap::with_capacity(64),
        }
    }

    pub fn send_log(&self, log : Log) {
        self.sessions.iter().for_each(|(_id, addr)| {
            addr.do_send(AgentLog(log.clone()))
        });
    }
    pub fn send_app_log(&self, log : AppLog) {
        self.app_sessions.iter().for_each(|(_id, addr)| {
            addr.do_send(AgentAppLog(log.clone()))
        });
    }

    pub fn unsubscribe(&mut self, id : &str) {
        self.sessions.remove(id);
    }
    pub fn unsubscribe_app(&mut self, id : &str) {
        self.app_sessions.remove(id);
    }
}

impl Actor for LogServer {
    type Context = Context<Self>;
}

impl Handler<ConnectLog> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: ConnectLog, _ctx: &mut Self::Context) -> Self::Result {
        let ConnectLog { id, addr } = msg;
        self.sessions.insert(id, addr);
    }
}

impl Handler<DisconnectLog> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: DisconnectLog, _ctx: &mut Self::Context) -> Self::Result {
        self.unsubscribe(&msg.id);
    }
}


impl Handler<ConnectAppLog> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: ConnectAppLog, _ctx: &mut Self::Context) -> Self::Result {
        let ConnectAppLog { id, addr } = msg;
        self.app_sessions.insert(id, addr);
    }
}

impl Handler<DisconnectAppLog> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: DisconnectAppLog, _ctx: &mut Self::Context) -> Self::Result {
        self.unsubscribe_app(&msg.id);
    }
}

impl Handler<AgentLog> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: AgentLog, _ctx: &mut Self::Context) -> Self::Result {
        self.send_log(msg.0);
    }
}

impl Handler<AgentAppLog> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: AgentAppLog, _ctx: &mut Self::Context) -> Self::Result {
        self.send_app_log(msg.0);
    }
}