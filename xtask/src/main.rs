use clap::Parser;
use params::Command;
pub(crate) mod params;
pub(crate) mod build;
pub(crate) mod testing;

fn main() {
    let args = Command::parse();
    match args {
        Command::BuildAgent(args) => {
            build::build_agent(args);
        },
        Command::BuildInstaller(args) => {
            build::build_installer(args);
        },
        Command::BuildFull(args) => {
            build::build_full(args);
        },
        Command::BuildServer(args) => {
            build::build_server(args);
        },
        Command::BuildUser(args) => {
            build::build_user(args);
        },
        Command::Testing => {
            testing::test_full().unwrap();
        },
    }
    
}