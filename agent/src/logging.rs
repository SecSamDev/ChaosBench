use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};

#[cfg(not(feature="no_service"))]
pub fn init_logging() {
    let stdout = ConsoleAppender::builder().build();

    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("agent.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("agent", Box::new(requests)))
        .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(Logger::builder()
            .appender("agent")
            .additive(false)
            .build("app::agent", LevelFilter::Info))
        .build(Root::builder().appender("agent").build(LevelFilter::Info))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();
}

#[cfg(feature="no_service")]
pub fn init_logging() {
    let stdout = ConsoleAppender::builder().build();

    let requests = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("agent.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("agent", Box::new(requests)))
        .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
        .logger(Logger::builder()
            .appender("agent")
            .additive(false)
            .build("app::agent", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();
}