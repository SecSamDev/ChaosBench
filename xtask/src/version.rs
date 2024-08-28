pub const REQUIRED_RUST_VERSION : u64 = 100_076_000;

pub fn set_support_for_win7() {
    if rust_needs_to_be_updated(){
        let first_version = get_rust_version().unwrap_or_default();
        match update_rust_version() {
            Some(_) => {
                let second_version = get_rust_version().unwrap_or_default();
                if first_version != second_version {
                    println!("Rust was updated: new {}, old {}", first_version, second_version);
                }
            },
            None => {
                println!("Rust cannot be updated");
            }
        }
    }
}

pub fn rust_needs_to_be_updated() -> bool {
    match get_rust_version() {
        Some(v) => {
            if v != REQUIRED_RUST_VERSION {
                return true
            }
            false
        },
        None => true
    }
}

pub fn update_rust_version() -> Option<bool> {
    let mut cmd = std::process::Command::new("rustup");
    cmd.arg("default").arg("1.76");
    cmd.status().unwrap();
    let mut cmd = std::process::Command::new("rustup");
    cmd.arg("target").arg("add").arg("i686-pc-windows-msvc");
    cmd.status().unwrap();
    let mut cmd = std::process::Command::new("rustup");
    cmd.arg("update");
    Some(cmd.status().ok()?.success())
}

pub fn get_rust_version() -> Option<u64> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("--version");
    let res = cmd.output().ok()?;
    let version = String::from_utf8_lossy(&res.stdout);
    if version.starts_with("cargo ") {
        let mut splited = version.split(' ');
        let _ = splited.next();
        let version = splited.next()?;
        return Some(version_to_numeric(version))
    }
    None
}

pub fn version_to_numeric(version : &str) -> u64 {
    let mut version_num: Vec<u64> = version
        .split('.')
        .map(|v| v.parse::<u64>().unwrap())
        .collect();
    let mut version: u64 = 0;
    let mut multiply = 1;
    if version_num.len() > 3 {
        for _ in 0..(version_num.len() - 3) {
            version_num.remove(0);
        }
    }
    version_num.reverse();
    for v_n in version_num {
        version += v_n * multiply;
        multiply *= 1000;
    }
    version
}