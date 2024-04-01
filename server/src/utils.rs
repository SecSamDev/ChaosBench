use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_milliseconds() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}

pub const CHAOS_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn version_numeric() -> u64 {
    let mut version_num: Vec<u64> = CHAOS_VERSION
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