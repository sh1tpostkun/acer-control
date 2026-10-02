use acercontrol_core::*;
use crate::HardwareBackend;
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const PLATFORM_PROFILE: &str = "/sys/firmware/acpi/platform_profile";
const PLATFORM_PROFILE_CHOICES: &str = "/sys/firmware/acpi/platform_profile_choices";

pub struct Backend {
    nitro_dir: Mutex<Option<PathBuf>>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            nitro_dir: Mutex::new(None),
        }
    }

    /// Discovers the nitro_sense sysfs directory across known locations or by platform scanning.
    fn discover_nitro_dir() -> Option<PathBuf> {
        let candidates = [
            "/sys/devices/platform/acer-wmi/nitro_sense",
            "/sys/module/linuwu_sense/drivers/platform:acer-wmi/acer-wmi/nitro_sense",
            "/sys/bus/platform/drivers/acer-wmi/acer-wmi/nitro_sense",
        ];

        for &path_str in &candidates {
            let p = Path::new(path_str);
            if p.exists() && p.is_dir() {
                debug!("[linuwu-sense] discovered nitro_sense at {}", path_str);
                return Some(p.to_path_buf());
            }
        }

        // Secondary discovery: scan /sys/devices/platform/ for any nitro_sense
        if let Ok(entries) = std::fs::read_dir("/sys/devices/platform") {
            for entry in entries.flatten() {
                let candidate = entry.path().join("nitro_sense");
                if candidate.exists() && candidate.is_dir() {
                    debug!("[linuwu-sense] discovered nitro_sense via platform scan: {:?}", candidate);
                    return Some(candidate);
                }
            }
        }

        None
    }

    fn get_dir(&self) -> Option<PathBuf> {
        self.nitro_dir.lock().unwrap().clone().or_else(Self::discover_nitro_dir)
    }

    fn nitro_path(&self, attr: &str) -> Option<PathBuf> {
        self.get_dir().map(|d| d.join(attr))
    }

    fn read_sysfs(path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| ErrorInfo::io_error(path.display().to_string(), e))
    }

    fn write_sysfs(path: &Path, value: &str) -> Result<()> {
        std::fs::write(path, value)
            .map_err(|e| ErrorInfo::io_error(path.display().to_string(), e))
    }
}

#[async_trait::async_trait]
impl HardwareBackend for Backend {
    fn name(&self) -> &'static str {
        "linuwu-sense"
    }

    async fn probe(&self) -> ProbeResult {
        let nitro_path = Self::discover_nitro_dir();
        if let Some(ref dir) = nitro_path {
            *self.nitro_dir.lock().unwrap() = Some(dir.clone());
        }

        let module_loaded = Path::new("/sys/module/linuwu_sense").exists() || nitro_path.is_some();

        if !module_loaded {
            info!("[linuwu-sense] module/interface not found");
            return ProbeResult::simple(false, vec![]);
        }

        let nitro_dir = match nitro_path {
            Some(d) => d,
            None => {
                warn!("[linuwu-sense] module loaded but nitro_sense sysfs interface not found");
                return ProbeResult::simple(false, vec![]);
            }
        };

        let mut details = vec![];

        // Check fan_speed
        let fan_path = nitro_dir.join("fan_speed");
        if fan_path.exists() {
            details.push(CapabilityDetail::new(
                Capability::FanControl,
                Some(fan_path.to_string_lossy().to_string()),
            ));
        }

        // Check ACPI platform_profile
        if Path::new(PLATFORM_PROFILE).exists() {
            details.push(CapabilityDetail::new(
                Capability::PerformanceMode,
                Some(PLATFORM_PROFILE.to_string()),
            ));
        }

        // Check battery_limiter
        let bat_path = nitro_dir.join("battery_limiter");
        if bat_path.exists() {
            details.push(CapabilityDetail::new(
                Capability::BatteryChargeLimit,
                Some(bat_path.to_string_lossy().to_string()),
            ));
        }

        // Check backlight_timeout
        let bl_path = nitro_dir.join("backlight_timeout");
        if bl_path.exists() {
            details.push(CapabilityDetail::new(
                Capability::KeyboardBacklight,
                Some(bl_path.to_string_lossy().to_string()),
            ));
        }

        info!("[linuwu-sense] probed successfully at {:?}, capabilities: {:?}", nitro_dir, details);

        ProbeResult::with_details(true, details)
    }

    async fn capabilities(&self) -> Vec<Capability> {
        let mut caps = vec![];
        if let Some(dir) = self.get_dir() {
            if dir.join("fan_speed").exists() {
                caps.push(Capability::FanControl);
            }
            if Path::new(PLATFORM_PROFILE).exists() {
                caps.push(Capability::PerformanceMode);
            }
            if dir.join("battery_limiter").exists() {
                caps.push(Capability::BatteryChargeLimit);
            }
            if dir.join("backlight_timeout").exists() {
                caps.push(Capability::KeyboardBacklight);
            }
        }
        caps
    }

    async fn telemetry(&self) -> Result<Telemetry> {
        let mut tel = Telemetry::default();

        if let Some(fan_path) = self.nitro_path("fan_speed") {
            if let Ok(val) = Self::read_sysfs(&fan_path) {
                let parts: Vec<&str> = val.split(',').collect();
                if parts.len() >= 2 {
                    if let Ok(cpu_pct) = parts[0].trim().parse::<u8>() {
                        tel.fans.cpu_percent = Some(cpu_pct);
                        tel.fans.cpu_speed_percent = Some(cpu_pct);
                    }
                    if let Ok(gpu_pct) = parts[1].trim().parse::<u8>() {
                        tel.fans.gpu_percent = Some(gpu_pct);
                        tel.fans.gpu_speed_percent = Some(gpu_pct);
                    }
                }
            }
        }

        tel.sync_all();
        Ok(tel)
    }

    async fn get_thermal_profile(&self) -> Result<ThermalProfile> {
        if let Ok(val) = Self::read_sysfs(Path::new(PLATFORM_PROFILE)) {
            let profile = match val.as_str() {
                "low-power" | "quiet" => ThermalProfile::Silent,
                "balanced" => ThermalProfile::Balanced,
                "balanced-performance" | "performance" => ThermalProfile::Performance,
                "turbo" => ThermalProfile::Turbo,
                _ => ThermalProfile::Balanced,
            };
            return Ok(profile);
        }
        Err(ErrorInfo::not_supported("ACPI platform_profile not found or not readable"))
    }

    async fn set_performance_mode(&self, mode: ThermalProfile) -> Result<()> {
        let choices_str = Self::read_sysfs(Path::new(PLATFORM_PROFILE_CHOICES))
            .unwrap_or_default();
        let choices: Vec<&str> = choices_str.split_whitespace().collect();

        let target_profile = match mode {
            ThermalProfile::Silent => {
                if choices.contains(&"quiet") {
                    "quiet"
                } else if choices.contains(&"low-power") {
                    "low-power"
                } else {
                    return Err(ErrorInfo::not_supported(format!(
                        "Silent profile not available (choices: {})",
                        choices_str
                    )));
                }
            }
            ThermalProfile::Balanced => {
                if choices.contains(&"balanced") {
                    "balanced"
                } else {
                    return Err(ErrorInfo::not_supported(format!(
                        "Balanced profile not available (choices: {})",
                        choices_str
                    )));
                }
            }
            ThermalProfile::Performance => {
                if choices.contains(&"performance") {
                    "performance"
                } else if choices.contains(&"balanced-performance") {
                    "balanced-performance"
                } else {
                    return Err(ErrorInfo::not_supported(format!(
                        "Performance profile not available (choices: {})",
                        choices_str
                    )));
                }
            }
            ThermalProfile::Turbo => {
                // Do NOT fake Turbo as balanced-performance! Report unsupported if hardware lacks explicit Turbo
                if choices.contains(&"turbo") {
                    "turbo"
                } else {
                    return Err(ErrorInfo::not_supported(format!(
                        "Turbo profile is not supported by ACPI platform_profile on this model (available: {})",
                        choices_str
                    )));
                }
            }
            ThermalProfile::Custom => {
                return Err(ErrorInfo::not_supported("Custom thermal profile is not supported by ACPI platform_profile"));
            }
        };

        Self::write_sysfs(Path::new(PLATFORM_PROFILE), target_profile)?;
        info!("[linuwu-sense] set performance mode to {}", target_profile);
        Ok(())
    }

    async fn set_fan_mode(&self, mode: FanMode) -> Result<()> {
        let fan_path = self.nitro_path("fan_speed")
            .ok_or_else(|| ErrorInfo::not_supported("fan_speed attribute not found"))?;

        match mode {
            FanMode::Auto => {
                Self::write_sysfs(&fan_path, "0,0")?;
                info!("[linuwu-sense] fan mode set to Auto (0,0)");
            }
            FanMode::Manual | FanMode::CustomCurve => {
                info!("[linuwu-sense] fan mode set to Manual");
            }
        }
        Ok(())
    }

    async fn set_fan_speed(&self, cpu_percent: u8, gpu_percent: u8) -> Result<()> {
        let fan_path = self.nitro_path("fan_speed")
            .ok_or_else(|| ErrorInfo::not_supported("fan_speed attribute not found"))?;

        let cpu = cpu_percent.min(100);
        let gpu = gpu_percent.min(100);
        let value = format!("{},{}", cpu, gpu);
        Self::write_sysfs(&fan_path, &value)?;
        info!("[linuwu-sense] set fan speed to CPU={}%, GPU={}%", cpu, gpu);
        Ok(())
    }

    async fn set_battery_limit(&self, limit: u8) -> Result<()> {
        let bat_path = self.nitro_path("battery_limiter")
            .ok_or_else(|| ErrorInfo::not_supported("battery_limiter attribute not found"))?;

        if limit != 80 && limit != 100 {
            return Err(ErrorInfo::invalid_argument("Acer battery limiter only supports exactly 80 (on) or 100 (off)"));
        }
        let val = if limit == 80 { "1" } else { "0" };
        Self::write_sysfs(&bat_path, val)?;
        info!("[linuwu-sense] set battery limiter to {}", val);
        Ok(())
    }

    async fn set_keyboard_timeout(&self, timeout_s: u32) -> Result<()> {
        let bl_path = self.nitro_path("backlight_timeout")
            .ok_or_else(|| ErrorInfo::not_supported("backlight_timeout attribute not found"))?;

        Self::write_sysfs(&bl_path, &timeout_s.to_string())?;
        info!("[linuwu-sense] set keyboard backlight timeout to {}s", timeout_s);
        Ok(())
    }
}
