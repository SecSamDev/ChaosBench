use std::sync::mpsc::{Receiver, SyncSender as Sender};
use std::time::Duration;
use std::io::Write;
use chaos_core::api::agent::AgentRequest;
use log::LevelFilter;

use log4rs::append::file::FileAppender;
use log4rs::append::Append;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::Encode;

#[cfg(not(feature="no_service"))]
pub fn init_logging() -> Option<Receiver<String>> {
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
    let config = Config::builder()
        .appender(Appender::builder().build("agent", Box::new(requests)))
        .appender(Appender::builder().build("api", Box::new(apiappender)))
        .build(Root::builder().appender("agent").appender("api").build(LevelFilter::Info))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
    Some(receiver)
}

#[cfg(feature="no_service")]
pub fn init_logging() -> Option<Receiver<String>> {
    use log4rs::append::console::ConsoleAppender;

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
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("agent", Box::new(requests)))
        .appender(Appender::builder().build("api", Box::new(apiappender)))
        .build(Root::builder().appender("agent").appender("stdout").appender("api").build(LevelFilter::Info))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
    Some(receiver)
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
        let _ = self.writer.try_send(msg);
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