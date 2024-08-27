#[cfg(target_os="windows")]
pub mod win;
use std::{cell::RefCell, collections::{BTreeMap, VecDeque}, sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};

use chaos_core::{action::metrics::{MetricsArtifact, StartMetricsForProcess, StartMetricsForService, StopMetricsForProcess, StopMetricsForService, UploadMetricsForProcess, UploadMetricsForService}, err::{ChaosError, ChaosResult}, parameters::TestParameters};
#[cfg(target_os="windows")]
pub use win::*;

#[cfg(target_os="linux")]
pub mod linux;
#[cfg(target_os="linux")]
pub use linux::*;

use crate::common::now_milliseconds;

thread_local! {
    pub static SERVICE_THREADS: RefCell<BTreeMap<String, (Arc<AtomicBool>, MetricCalculator)>> = RefCell::new(BTreeMap::new());
    pub static PROCESS_THREADS: RefCell<BTreeMap<String, (Arc<AtomicBool>, MetricCalculator)>> = RefCell::new(BTreeMap::new());
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


pub fn start_metric_for_service(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: StartMetricsForService = parameters.try_into()?;
    let stopper = Arc::new(AtomicBool::new(true));
    let service_name = parameters.service_name.clone();
    let stpr = stopper.clone();
    SERVICE_THREADS.with_borrow_mut(|v| {
        v.insert(service_name, (stpr, MetricCalculator::new(parameters.sampling_frequency)));
    });

    spawn_metrics_for_service(parameters, stopper);
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

pub fn start_metric_for_process(parameters: &TestParameters) -> ChaosResult<()> {
    let parameters: StartMetricsForProcess = parameters.try_into()?;
    let stopper = Arc::new(AtomicBool::new(true));
    let executable_path = parameters.executable_path.clone();
    let stpr = stopper.clone();
    PROCESS_THREADS.with_borrow_mut(|v| {
        v.insert(executable_path, (stpr, MetricCalculator::new(parameters.sampling_frequency)));
    });
    spawn_metrics_for_process(parameters, stopper);
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
    crate::api::upload_metric(&format!("service-{}", parameters.service_name), &metric)?;
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
    crate::api::upload_metric(&format!("process-{}", exepath), &metric)?;
    Ok(())
}

pub fn add_service_metric(service_name: &str, metrics : (u64, f32)) {
    SERVICE_THREADS.with_borrow_mut(|v| {
        v.get_mut(service_name).map(|v| {
            v.1.add(metrics.0, metrics.1);
        })
    });
}
pub fn add_process_metric(service_name: &str, metrics : (u64, f32)) {
    PROCESS_THREADS.with_borrow_mut(|v| {
        v.get_mut(service_name).map(|v| {
            v.1.add(metrics.0, metrics.1);
        })
    });
}