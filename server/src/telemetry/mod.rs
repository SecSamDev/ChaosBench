//use opentelemetry::global;
//use opentelemetry_sdk::{trace::TracerProvider, propagation::TraceContextPropagator, logs::LoggerProvider};
//use opentelemetry_appender_tracing::layer;
//use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}
/* 
pub fn init_telemetry() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let log_exporter = opentelemetry_stdout::LogExporter::default();
    // Configure your tracer provider with your exporter(s)
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let log_provider = LoggerProvider::builder()
        .with_simple_exporter(log_exporter).build();
    let layer = layer::OpenTelemetryTracingBridge::new(&log_provider);
    global::set_tracer_provider(provider);
    tracing_subscriber::registry().with(layer).init();
    global::set_logger_provider(log_provider);
    
}
*/