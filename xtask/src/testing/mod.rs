use std::{path::PathBuf, process::ExitStatus};


pub fn test_full() -> Result<(), anyhow::Error>{
    test_agent()?;
    Ok(())
}

pub fn test_agent() -> Result<(), anyhow::Error> {
    test_components("Agent", "agent")
}

pub fn test_components(name : &str, path : &str) -> Result<(), anyhow::Error> {
    println!("---- Testing {} ----", name);
    let dir = PathBuf::from(path);
    let args = vec!["test"];
    let status = test_comand(&dir, &args, &format!("Failed to test {}", name));
    assert!(status.success());
    Ok(())
}

fn test_comand(dir : &PathBuf, args : &Vec<&str>, expect : &str) -> ExitStatus {
    std::process::Command::new("cargo").current_dir(dir).args(args).status().expect(expect)
}