mod agent;
mod user;
mod proxy;
use actix_web::web;
pub use agent::*;
pub use user::*;
pub use proxy::proxy_config;

pub fn config(cfg: &mut web::ServiceConfig) {
    agent_config(cfg);
    user_config(cfg);
    proxy_config(cfg);
}
