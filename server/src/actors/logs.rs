use std::collections::HashMap;

use actix::{Actor, Context, Handler, Recipient};
use chaos_core::api::{agent::AppLog, Log};

use crate::domains::connection::{AgentAppLog, AgentCompletionUpdate, AgentLog, ConnectAppLog, ConnectAppLogById, ConnectLog, ConnectLogByAgent, DisconnectAppLog, DisconnectLog};

pub struct LogServer {
    sessions: HashMap<String, (Recipient<AgentLog>, Recipient<AgentCompletionUpdate>)>,
    sessions_by_agent: HashMap<String, HashMap<String, (Recipient<AgentLog>, Recipient<AgentCompletionUpdate>)>>,
    app_sessions: HashMap<String, Recipient<AgentAppLog>>,
    app_sessions_by_agent: HashMap<String, HashMap<String, Recipient<AgentAppLog>>,>
}

impl LogServer {
    pub fn new() -> Self {
        LogServer {
            sessions: HashMap::with_capacity(64),
            sessions_by_agent: HashMap::with_capacity(64),
            app_sessions: HashMap::with_capacity(64),
            app_sessions_by_agent: HashMap::with_capacity(64),
        }
    }

    pub fn send_log(&self, log : Log) {
        self.sessions.iter().for_each(|(_id, addr)| {
            addr.0.do_send(AgentLog(log.clone()))
        });
        if let Some(agent_listener) = self.sessions_by_agent.get(&log.agent) {
            for (_, addr) in agent_listener.iter() {
                addr.0.do_send(AgentLog(log.clone()))
            }
        }
    }
    pub fn send_app_log(&self, log : AppLog) {
        self.app_sessions.iter().for_each(|(_id, addr)| {
            addr.do_send(AgentAppLog(log.clone()))
        });
        if let Some(agent_listener) = self.app_sessions_by_agent.get(&log.agent) {
            for (_, addr) in agent_listener.iter() {
                addr.do_send(AgentAppLog(log.clone()))
            }
        }
    }

    pub fn unsubscribe(&mut self, id : &str) {
        self.sessions.remove(id);
    }
    pub fn unsubscribe_app(&mut self, id : &str) {
        self.app_sessions.remove(id);
        for (_, map) in self.app_sessions_by_agent.iter_mut() {
            map.remove(id);
        }
    }
}

impl Actor for LogServer {
    type Context = Context<Self>;
}

impl Handler<ConnectLog> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: ConnectLog, _ctx: &mut Self::Context) -> Self::Result {
        let ConnectLog { id, addr, upd } = msg;
        self.sessions.insert(id, (addr, upd));
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
impl Handler<ConnectAppLogById> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: ConnectAppLogById, _ctx: &mut Self::Context) -> Self::Result {
        let ConnectAppLogById { id, addr, agent } = msg;
        self.app_sessions_by_agent.entry(agent).and_modify(|v| {
            v.insert(id.clone(), addr.clone());
        }).or_insert({
            let mut map = HashMap::new();
            map.insert(id, addr);
            map
        });
    }
}
impl Handler<ConnectLogByAgent> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: ConnectLogByAgent, _ctx: &mut Self::Context) -> Self::Result {
        let ConnectLogByAgent { id, addr, agent, upd } = msg;
        self.sessions_by_agent.entry(agent).and_modify(|v| {
            v.insert(id.clone(), (addr.clone(), upd.clone()));
        }).or_insert({
            let mut map = HashMap::new();
            map.insert(id, (addr, upd));
            map
        });
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

impl Handler<AgentCompletionUpdate> for LogServer {
    type Result = ();

    fn handle(&mut self, msg: AgentCompletionUpdate, _ctx: &mut Self::Context) -> Self::Result {
        for (_, session) in self.sessions.iter_mut() {
            session.1.do_send(msg.clone());
        }
    }
}