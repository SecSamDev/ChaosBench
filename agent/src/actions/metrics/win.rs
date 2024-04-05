use chaos_core::{action::metrics::{StartMetricsForService, StopMetricsForService}, err::ChaosResult, parameters::TestParameters};
use windows::Win32::{Foundation::FILETIME, System::{ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX}, Services::{QueryServiceStatusEx, SC_STATUS_PROCESS_INFO, SERVICE_STATUS_PROCESS}, SystemInformation::{GetSystemInfo, GetSystemTimeAsFileTime, GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO}, Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_INFORMATION}}};

use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::actions::service::open_service;

thread_local! {
    pub static SERVICE_THREADS: RefCell<BTreeMap<String, (Arc<AtomicBool>, MetricCalculator)>> = RefCell::new(BTreeMap::new());
}


pub fn start_metric_for_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: StartMetricsForService = parameters.try_into()?;
    let stopper = Arc::new(AtomicBool::new(true));
    let service_name = parameters.service_name.clone();
    let stpr = stopper.clone();
    std::thread::spawn(move || {
        let mut kernel_time = unsafe { GetSystemTimeAsFileTime() }; 
        let mut user_time = kernel_time;
        let mut last_check = kernel_time;
        loop {
            if !stopper.load(Ordering::Relaxed) {
                break;
            }
            let service_pid = match get_pid_of_service(&parameters.service_name) {
                Some(v) => v,
                None => return
            };
            let (ram, cpu, lc, kt, ut) = get_cpu_and_memory_usage(service_pid, last_check, kernel_time, user_time);
            kernel_time = kt;
            user_time = ut;
            last_check = lc;
            add_metric(&parameters.service_name, (ram, cpu));
            std::thread::sleep(parameters.sampling_frequency);
        }
    });
    SERVICE_THREADS.with_borrow_mut(|v| {
        v.insert(service_name, (stpr, MetricCalculator::new()));
    });
    Ok(())
}

fn stop_metric_for_service(parameters: &TestParameters) -> ChaosResult<()> {
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

fn add_metric(service_name: &str, metrics : (u64, f32)) {
    SERVICE_THREADS.with_borrow_mut(|v| {
        v.get_mut(service_name).map(|v| {
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
    let mut sys_info = SYSTEM_INFO::default();
    unsafe { GetSystemInfo(&mut sys_info) };
    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();
    let mut syst = unsafe { GetSystemTimeAsFileTime() };
    let system_time = (syst.dwHighDateTime as u64) << 32 | syst.dwLowDateTime as u64;

    unsafe { GetProcessTimes(process, &mut creation_time, &mut exit_time,&mut kernel_time, &mut user_time).unwrap() };

    (ram_used as u64, 0.0, syst, kernel_time, user_time)
}

fn get_pid_of_service(name : &str) -> Option<u32> {
    let service = open_service(name).ok()?;
    let mut buffer = vec![0; 2048];
    let mut bytest_needed = 0;
    unsafe { QueryServiceStatusEx(service, SC_STATUS_PROCESS_INFO, Some(&mut buffer), &mut bytest_needed).ok()? };
    let (head, body, _) = unsafe { buffer.align_to::<SERVICE_STATUS_PROCESS>() };
    if !head.is_empty() {
        return None
    }
    Some(body[0].dwProcessId)
}

struct MetricCalculator {
    pub cpu_sum : f64,
    pub ram_sum : u64,
    pub samples : usize,
}

impl MetricCalculator {
    pub fn new() -> Self {
        Self {
            cpu_sum : 0.0,
            ram_sum : 0,
            samples : 0
        }
    }
    pub fn add(&mut self, ram : u64, cpu : f32) {
        let (cpu, ram, samples) = match self.ram_sum.checked_add(ram) {
            Some(v) => {
                (self.cpu_sum + cpu as f64, v, self.samples + 1)
            },
            None => {
                let avg_cpu = self.cpu_sum / (self.samples as f64);
                let avg_ram = self.ram_sum / (self.samples as u64);
                (avg_cpu + cpu as f64, avg_ram + ram, 2)
            }
        };
        self.samples = samples;
        self.cpu_sum = cpu;
        self.ram_sum = ram;
    }

    pub fn calculate(&self) -> (u64, f32) {
        let avg_cpu = self.cpu_sum / (self.samples as f64);
        let avg_ram = self.ram_sum / (self.samples as u64);
        (avg_ram, avg_cpu as f32)
    }
}