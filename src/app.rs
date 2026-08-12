use crate::gpu::GpuInfo;

pub struct App {
    pub gpu_util: u32,
    pub mem_used: Option<u64>,
    pub mem_total: Option<u64>,
    pub temperature: u32,
    pub power_usage: u32,
}

impl App {
    pub fn new() -> Self {
        Self {
            gpu_util: 0,
            mem_used: None,
            mem_total: None,
            temperature: 0,
            power_usage: 0,
        }
    }

    pub fn update(&mut self, info: &GpuInfo) {
        self.gpu_util = info.utilization_gpu;
        self.mem_used = info.memory_used;
        self.mem_total = info.memory_total;
        self.temperature = info.temperature;
        self.power_usage = info.power_usage;
    }

    pub fn mem_used_gb(&self) -> Option<f64> {
        self.mem_used.map(|u| u as f64 / (1024.0 * 1024.0 * 1024.0))
    }

    pub fn mem_total_gb(&self) -> Option<f64> {
        self.mem_total
            .map(|t| t as f64 / (1024.0 * 1024.0 * 1024.0))
    }

    /// Big number part only, e.g. "6.2" or "6.2/8.0"
    pub fn mem_val_display(&self) -> String {
        match (self.mem_used_gb(), self.mem_total_gb()) {
            (Some(used), Some(total)) => format!("{:.1}/{:.1}", used, total),
            (Some(used), None) => format!("{:.1}", used),
            _ => "0".to_string(),
        }
    }

    /// Big number part only, e.g. "70.4"
    pub fn power_val_display(&self) -> String {
        if self.power_usage > 0 {
            format!("{:.1}", self.power_usage as f64 / 1000.0)
        } else {
            "0".to_string()
        }
    }
}
