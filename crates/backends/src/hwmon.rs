use acercontrol_core::*;
use crate::HardwareBackend;
use log::{info, debug};
use std::path::{Path, PathBuf};

/// hwmon backend — discovers hwmon devices dynamically by name, reads
/// fan RPM, temperatures, and battery info from sysfs.
pub struct Backend;

impl Backend {
    pub fn new() -> Self {
        Self
    }

    /// Find an hwmon device directory by its name.
    fn find_hwmon_by_name(target_name: &str) -> Option<PathBuf> {
        let hwmon_base = Path::new("/sys/class/hwmon");
        if !hwmon_base.exists() {
            return None;
        }

        if let Ok(entries) = std::fs::read_dir(hwmon_base) {
            for entry in entries.flatten() {
                let name_path = entry.path().join("name");
                if let Ok(name) = std::fs::read_to_string(&name_path) {
                    if name.trim() == target_name {
                        return Some(entry.path());
                    }
                }
            }
        }
        None
    }

    fn read_sysfs(path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| ErrorInfo::io_error(&path.to_string_lossy(), e))
    }

    /// Read a millidegree value (e.g. 55000) and convert to degrees.
    fn read_temp_c(hwmon_dir: &Path, file: &str) -> Option<u8> {
        let path = hwmon_dir.join(file);
        if let Ok(val) = Self::read_sysfs(&path) {
            if let Ok(millideg) = val.parse::<i64>() {
                return Some((millideg / 1000).clamp(0, 255) as u8);
            }
        }
        None
    }

    /// Read fan RPM.
    fn read_fan_rpm(hwmon_dir: &Path, file: &str) -> Option<u32> {
        let path = hwmon_dir.join(file);
        if let Ok(val) = Self::read_sysfs(&path) {
            return val.parse::<u32>().ok();
        }
        None
    }
}

#[async_trait::async_trait]
impl HardwareBackend for Backend {
    fn name(&self) -> &'static str {
        "hwmon"
    }

    async fn probe(&self) -> ProbeResult {
        let mut caps = vec![];

        let acer = Self::find_hwmon_by_name("acer");
        if let Some(ref dir) = acer {
            // Check for fan inputs
            if dir.join("fan1_input").exists() {
                caps.push(Capability::FanTelemetry);
            }
            info!("[hwmon] found 'acer' device at {:?}", dir);
        }

        let coretemp = Self::find_hwmon_by_name("coretemp");
        if coretemp.is_some() {
            debug!("[hwmon] found 'coretemp' device");
        }

        let available = !caps.is_empty() || acer.is_some();

        if !available {
            info!("[hwmon] no relevant hwmon devices found");
        }

        ProbeResult {
            available,
            active_capabilities: caps,
        }
    }

    async fn capabilities(&self) -> Vec<Capability> {
        let mut caps = vec![];
        if let Some(ref dir) = Self::find_hwmon_by_name("acer") {
            if dir.join("fan1_input").exists() {
                caps.push(Capability::FanTelemetry);
            }
        }
        caps
    }

    async fn telemetry(&self) -> Result<Telemetry> {
        let mut tel = Telemetry::default();

        // Read from "acer" hwmon: fans + acer temps
        if let Some(dir) = Self::find_hwmon_by_name("acer") {
            tel.fans.cpu_rpm = Self::read_fan_rpm(&dir, "fan1_input");
            tel.fans.gpu_rpm = Self::read_fan_rpm(&dir, "fan2_input");

            // temp1 = CPU, temp2 = GPU on Acer hwmon typically
            tel.temps.cpu_c = Self::read_temp_c(&dir, "temp1_input");
            tel.temps.gpu_c = Self::read_temp_c(&dir, "temp2_input");
        }

        // Battery telemetry from power_supply
        if let Ok(cap) = std::fs::read_to_string("/sys/class/power_supply/BAT1/capacity") {
            tel.battery.percentage = cap.trim().parse::<u8>().ok();
        }
        if let Ok(status) = std::fs::read_to_string("/sys/class/power_supply/BAT1/status") {
            let s = status.trim();
            tel.battery.is_charging = Some(s == "Charging");
        }

        Ok(tel)
    }

    async fn set_performance_mode(&self, _mode: ThermalProfile) -> Result<()> {
        Err(ErrorInfo::not_supported("hwmon does not control performance modes"))
    }

    async fn set_fan_mode(&self, _mode: FanMode) -> Result<()> {
        Err(ErrorInfo::not_supported("hwmon is read-only for fans"))
    }

    async fn set_fan_speed(&self, _cpu: u8, _gpu: u8) -> Result<()> {
        Err(ErrorInfo::not_supported("hwmon is read-only for fans"))
    }

    async fn set_battery_limit(&self, _limit: u8) -> Result<()> {
        Err(ErrorInfo::not_supported("hwmon does not control battery limit"))
    }

    async fn set_keyboard_timeout(&self, _timeout_s: u32) -> Result<()> {
        Err(ErrorInfo::not_supported("hwmon does not control keyboard"))
    }
}
