use common::set_home;

mod services;
pub(crate) mod common;
pub(crate) mod stopper;
pub(crate) mod actions;
pub(crate) mod err;
pub(crate) mod state;
pub(crate) mod logging;
pub(crate) mod sys_info;
pub(crate) mod db;
pub(crate) mod api;
#[cfg(target_os="windows")]
pub(crate) mod reg;

fn main() {
    set_home();
    services::run();
}
