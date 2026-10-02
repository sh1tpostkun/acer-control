use acercontrol_core::*;
use crate::HardwareBackend;
use std::fs;

pub struct Backend {
    last_cpu_idle: std::sync::Mutex<f32>,
    last_cpu_total: std::sync::Mutex<f32>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            last_cpu_idle: std::sync::Mutex::new(0.0),
            last_cpu_total: std::sync::Mutex::new(0.0),
        }
    }
}

#[async_trait::async_trait]
impl HardwareBackend for Backend {
    fn name(&self) -> &'static str {
        "os-stats"
    }

    async fn probe(&self) -> ProbeResult {
        ProbeResult::simple(true, vec![])
    }

    async fn capabilities(&self) -> Vec<Capability> {
        vec![]
    }

    async fn telemetry(&self) -> Result<Telemetry> {
        let mut tel = Telemetry::default();

        // 1. RAM Usage
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut total = 0.0;
            let mut available = 0.0;
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Ok(kb) = parts[1].parse::<f32>() {
                        total = kb / 1024.0 / 1024.0;
                    }
                }
                if line.starts_with("MemAvailable:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Ok(kb) = parts[1].parse::<f32>() {
                        available = kb / 1024.0 / 1024.0;
                    }
                }
            }
            if total > 0.0 {
                tel.system.ram_total_gb = Some(total);
                tel.system.ram_used_gb = Some(total - available);
            }
        }

        // 2. CPU Usage
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            if let Some(line) = stat.lines().find(|l| l.starts_with("cpu ")) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let mut total = 0.0;
                    for p in &parts[1..] {
                        if let Ok(v) = p.parse::<f32>() {
                            total += v;
                        }
                    }
                    if let Ok(idle) = parts[4].parse::<f32>() {
                        let mut last_idle = self.last_cpu_idle.lock().unwrap();
                        let mut last_total = self.last_cpu_total.lock().unwrap();
                        
                        let diff_idle = idle - *last_idle;
                        let diff_total = total - *last_total;
                        
                        if diff_total > 0.0 {
                            let usage = (1.0 - diff_idle / diff_total) * 100.0;
                            tel.system.cpu_usage = Some(usage.clamp(0.0, 100.0));
                        }
                        
                        *last_idle = idle;
                        *last_total = total;
                    }
                }
            }
        }
        
        // 3. Disk Usage (Root)
        let mut vfs = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        unsafe {
            if libc::statvfs(b"/\0".as_ptr() as *const libc::c_char, vfs.as_mut_ptr()) == 0 {
                let vfs = vfs.assume_init();
                let total_bytes = vfs.f_blocks * vfs.f_frsize;
                let free_bytes = vfs.f_bfree * vfs.f_frsize;
                let used_bytes = total_bytes - free_bytes;
                
                tel.system.disk_total_gb = Some(total_bytes as f32 / 1024.0 / 1024.0 / 1024.0);
                tel.system.disk_used_gb = Some(used_bytes as f32 / 1024.0 / 1024.0 / 1024.0);
            }
        }

        Ok(tel)
    }

    async fn get_thermal_profile(&self) -> Result<ThermalProfile> {
        Err(ErrorInfo::not_supported("Not supported"))
    }
    async fn set_performance_mode(&self, _mode: ThermalProfile) -> Result<()> {
        Err(ErrorInfo::not_supported("Not supported"))
    }
    async fn set_fan_mode(&self, _mode: FanMode) -> Result<()> {
        Err(ErrorInfo::not_supported("Not supported"))
    }
    async fn set_fan_speed(&self, _cpu: u8, _gpu: u8) -> Result<()> {
        Err(ErrorInfo::not_supported("Not supported"))
    }
    async fn set_battery_limit(&self, _limit: u8) -> Result<()> {
        Err(ErrorInfo::not_supported("Not supported"))
    }
    async fn set_keyboard_timeout(&self, _timeout_s: u32) -> Result<()> {
        Err(ErrorInfo::not_supported("Not supported"))
    }
}
