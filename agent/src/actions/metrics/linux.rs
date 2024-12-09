use std::{io::Read, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use chaos_core::{action::metrics::{StartMetricsForProcess, StartMetricsForService}, err::{ChaosError, ChaosResult}};

use crate::{actions::metrics::{add_process_metric, add_service_metric}, common::spawn_child_and_return_stdout};

pub fn spawn_metrics_for_process(parameters : StartMetricsForProcess, stopper : Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let mut buffer = String::with_capacity(2048);
        let mut cpu_times = Vec::with_capacity(32);
        let mut last_utime = 0;
        let mut last_systime = 0;
        let mut last_putime = 0;
        let mut last_psystime = 0;
        if let Ok((utime, stime)) = extract_cpu_times(&mut cpu_times, &mut buffer) {
            last_utime = utime;
            last_systime = stime;
        }
        buffer.clear();
        cpu_times.clear();
        loop {
            if !stopper.load(Ordering::Relaxed) {
                break;
            }
            let pid = match get_pid_of_process_by_name(&parameters.executable_path) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Cannot extract PID of process {}: {}", parameters.executable_path, e);
                    sleep_error();
                    continue
                }
            };
            let ram = match extract_process_ram_usage(pid, &mut buffer) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("{}", e);
                    continue
                }
            };
            let (utime, stime) = match extract_cpu_times(&mut cpu_times, &mut buffer) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("{}", e);
                    last_utime = 0;
                    last_systime = 0;
                    continue
                }
            };
            let (putime, pstime) = match extract_process_cpu_times(pid, &mut cpu_times, &mut buffer) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("{}", e);
                    last_putime = 0;
                    last_psystime = 0;
                    continue
                }
            };
            if last_putime == 0 || last_utime == 0 {
                last_putime = putime;
                last_psystime = pstime;
                last_utime = utime;
                last_systime = stime;
                continue
            }
            let user_util = 100.0 * ((utime - last_utime) as f32) / ((putime - last_putime) as f32);
            let sys_util = 100.0 * ((stime - last_systime) as f32) / ((pstime - last_psystime) as f32);
            last_utime = utime;
            last_putime = putime;
            last_systime = stime;
            last_psystime = pstime;
            let cpu = user_util + sys_util;
            log::info!("Metrics of process {pid}: CPU={cpu} RAM={ram}");
            add_process_metric(&parameters.executable_path, (ram, cpu));
            std::thread::sleep(parameters.sampling_frequency);
        }
    });
}

pub fn spawn_metrics_for_service(parameters : StartMetricsForService, stopper : Arc<AtomicBool>) {
    let parameters = parameters.clone();
    std::thread::spawn(move || {
        let mut buffer = String::with_capacity(2048);
        let mut cpu_times = Vec::with_capacity(32);
        let mut last_utime = 0;
        let mut last_systime = 0;
        let mut last_putime = 0;
        let mut last_psystime = 0;
        if let Ok((utime, stime)) = extract_cpu_times(&mut cpu_times, &mut buffer) {
            last_utime = utime;
            last_systime = stime;
        }
        buffer.clear();
        cpu_times.clear();
        loop {
            if !stopper.load(Ordering::Relaxed) {
                break;
            }
            let pid = match get_pid_of_service(&parameters.service_name) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Cannot extract PID of service {}: {}", parameters.service_name, e);
                    sleep_error();
                    continue
                }
            };
            let ram = match extract_process_ram_usage(pid, &mut buffer) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("{}", e);
                    continue
                }
            };
            let (utime, stime) = match extract_cpu_times(&mut cpu_times, &mut buffer) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("{}", e);
                    last_utime = 0;
                    last_systime = 0;
                    continue
                }
            };
            let (putime, pstime) = match extract_process_cpu_times(pid, &mut cpu_times, &mut buffer) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("{}", e);
                    last_putime = 0;
                    last_psystime = 0;
                    continue
                }
            };
            if last_putime == 0 || last_utime == 0 {
                last_putime = putime;
                last_psystime = pstime;
                last_utime = utime;
                last_systime = stime;
                continue
            }
            let user_util = 100.0 * ((utime - last_utime) as f32) / ((putime - last_putime) as f32);
            let sys_util = 100.0 * ((stime - last_systime) as f32) / ((pstime - last_psystime) as f32);
            last_utime = utime;
            last_putime = putime;
            last_systime = stime;
            last_psystime = pstime;
            let cpu = user_util + sys_util;
            log::info!("Metrics of process {pid}: CPU={cpu} RAM={ram}");
            add_service_metric(&parameters.service_name, (ram, cpu));
            std::thread::sleep(parameters.sampling_frequency);
        }
    });
}

fn sleep_error() {
    std::thread::sleep(std::time::Duration::from_millis(500));
}

fn extract_process_cpu_times(process : u32, process_times : &mut Vec<u32>, buffer : &mut String) -> ChaosResult<(u32, u32)> {
    let res : std::io::Result<usize> = (|| {
        let mut f = std::fs::File::open(format!("/proc/{}/stat", process))?;
        f.read_to_string(buffer)
    })();
    let readed = match res {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Cannot extract cpu times: {}", e)))
    };
    let lines = &buffer[0..readed];
    
    if let Some((utime, stime)) = parse_process_cpu_line(lines, process_times) {
        return Ok((utime, stime))
    }
    Err(ChaosError::Other("Cannot extract process cpu times".into()))
}

fn extract_process_ram_usage(pid : u32, buffer : &mut String)-> ChaosResult<u64> {
    let res : std::io::Result<usize> = (|| {
        let mut f = std::fs::File::open(format!("/proc/{}/status", pid))?;
        f.read_to_string(buffer)
    })();
    let readed = match res {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Cannot extract process RAM usage of {}: {}", pid, e)))
    };
    let lines = &buffer[0..readed];
    if let Some(v) = parse_process_ram_usage(lines) {
        return Ok(v)
    }
    Err(ChaosError::Other(format!("Cannot extract process RAM usage of {}: not found", pid)))
}

fn extract_cpu_times(cpu_times : &mut Vec<u32>, buffer : &mut String) -> ChaosResult<(u32, u32)> {
    let res : std::io::Result<usize> = (|| {
        let mut f = std::fs::File::open("/proc/stat")?;
        f.read_to_string(buffer)
    })();
    let readed = match res {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Cannot extract cpu times: {}", e)))
    };
    let lines = &buffer[0..readed];
    parse_cpu_lines(lines, cpu_times)?;
    Ok((cpu_times[0], cpu_times[1]))
}

fn parse_cpu_lines(lines : &str, cpu_times : &mut Vec<u32>) -> ChaosResult<()> {
    for line in lines.lines() {
        if parse_splited_cpu_line(line, cpu_times).is_some() {
            return Ok(())
        }
    }
    Err(ChaosError::Other("Cannot find aggregate CPU".into()))
}

/// Parses the result of cat /proc/stat: normal proc in usermode, nice proc in usermode, proc in kernelmode, idle, iowait, irq, sofirq
fn parse_splited_cpu_line(line : &str, cpu_times : &mut Vec<u32>) -> Option<()> {
    let mut splited = line.split(" ");

    let cpu = splited.next()?;
    if cpu != "cpu" {
        return None
    }
    cpu_times.clear();
    loop {
        let splt = splited.next()?.trim();
        if splt.is_empty() {
            continue
        }
        let n = splt.parse::<u32>().ok()?;
        cpu_times.push(n);
        if cpu_times.len() == 7 {
            return Some(())
        }
    }
}

fn parse_process_cpu_line(line : &str, process_times : &mut Vec<u32>) -> Option<(u32,u32)> {
    let mut splited = line.split(" ");
    splited.next()?;
    process_times.clear();
    loop {
        let splt = splited.next()?.trim();
        if splt.is_empty() {
            continue
        }
        let n = match splt.parse::<u32>().ok() {
            Some(v) =>v,
            None => continue 
        };
        process_times.push(n);
        if process_times.len() == 35 {
            return Some((process_times[10], process_times[11]))
        }
    }
}
fn parse_process_ram_usage(line : &str) -> Option<u64> {
    let mut splited = line.split(" ");
    splited.next()?;
    loop {
        let splt = splited.next()?.trim();
        if splt.is_empty() {
            continue
        }
        if !splt.starts_with("VmSize:") {
            continue
        }
        let line = splt[7..].trim();
        let mut splited = line.split(" ");
        let size = splited.next()?.parse::<u64>().ok()?;
        let modifier : u64 = match splited.next() {
            Some(v) => match v {
                "kB" => 1000,
                "mB" => 1_000_000,
                "gB" => 1_000_000_000,
                _ => 1
            },
            None => 1
        };
        return Some(size * modifier);
    }
}

fn get_pid_of_service(name : &str) -> ChaosResult<u32> {
    let mut cmd = std::process::Command::new("systemctl");
    cmd.arg("show").arg("--property").arg("MainPID").arg(name);
    let stdout = spawn_child_and_return_stdout(cmd, std::time::Duration::from_secs_f32(10.0), "Cannot extract PID of service using systemctl")?;
    parse_pid_of_service(&stdout)
}

fn get_pid_of_process_by_name(name : &str) -> ChaosResult<u32> {
    let process_list = get_running_process_ids()?;
    for pid in process_list {
        let pth = match get_exe_of_pid(pid) {
            Ok(v) => v,
            Err(_) => continue
        };
        if pth.trim() == name {
            return Ok(pid)
        }
    }
    Err(ChaosError::Other(format!("Cannot get PID of process with name {}: not found", name)))
}

fn get_exe_of_pid(pid : u32) -> ChaosResult<String> {
    let ex = match std::fs::read_link(format!("/proc/{}/exe", pid)) {
        Ok(v) => v,
        Err(e) => return Err(ChaosError::Other(format!("Cannot extract EXE of process {}: {}", pid, e)))
    };
    Ok(ex.to_string_lossy().to_string())
}

fn get_running_process_ids() -> ChaosResult<Vec<u32>> {
    Ok(std::fs::read_dir("/proc").map_err(|e| ChaosError::Other(format!("Cannot list running processes: {}", e)))?        
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|pid_str| pid_str.parse::<u32>().ok())
        .collect())
}

fn parse_pid_of_service(line : &str) -> ChaosResult<u32> {
    let splited = line.split("MainPID=");
    for n in splited {
        if n.is_empty() {
            continue
        }
        match n.parse::<u32>() {
            Ok(v) => return Ok(v),
            Err(_) => continue,
        }
    }
    Err(ChaosError::Other("Cannot extract PID of service using systemctl: MainPID not found".into()))
}

#[test]
fn should_parse_cpu_times() {
    let stat = r#"cpu  1310544 115 1802365 13973055 14614 0 321334 0 0 0
cpu0 56269 0 48800 1315575 1786 0 229272 0 0 0
cpu1 196914 0 319669 903460 3821 0 41206 0 0 0"#;
    let mut cpu_times = Vec::with_capacity(32);
    parse_cpu_lines(stat, &mut cpu_times).unwrap();
    assert_eq!(1310544, cpu_times[0], "invalid time: normal process executing in user mode");
    assert_eq!(115, cpu_times[1], "invalid time: nice process executing in user mode");
    assert_eq!(1802365, cpu_times[2], "invalid time: process executing in kernel mode");
    assert_eq!(13973055, cpu_times[3], "invalid time: idle (twiddling thumbs)");
    assert_eq!(14614, cpu_times[4], "invalid time: iowait (waiting for I/O to complete)");
    assert_eq!(0, cpu_times[5], "invalid time: irq (servicing interrupts)");
    assert_eq!(321334, cpu_times[6], "invalid time: sofirq (servicing  softirqs)");
}

#[test]
fn should_parse_process_times() {
    let stat = r#"13026 (node) S 737 640 640 34816 640 4194304 64133 0 336 0 560 81 0 0 20 0 7 0 143460 1044410368 8465 18446744073709551615 4194304 87735666 140731954156832 0 0 0 0 16781312 17922 0 0 0 17 10 0 0 0 0 0 89832920 89969728 91230208 140731954159149 140731954159401 140731954159401 140731954163625 0"#;
    let mut cpu_times = Vec::with_capacity(32);
    parse_process_cpu_line(stat, &mut cpu_times).unwrap();
}

#[test]
fn should_parse_service_pid() {
    let stdout = r#"MainPID=125688"#;
    assert_eq!(125688, parse_pid_of_service(stdout).unwrap());
}