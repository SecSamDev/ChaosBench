mod agent;
mod user;
use actix_web::web;
pub use agent::*;
pub use user::*;


pub fn config(cfg : &mut web::ServiceConfig) {
    agent_config(cfg);
    user_config(cfg);   
}