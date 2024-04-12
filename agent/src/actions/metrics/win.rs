use chaos_core::{action::metrics::{MetricsArtifact, StartMetricsForProcess, StartMetricsForService, StopMetricsForProcess, StopMetricsForService, UploadMetricsForProcess, UploadMetricsForService}, err::{ChaosError, ChaosResult}, parameters::TestParameters};
use windows::Win32::{Foundation::FILETIME, System::{ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX}, Services::{QueryServiceStatusEx, SC_STATUS_PROCESS_INFO, SERVICE_STATUS_PROCESS}, SystemInformation::{GetSystemTimeAsFileTime, GlobalMemoryStatusEx, MEMORYSTATUSEX}, Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_INFORMATION}}};

use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }, time::Duration
};

use crate::{actions::service::open_service, api::{upload_data, upload_metric}, common::now_milliseconds, sys_info::get_process_by_name};

thread_local! {
    pub static SERVICE_THREADS: RefCell<BTreeMap<String, (Arc<AtomicBool>, MetricCalculator)>> = RefCell::new(BTreeMap::new());
    pub static PROCESS_THREADS: RefCell<BTreeMap<String, (Arc<AtomicBool>, MetricCalculator)>> = RefCell::new(BTreeMap::new());
}


pub fn start_metric_for_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: StartMetricsForService = parameters.try_into()?;
    let stopper = Arc::new(AtomicBool::new(true));
    let service_name = parameters.service_name.clone();
    let stpr = stopper.clone();
    SERVICE_THREADS.with_borrow_mut(|v| {
        v.insert(service_name, (stpr, MetricCalculator::new(parameters.sampling_frequency)));
    });
    std::thread::spawn(move || {
        let mut kernel_time = unsafe { GetSystemTimeAsFileTime() }; 
        let mut user_time = kernel_time;
        let mut last_check = kernel_time;
        loop {
            if !stopper.load(Ordering::Relaxed) {
                break;
            }
            let service_pid = match get_pid_of_service(&parameters.service_name) {
                Ok(v) => v,
                Err(e) => {
                    log::info!("Cannot obtain PID of service {}", e);
                    return
                }
            };
            let (ram, cpu, lc, kt, ut) = get_cpu_and_memory_usage(service_pid, last_check, kernel_time, user_time);
            kernel_time = kt;
            user_time = ut;
            last_check = lc;
            log::info!("Metrics of {}: CPU={cpu} RAM={ram}", parameters.service_name);
            add_service_metric(&parameters.service_name, (ram, cpu));
            std::thread::sleep(parameters.sampling_frequency);
        }
    });
    Ok(())
}

pub fn start_metric_for_process(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: StartMetricsForProcess = parameters.try_into()?;
    let stopper = Arc::new(AtomicBool::new(true));
    let executable_path = parameters.executable_path.clone();
    let stpr = stopper.clone();
    PROCESS_THREADS.with_borrow_mut(|v| {
        v.insert(executable_path, (stpr, MetricCalculator::new(parameters.sampling_frequency)));
    });
    let process_name = std::path::Path::new(&parameters.executable_path);
    let process_name = process_name.file_name().map(|v| v.to_string_lossy().to_string()).unwrap_or_else(|| " ".to_string());
    std::thread::spawn(move || {
        let mut last_check = unsafe { GetSystemTimeAsFileTime() }; 
        let mut user_time = FILETIME::default();
        let mut kernel_time = FILETIME::default();
        loop {
            if !stopper.load(Ordering::Relaxed) {
                break;
            }
            let service_pid = match get_process_by_name(&process_name) {
                Some(v) => v,
                None => return
            }; 
            let (ram, cpu, lc, kt, ut) = get_cpu_and_memory_usage(service_pid, last_check, kernel_time, user_time);
            kernel_time = kt;
            user_time = ut;
            last_check = lc;
            log::info!("Metrics of {}: CPU={cpu} RAM={ram}", parameters.executable_path);
            add_process_metric(&parameters.executable_path, (ram, cpu));
            std::thread::sleep(parameters.sampling_frequency);
        }
    });
    Ok(())
}

pub fn stop_metric_for_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: StopMetricsForService = parameters.try_into()?;
    let metric = SERVICE_THREADS.with_borrow_mut(|v| {
        v.get(&parameters.service_name).map(|v| v.0.store(false, Ordering::Relaxed));
        v.remove(&parameters.service_name).unwrap()
    });
    let avg_metrics = metric.1.calculate();
    if avg_metrics.0 > parameters.max_average_ram {
        return Err(chaos_core::err::ChaosError::Other(format!("Average RAM usage larger than expected: {} vs {}", avg_metrics.0, parameters.max_average_ram)));
    }
    if avg_metrics.1 > parameters.max_average_cpu {
        return Err(chaos_core::err::ChaosError::Other(format!("Average CPU usage larger than expected: {} vs {}", avg_metrics.1, parameters.max_average_cpu)));
    }
    Ok(())
}

pub fn stop_metric_for_process(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: StopMetricsForProcess = parameters.try_into()?;
    let metric = PROCESS_THREADS.with_borrow_mut(|v| {
        v.get(&parameters.executable_path).map(|v| v.0.store(false, Ordering::Relaxed));
        v.remove(&parameters.executable_path).unwrap()
    });
    let avg_metrics = metric.1.calculate();
    if avg_metrics.0 > parameters.max_average_ram {
        return Err(chaos_core::err::ChaosError::Other(format!("Average RAM usage larger than expected: {} vs {}", avg_metrics.0, parameters.max_average_ram)));
    }
    if avg_metrics.1 > parameters.max_average_cpu {
        return Err(chaos_core::err::ChaosError::Other(format!("Average CPU usage larger than expected: {} vs {}", avg_metrics.1, parameters.max_average_cpu)));
    }
    Ok(())
}

pub fn upload_metric_for_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: UploadMetricsForService = parameters.try_into()?;
    let metric : Option<MetricsArtifact> = SERVICE_THREADS.with_borrow_mut(|v| {
        let metrics = v.get(&parameters.service_name)?;
        Some(metrics.1.full_metrics())
    });
    let mut metric = match metric  {
        Some(v) => v,
        None => return Err(ChaosError::Other("Cannot find metric registry".into())),
    };
    metric.end_time = now_milliseconds();
    upload_metric(&format!("service-{}", parameters.service_name), &metric)?;
    Ok(())
}

pub fn upload_metric_for_process(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: UploadMetricsForProcess = parameters.try_into()?;
    let metric : Option<MetricsArtifact> = PROCESS_THREADS.with_borrow_mut(|v| {
        let metrics = v.get(&parameters.executable_path)?;
        Some(metrics.1.full_metrics())
    });
    let mut metric = match metric  {
        Some(v) => v,
        None => return Err(ChaosError::Other("Cannot find metric registry".into())),
    };
    metric.end_time = now_milliseconds();
    let exepath = std::path::Path::new(&parameters.executable_path).file_name().unwrap_or_default().to_string_lossy();
    upload_metric(&format!("process-{}", exepath), &metric)?;
    Ok(())
}

fn add_service_metric(service_name: &str, metrics : (u64, f32)) {
    SERVICE_THREADS.with_borrow_mut(|v| {
        v.get_mut(service_name).map(|v| {
            v.1.add(metrics.0, metrics.1);
        })
    });
}
fn add_process_metric(executable_path: &str, metrics : (u64, f32)) {
    PROCESS_THREADS.with_borrow_mut(|v| {
        v.get_mut(executable_path).map(|v| {
            v.1.add(metrics.0, metrics.1);
        })
    });
}

fn get_cpu_and_memory_usage(pid : u32, last_check : FILETIME, last_kernel_time : FILETIME, last_user_time : FILETIME) -> (u64, f32, FILETIME, FILETIME, FILETIME){
    let mut mem_info : MEMORYSTATUSEX = MEMORYSTATUSEX::default();
    mem_info.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    unsafe { GlobalMemoryStatusEx(&mut mem_info).unwrap() };
    let mut process_memory : PROCESS_MEMORY_COUNTERS_EX = PROCESS_MEMORY_COUNTERS_EX::default();
    let pmp = std::ptr::addr_of_mut!(process_memory);
    let process = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, false, pid).unwrap() };
    unsafe { GetProcessMemoryInfo(process, pmp as _, std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32).unwrap() };
    let ram_used = process_memory.WorkingSetSize;
    //let mut sys_info = SYSTEM_INFO::default();
    //unsafe { GetSystemInfo(&mut sys_info) };
    // Not used because we want percentage of usage of each processor so 400% of 4 0 4.0f32 = 4 cores at 100% 
    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();
    let now = unsafe { GetSystemTimeAsFileTime() };
    unsafe { GetProcessTimes(process, &mut creation_time, &mut exit_time,&mut kernel_time, &mut user_time).unwrap() };
    let percent = (filetime_to_u64(kernel_time).wrapping_sub(filetime_to_u64(last_kernel_time))) + (filetime_to_u64(user_time).wrapping_sub(filetime_to_u64(last_user_time)));
    let diff = filetime_to_u64(now).wrapping_sub(filetime_to_u64(last_check));
    let percent : f64 = if diff == 0 { 0.0 } else {percent as f64 / (diff as f64)};
    (ram_used as u64, percent as f32, now, kernel_time, user_time)
}

fn filetime_to_u64(filetime : FILETIME) -> u64{
    (filetime.dwHighDateTime as u64) << 32 | filetime.dwLowDateTime as u64
}

fn get_pid_of_service(name : &str) -> ChaosResult<u32> {
    let service = open_service(name)?;
    let mut buffer = vec![0; 2048];
    let mut bytest_needed = 0;
    unsafe { QueryServiceStatusEx(service, SC_STATUS_PROCESS_INFO, Some(&mut buffer), &mut bytest_needed).map_err(|e| ChaosError::Other(e.to_string()))? };
    let (head, body, _) = unsafe { buffer.align_to::<SERVICE_STATUS_PROCESS>() };
    if !head.is_empty() {
        return Err(ChaosError::Other(format!("Cannot cast structure to SERVICE_STATUS_PROCESS")))
    }
    Ok(body[0].dwProcessId)
}

struct MetricCalculator {
    pub cpu_samples : VecDeque<f32>,
    pub ram_samples : VecDeque<u64>,
    pub start_time : i64,
    pub freq : Duration
}

impl MetricCalculator {
    pub fn new(freq : Duration) -> Self {
        Self {
            cpu_samples : VecDeque::with_capacity(128),
            ram_samples : VecDeque::with_capacity(128),
            freq,
            start_time : now_milliseconds()
        }
    }
    pub fn add(&mut self, ram : u64, cpu : f32) {
        self.cpu_samples.push_back(cpu);
        self.ram_samples.push_back(ram);
    }

    pub fn calculate(&self) -> (u64, f32) {
        let samples = self.cpu_samples.len();
        let mut total_cpu : f64 = 0.0;
        for cpu in self.cpu_samples.iter() {
            total_cpu += *cpu as f64;
        }
        let avg_cpu = (total_cpu / (samples.max(1) as f64)) as f32;
        let mut samples = self.ram_samples.len();
        let mut total_ram : u64 = 0;
        for ram in self.ram_samples.iter() {
            let (ram, smpls) = match total_ram.checked_add(*ram) {
                Some(v) => (v, samples + 1),
                None => {
                    let avg_ram = total_ram / (samples as u64);
                    (avg_ram + ram, 2)
                }
            };
            samples = smpls;
            total_ram += ram;
        }
        
        (total_ram / samples.max(1) as u64, avg_cpu)
    }

    pub fn full_metrics(&self) -> MetricsArtifact {
        MetricsArtifact {
            ram_samples: self.ram_samples.iter().map(|v| *v).collect(),
            cpu_samples: self.cpu_samples.iter().map(|v| *v as f64).collect(),
            start_time: self.start_time,
            end_time: now_milliseconds(),
            freq: self.freq,
        }
    }
}


#[test]
fn should_extract_process_metrics() {
    let mut last_check = unsafe { GetSystemTimeAsFileTime() };
    let mut last_kernel_time = FILETIME::default();
    let mut last_user_time = FILETIME::default();
    let _ = std::thread::spawn(|| {
        let mut total = 1;
        for _ in 0..10_000_000 {
            total += 2;
        }
        total
    });
    std::thread::sleep(std::time::Duration::from_millis(100));
    let (ram, cpu, lc, kt, ut) = get_cpu_and_memory_usage(std::process::id(), last_check, last_kernel_time, last_user_time);
    assert!(ram > 0);
    assert!(cpu > 0.0);
    println!("RAM={ram} CPU={cpu}");
    std::thread::sleep(std::time::Duration::from_millis(1_000));
    last_check = lc;
    last_kernel_time = kt;
    last_user_time = ut;
    let (ram, cpu, _lc, _kt, _ut) = get_cpu_and_memory_usage(std::process::id(), last_check, last_kernel_time, last_user_time);
    println!("RAM={ram} CPU={cpu}");
    assert!(ram > 0);
}