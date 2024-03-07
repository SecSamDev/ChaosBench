use common::set_home;
use logging::init_logging;

mod services;
pub(crate) mod common;
pub(crate) mod stopper;
pub(crate) mod actions;
pub(crate) mod err;
pub(crate) mod state;
pub(crate) mod logging;
pub(crate) mod sys_info;
pub(crate) mod db;

fn main() {
    set_home();
    services::run();
}
