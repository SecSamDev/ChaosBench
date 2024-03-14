pub fn init_logging() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
}