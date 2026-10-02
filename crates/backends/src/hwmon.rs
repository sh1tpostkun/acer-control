use acercontrol_core::*;
use crate::HardwareBackend;
use log::info;
use std::path::{Path, PathBuf};

/// hwmon backend — discovers hwmon devices dynamically by labels and device names,
/// reading fan RPM, temperatures, and battery telemetry from sysfs.
pub struct Backend;

impl Backend {
    pub fn new() -> Self {
        Self
    }

    /// Discover the battery power_supply path dynamically (looking for type "Battery" or name starting with "BAT").
    fn find_battery_path() -> Option<PathBuf> {
        let base = Path::new("/sys/class/power_supply");
        if !base.exists() {
            return None;
        }

        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let p = entry.path();
                let type_file = p.join("type");
                if let Ok(t) = std::fs::read_to_string(&type_file) {
                    if t.trim().eq_ignore_ascii_case("Battery") {
                        return Some(p);
                    }
                }
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with("BAT") {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Find an hwmon device directory by its name attribute (e.g. "acer", "coretemp").
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
            .map_err(|e| ErrorInfo::io_error(path.display().to_string(), e))
    }

    /// Read a millidegree value (e.g. 55000) and convert to degrees Celsius.
    fn read_temp_c(path: &Path) -> Option<u8> {
        if let Ok(val) = Self::read_sysfs(path) {
            if let Ok(millideg) = val.parse::<i64>() {
                return Some((millideg / 1000).clamp(0, 255) as u8);
            }
        }
        None
    }

    /// Read fan RPM from path.
    fn read_fan_rpm(path: &Path) -> Option<u32> {
        if let Ok(val) = Self::read_sysfs(path) {
            return val.parse::<u32>().ok();
        }
        None
    }

    /// Scan all hwmon directories and find CPU temperature via labels or known devices.
    fn find_cpu_temp() -> Option<(u8, String)> {
        let hwmon_base = Path::new("/sys/class/hwmon");
        if !hwmon_base.exists() {
            return None;
        }

        // Pass 1: Look for explicit labels across all hwmon devices
        if let Ok(entries) = std::fs::read_dir(hwmon_base) {
            for entry in entries.flatten() {
                let dir = entry.path();
                if let Ok(files) = std::fs::read_dir(&dir) {
                    for f in files.flatten() {
                        let fname = f.file_name().to_string_lossy().to_string();
                        if fname.starts_with("temp") && fname.ends_with("_label") {
                            if let Ok(label) = std::fs::read_to_string(f.path()) {
                                let l = label.trim().to_lowercase();
                                // Typical labels for CPU package temperature
                                if l.contains("package id") || l.contains("tctl") || l.contains("tdie") || l.contains("cpu") {
                                    let input_name = fname.replace("_label", "_input");
                                    let input_path = dir.join(&input_name);
                                    if let Some(t) = Self::read_temp_c(&input_path) {
                                        return Some((t, input_path.to_string_lossy().to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Pass 2: Fallback to acer hwmon temp1_input or coretemp temp1_input
        if let Some(dir) = Self::find_hwmon_by_name("coretemp") {
            let p = dir.join("temp1_input");
            if let Some(t) = Self::read_temp_c(&p) {
                return Some((t, p.to_string_lossy().to_string()));
            }
        }

        if let Some(dir) = Self::find_hwmon_by_name("acer") {
            let p = dir.join("temp1_input");
            if let Some(t) = Self::read_temp_c(&p) {
                return Some((t, p.to_string_lossy().to_string()));
            }
        }

        None
    }

    /// Scan hwmon devices for fan RPM (inspecting labels first, then fallback to acer indices).
    fn find_fans() -> (Option<u32>, Option<u32>, Option<String>) {
        let mut cpu_rpm = None;
        let mut gpu_rpm = None;
        let mut source_desc = None;

        let hwmon_base = Path::new("/sys/class/hwmon");
        if !hwmon_base.exists() {
            return (None, None, None);
        }

        // Pass 1: Try labels
        if let Ok(entries) = std::fs::read_dir(hwmon_base) {
            for entry in entries.flatten() {
                let dir = entry.path();
                if let Ok(files) = std::fs::read_dir(&dir) {
                    for f in files.flatten() {
                        let fname = f.file_name().to_string_lossy().to_string();
                        if fname.starts_with("fan") && fname.ends_with("_label") {
                            if let Ok(label) = std::fs::read_to_string(f.path()) {
                                let l = label.trim().to_lowercase();
                                let input_name = fname.replace("_label", "_input");
                                let input_path = dir.join(&input_name);
                                if l.contains("cpu") && cpu_rpm.is_none() {
                                    cpu_rpm = Self::read_fan_rpm(&input_path);
                                    source_desc = Some(input_path.to_string_lossy().to_string());
                                } else if (l.contains("gpu") || l.contains("video")) && gpu_rpm.is_none() {
                                    gpu_rpm = Self::read_fan_rpm(&input_path);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Pass 2: Fallback to acer hwmon (fan1 = cpu, fan2 = gpu)
        if cpu_rpm.is_none() || gpu_rpm.is_none() {
            if let Some(dir) = Self::find_hwmon_by_name("acer") {
                let f1 = dir.join("fan1_input");
                let f2 = dir.join("fan2_input");
                if cpu_rpm.is_none() && f1.exists() {
                    cpu_rpm = Self::read_fan_rpm(&f1);
                    source_desc = Some(f1.to_string_lossy().to_string());
                }
                if gpu_rpm.is_none() && f2.exists() {
                    gpu_rpm = Self::read_fan_rpm(&f2);
                }
            }
        }

        (cpu_rpm, gpu_rpm, source_desc)
    }
}

#[async_trait::async_trait]
impl HardwareBackend for Backend {
    fn name(&self) -> &'static str {
        "hwmon"
    }

    async fn probe(&self) -> ProbeResult {
        let mut details = vec![];

        let (cpu_rpm, _, fan_source) = Self::find_fans();
        if cpu_rpm.is_some() || fan_source.is_some() {
            details.push(CapabilityDetail::new(
                Capability::FanTelemetry,
                fan_source,
            ));
        }

        let available = !details.is_empty() || Self::find_hwmon_by_name("acer").is_some() || Self::find_hwmon_by_name("coretemp").is_some();

        if available {
            info!("[hwmon] probed successfully, capabilities: {:?}", details);
        } else {
            info!("[hwmon] no relevant hwmon devices found");
        }

        ProbeResult::with_details(available, details)
    }

    async fn capabilities(&self) -> Vec<Capability> {
        let (cpu_rpm, _, fan_source) = Self::find_fans();
        if cpu_rpm.is_some() || fan_source.is_some() {
            vec![Capability::FanTelemetry]
        } else {
            vec![]
        }
    }

    async fn telemetry(&self) -> Result<Telemetry> {
        let mut tel = Telemetry::default();

        // 1. Fans (with label matching + fallback)
        let (cpu_rpm, gpu_rpm, _) = Self::find_fans();
        tel.fans.cpu_rpm = cpu_rpm;
        tel.fans.gpu_rpm = gpu_rpm;

        // 2. CPU Temperature (via label inspection or acer/coretemp fallback)
        if let Some((temp, _)) = Self::find_cpu_temp() {
            tel.temps.cpu_c = Some(temp);
            tel.temps.cpu_temp_c = Some(temp);
        }

        // 3. Fallback for GPU Temp from acer hwmon if not already read
        if let Some(dir) = Self::find_hwmon_by_name("acer") {
            let p = dir.join("temp2_input");
            if let Some(t) = Self::read_temp_c(&p) {
                tel.temps.gpu_c = Some(t);
                tel.temps.gpu_temp_c = Some(t);
            }
        }

        // 4. Battery telemetry via dynamic discovery (BAT*, type == Battery)
        if let Some(bat_dir) = Self::find_battery_path() {
            let cap_file = bat_dir.join("capacity");
            if let Ok(cap) = std::fs::read_to_string(&cap_file) {
                tel.battery.percentage = cap.trim().parse::<u8>().ok();
            }

            let status_file = bat_dir.join("status");
            if let Ok(status) = std::fs::read_to_string(&status_file) {
                let s = status.trim();
                tel.battery.is_charging = Some(s == "Charging");
            }
        }

        tel.sync_all();
        Ok(tel)
    }

    async fn get_thermal_profile(&self) -> Result<ThermalProfile> {
        Err(ErrorInfo::not_supported("hwmon does not provide thermal profiles"))
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
