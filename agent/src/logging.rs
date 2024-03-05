use std::sync::mpsc::{Receiver, SyncSender as Sender};
use std::time::Duration;
use std::io::Write;
use chaos_core::api::agent::AgentLogReq;
use chaos_core::api::Log;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::append::Append;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::Encode;
use websocket::OwnedMessage;

use crate::sys_info::get_system_uuid;

pub fn init_remote_logging(receiver : Receiver<String>) {
    use crate::state::SERVER_ADDRESS;

    let route = format!("ws://{}/agent/connect", SERVER_ADDRESS);
    
    let agent = get_system_uuid().unwrap();

    std::thread::spawn(move || {
        loop {
            let mut msg = String::with_capacity(1024);
            let mut client = match websocket::ClientBuilder::new(&route){
                Ok(v) => v,
                Err(_) => {
                    std::thread::sleep(Duration::from_secs_f32(20.0));
                    continue
                }
            };
            let mut client = match client.connect_insecure() {
                Ok(v) => v,
                Err(_) => {
                    std::thread::sleep(Duration::from_secs_f32(20.0));
                    continue
                }
            };
            loop {
                let log = receiver.recv().unwrap();
                msg.push_str(&log);
                if !log.contains("\n") {
                    continue;
                }
                let rq = AgentLogReq {
                    agent : agent.clone(),
                    log : Log {
                        agent : agent.clone(),
                        file : String::new(),
                        msg
                    }
                };
                msg = String::with_capacity(1024);
                let bin = OwnedMessage::Binary(serde_json::to_vec(&rq).unwrap());
                if let Err(_) = client.send_message(&bin) {
                    break
                }
            }
        }
    });

}

#[cfg(not(feature="no_service"))]
pub fn init_logging() {
    let pattern = "{d(%Y-%m-%d %H:%M:%S)} | {T} | {({l}):5.5} | {m}{n}";
    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build("agent.log")
        .unwrap();
    let (sender, receiver) = std::sync::mpsc::sync_channel(4096);
    let apiappender = ApiAppender::new(
        Box::new(PatternEncoder::new(pattern)),
        sender.clone(),
    );
    init_remote_logging(receiver);
    let config = Config::builder()
        .appender(Appender::builder().build("agent", Box::new(requests)))
        .appender(Appender::builder().build("api", Box::new(apiappender)))
        .build(Root::builder().appender("agent").appender("api").build(LevelFilter::Info))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}

#[cfg(feature="no_service")]
pub fn init_logging() {
    let stdout = ConsoleAppender::builder().build();
    let pattern = "{d(%Y-%m-%d %H:%M:%S)} | {T} | {({l}):5.5} | {m}{n}";
    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build("agent.log")
        .unwrap();
    let (sender, receiver) = std::sync::mpsc::sync_channel(4096);
    let apiappender = ApiAppender::new(
        Box::new(PatternEncoder::new(pattern)),
        sender.clone(),
    );
    init_remote_logging(receiver);
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("agent", Box::new(requests)))
        .appender(Appender::builder().build("api", Box::new(apiappender)))
        .build(Root::builder().appender("agent").appender("stdout").appender("api").build(LevelFilter::Info))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();
}



#[derive(Debug)]
pub struct ApiAppender {
    writer: ApiWriter,
    encoder: Box<dyn Encode>,
}
#[derive(Debug)]
pub struct ApiWriter {
    writer: Sender<String>,
}
pub struct ApiWriterLock {
    writer: Sender<String>,
}

impl ApiAppender {
    pub fn new(encoder: Box<dyn Encode>, sender: Sender<String>) -> Self {
        Self {
            writer: ApiWriter::new(sender),
            encoder,
        }
    }
}
impl ApiWriter {
    pub fn new(sender: Sender<String>) -> Self {
        Self { writer: sender }
    }
}

impl Append for ApiAppender {
    fn flush(&self) {}

    fn append(&self, record: &log::Record) -> anyhow::Result<()> {
        let mut writer = self.writer.lock();
        self.encoder.encode(&mut writer, record)?;
        Ok(())
    }
}

impl Write for ApiWriterLock {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let msg = String::from_utf8_lossy(buf).to_string();
        // TODO: Tener en cuenta que el canal esté lleno...
        // NOTA: Las llamadas a este método son por cada parte del pattern, no una linea entera
        let ln = msg.len();
        let _ = self.writer.send(msg);
        Ok(ln)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl ApiWriter {
    fn lock(&self) -> ApiWriterLock {
        ApiWriterLock {
            writer: self.writer.clone(),
        }
    }
}

impl log4rs::encode::Write for ApiWriterLock {
    fn set_style(&mut self, _style: &log4rs::encode::Style) -> std::io::Result<()> {
        Ok(())
    }
}