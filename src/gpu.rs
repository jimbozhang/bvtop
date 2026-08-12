use anyhow::{Context, Result};
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::Nvml;

#[allow(dead_code)]
pub struct GpuInfo {
    pub utilization_gpu: u32,      // 0-100
    pub memory_used: Option<u64>,  // bytes, None if unavailable
    pub memory_total: Option<u64>, // bytes, None if unavailable
    pub temperature: u32,          // celsius
    pub power_usage: u32,          // milliwatts
    pub process_count: usize,
}

pub struct GpuContext {
    nvml: Nvml,
}

impl GpuContext {
    pub fn new() -> Result<Self> {
        let nvml = Nvml::init().context("Failed to initialize NVML")?;
        Ok(Self { nvml })
    }

    pub fn process_count(&self) -> Result<usize> {
        let device = self
            .nvml
            .device_by_index(0)
            .context("Failed to get device")?;
        let count = device
            .running_compute_processes_count()
            .context("Failed to get processes")?;
        Ok(count as usize)
    }

    pub fn query(&self) -> Result<GpuInfo> {
        let device = self
            .nvml
            .device_by_index(0)
            .context("Failed to get device")?;

        let util = device
            .utilization_rates()
            .context("Failed to get utilization")?;

        // Try memory_info(); on some devices (e.g. GB10) it may not be available
        let (memory_used, memory_total) = match device.memory_info() {
            Ok(mem) => (Some(mem.used), Some(mem.total)),
            Err(_) => {
                // Fallback: sum per-process memory usage
                let procs = device.running_compute_processes().unwrap_or_default();
                let used: u64 = procs
                    .iter()
                    .map(|p| match p.used_gpu_memory {
                        UsedGpuMemory::Used(bytes) => bytes,
                        UsedGpuMemory::Unavailable => 0,
                    })
                    .sum();
                (Some(used), None)
            }
        };

        let temp = device
            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .context("Failed to get temperature")?;

        let power = device.power_usage().unwrap_or(0);

        let proc_count = device
            .running_compute_processes_count()
            .context("Failed to get processes")?;

        Ok(GpuInfo {
            utilization_gpu: util.gpu,
            memory_used,
            memory_total,
            temperature: temp,
            power_usage: power,
            process_count: proc_count as usize,
        })
    }
}
