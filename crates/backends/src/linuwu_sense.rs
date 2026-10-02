use acercontrol_core::*;
use crate::HardwareBackend;
use log::{info, warn};
use std::path::Path;

const NITRO_SENSE_BASE: &str = "/sys/module/linuwu_sense/drivers/platform:acer-wmi/acer-wmi/nitro_sense";
const PLATFORM_PROFILE: &str = "/sys/firmware/acpi/platform_profile";
const PLATFORM_PROFILE_CHOICES: &str = "/sys/firmware/acpi/platform_profile_choices";

pub struct Backend;

impl Backend {
    pub fn new() -> Self {
        Self
    }

    fn nitro_path(attr: &str) -> String {
        format!("{}/{}", NITRO_SENSE_BASE, attr)
    }

    fn read_sysfs(path: &str) -> Result<String> {
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .map_err(|e| ErrorInfo::io_error(path, e))
    }

    fn write_sysfs(path: &str, value: &str) -> Result<()> {
        std::fs::write(path, value)
            .map_err(|e| ErrorInfo::io_error(path, e))
    }
}

#[async_trait::async_trait]
impl HardwareBackend for Backend {
    fn name(&self) -> &'static str {
        "linuwu-sense"
    }

    async fn probe(&self) -> ProbeResult {
        let module_exists = Path::new("/sys/module/linuwu_sense").exists();
        let nitro_exists = Path::new(NITRO_SENSE_BASE).exists();

        if !module_exists {
            info!("[linuwu-sense] module not loaded");
            return ProbeResult { available: false, active_capabilities: vec![] };
        }

        if !nitro_exists {
            warn!("[linuwu-sense] module loaded but nitro_sense interface not found");
            return ProbeResult { available: false, active_capabilities: vec![] };
        }

        let mut caps = vec![];

        // Check fan_speed (manual fan control)
        if Path::new(&Self::nitro_path("fan_speed")).exists() {
            caps.push(Capability::FanControl);
        }

        // Check platform_profile (thermal profiles)
        if Path::new(PLATFORM_PROFILE).exists() {
            caps.push(Capability::PerformanceMode);
        }

        // Check battery_limiter
        if Path::new(&Self::nitro_path("battery_limiter")).exists() {
            caps.push(Capability::BatteryChargeLimit);
        }

        // Check backlight_timeout (keyboard backlight)
        if Path::new(&Self::nitro_path("backlight_timeout")).exists() {
            caps.push(Capability::KeyboardBacklight);
        }

        info!("[linuwu-sense] probed OK, capabilities: {:?}", caps);

        ProbeResult {
            available: true,
            active_capabilities: caps,
        }
    }

    async fn capabilities(&self) -> Vec<Capability> {
        let mut caps = vec![];
        if Path::new(&Self::nitro_path("fan_speed")).exists() {
            caps.push(Capability::FanControl);
        }
        if Path::new(PLATFORM_PROFILE).exists() {
            caps.push(Capability::PerformanceMode);
        }
        if Path::new(&Self::nitro_path("battery_limiter")).exists() {
            caps.push(Capability::BatteryChargeLimit);
        }
        if Path::new(&Self::nitro_path("backlight_timeout")).exists() {
            caps.push(Capability::KeyboardBacklight);
        }
        caps
    }

    async fn telemetry(&self) -> Result<Telemetry> {
        // linuwu-sense itself doesn't provide telemetry (hwmon does that via the
        // acer hwmon device it registers). But we can read fan_speed for manual mode.
        let mut tel = Telemetry::default();

        // Read current fan speed setting (manual mode values, comma-separated)
        if let Ok(val) = Self::read_sysfs(&Self::nitro_path("fan_speed")) {
            let parts: Vec<&str> = val.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(cpu_pct) = parts[0].trim().parse::<u8>() {
                    tel.fans.cpu_percent = Some(cpu_pct);
                }
                if let Ok(gpu_pct) = parts[1].trim().parse::<u8>() {
                    tel.fans.gpu_percent = Some(gpu_pct);
                }
            }
        }

        Ok(tel)
    }

    async fn set_performance_mode(&self, mode: ThermalProfile) -> Result<()> {
        let profile_str = match mode {
            ThermalProfile::Silent => "quiet",
            ThermalProfile::Balanced => "balanced",
            ThermalProfile::Performance => "balanced-performance",
            ThermalProfile::Turbo => "balanced-performance",
            ThermalProfile::Custom => return Err(ErrorInfo::not_supported("Custom profile")),
        };

        // Verify the profile is available
        if let Ok(choices) = Self::read_sysfs(PLATFORM_PROFILE_CHOICES) {
            if !choices.split_whitespace().any(|c| c == profile_str) {
                return Err(ErrorInfo::new(format!(
                    "Profile '{}' not available. Available: {}",
                    profile_str, choices
                )));
            }
        }

        Self::write_sysfs(PLATFORM_PROFILE, profile_str)?;
        info!("[linuwu-sense] set performance mode to {}", profile_str);
        Ok(())
    }

    async fn set_fan_mode(&self, mode: FanMode) -> Result<()> {
        match mode {
            FanMode::Auto => {
                // Setting 0,0 returns to automatic fan control
                Self::write_sysfs(&Self::nitro_path("fan_speed"), "0,0")?;
                info!("[linuwu-sense] fan mode set to Auto");
            }
            FanMode::Manual | FanMode::CustomCurve => {
                // Manual mode is enabled when non-zero fan speeds are set
                info!("[linuwu-sense] fan mode set to Manual (use SetFanSpeed to set values)");
            }
        }
        Ok(())
    }

    async fn set_fan_speed(&self, cpu_percent: u8, gpu_percent: u8) -> Result<()> {
        let cpu = cpu_percent.min(100);
        let gpu = gpu_percent.min(100);
        let value = format!("{},{}", cpu, gpu);
        Self::write_sysfs(&Self::nitro_path("fan_speed"), &value)?;
        info!("[linuwu-sense] set fan speed to CPU={}%, GPU={}%", cpu, gpu);
        Ok(())
    }

    async fn set_battery_limit(&self, limit: u8) -> Result<()> {
        // battery_limiter: 1 = enabled (limit to ~80%), 0 = disabled
        let val = if limit < 100 { "1" } else { "0" };
        Self::write_sysfs(&Self::nitro_path("battery_limiter"), val)?;
        info!("[linuwu-sense] set battery limiter to {}", val);
        Ok(())
    }

    async fn set_keyboard_timeout(&self, timeout_s: u32) -> Result<()> {
        Self::write_sysfs(
            &Self::nitro_path("backlight_timeout"),
            &timeout_s.to_string(),
        )?;
        info!("[linuwu-sense] set keyboard backlight timeout to {}s", timeout_s);
        Ok(())
    }
}
