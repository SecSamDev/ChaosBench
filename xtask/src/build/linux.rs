use std::path::PathBuf;

use crate::params::BuildParameters;

pub fn build_full(params : BuildParameters) {
    
}

pub fn build_agent(params : BuildParameters) {
    let project_dir = super::agent_dir();
    let cargo_build = super::cargo_command(&params, &project_dir, &["build", "--release"]);
    assert!(cargo_build.success());
}