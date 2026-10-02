use acercontrol_core::*;
use log::{info, warn};

pub mod acer_wmi;
pub mod hwmon;
pub mod linuwu_sense;
pub mod nvml;
pub mod sysfs;

#[async_trait::async_trait]
pub trait HardwareBackend: Send + Sync {
    fn name(&self) -> &'static str;
    async fn probe(&self) -> ProbeResult;
    async fn capabilities(&self) -> Vec<Capability>;
    async fn telemetry(&self) -> Result<Telemetry>;
    async fn set_performance_mode(&self, mode: ThermalProfile) -> Result<()>;
    async fn set_fan_mode(&self, mode: FanMode) -> Result<()>;
    async fn set_fan_speed(&self, cpu_percent: u8, gpu_percent: u8) -> Result<()>;
    async fn set_battery_limit(&self, limit: u8) -> Result<()>;
    async fn set_keyboard_timeout(&self, timeout_s: u32) -> Result<()>;
}

/// Telemetry source ownership — defines which backend is authoritative for each
/// data source.  Earlier entries in `backends` vec have higher priority; the FIRST
/// backend that supplies a non-None value wins.
pub struct BackendManager {
    backends: Vec<Box<dyn HardwareBackend>>,
    active_backends: Vec<String>,
}

impl BackendManager {
    pub fn new() -> Self {
        Self {
            backends: vec![],
            active_backends: vec![],
        }
    }

    pub fn add_backend(&mut self, backend: Box<dyn HardwareBackend>) {
        self.backends.push(backend);
    }

    pub async fn initialize(&mut self) {
        for backend in &self.backends {
            let probe = backend.probe().await;
            if probe.available {
                info!("[BackendManager] '{}' is active, capabilities: {:?}",
                      backend.name(), probe.active_capabilities);
                self.active_backends.push(backend.name().to_string());
            } else {
                info!("[BackendManager] '{}' is not available", backend.name());
            }
        }
        info!("[BackendManager] active backends: {:?}", self.active_backends);
    }

    pub fn active_backends(&self) -> &[String] {
        &self.active_backends
    }

    fn is_active(&self, name: &str) -> bool {
        self.active_backends.iter().any(|b| b == name)
    }

    /// Build a rich capabilities map with status + backend attribution.
    pub async fn get_capabilities(&self) -> CapabilitiesMap {
        let mut map = CapabilitiesMap::default();

        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            let caps = backend.capabilities().await;
            let bname = backend.name().to_string();

            for cap in &caps {
                let status = CapabilityStatus::Supported { backend: bname.clone() };
                match cap {
                    Capability::FanControl => if !map.fan_control.is_supported() { map.fan_control = status; },
                    Capability::FanTelemetry => if !map.fan_telemetry.is_supported() { map.fan_telemetry = status; },
                    Capability::PerformanceMode => if !map.thermal_profile.is_supported() { map.thermal_profile = status; },
                    Capability::BatteryChargeLimit => if !map.battery_limit.is_supported() { map.battery_limit = status; },
                    Capability::KeyboardBacklight => if !map.keyboard_backlight.is_supported() { map.keyboard_backlight = status; },
                    Capability::GpuMode => if !map.gpu_mode.is_supported() { map.gpu_mode = status; },
                    Capability::CpuPowerLimit => if !map.cpu_power_limit.is_supported() { map.cpu_power_limit = status; },
                    Capability::GpuPowerLimit => if !map.gpu_power_limit.is_supported() { map.gpu_power_limit = status; },
                }
            }
        }
        map
    }

    /// Aggregate telemetry from all active backends.
    /// First-write-wins: the first backend (by registration order) that provides
    /// a non-None value is authoritative for that field.
    pub async fn get_telemetry(&self) -> Result<Telemetry> {
        let mut agg = Telemetry::default();

        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            match backend.telemetry().await {
                Ok(tel) => {
                    // First-write-wins for each field
                    if agg.temps.cpu_c.is_none() && tel.temps.cpu_c.is_some() {
                        agg.temps.cpu_c = tel.temps.cpu_c;
                    }
                    if agg.temps.gpu_c.is_none() && tel.temps.gpu_c.is_some() {
                        agg.temps.gpu_c = tel.temps.gpu_c;
                    }
                    if agg.fans.cpu_rpm.is_none() && tel.fans.cpu_rpm.is_some() {
                        agg.fans.cpu_rpm = tel.fans.cpu_rpm;
                    }
                    if agg.fans.gpu_rpm.is_none() && tel.fans.gpu_rpm.is_some() {
                        agg.fans.gpu_rpm = tel.fans.gpu_rpm;
                    }
                    if agg.fans.cpu_percent.is_none() && tel.fans.cpu_percent.is_some() {
                        agg.fans.cpu_percent = tel.fans.cpu_percent;
                    }
                    if agg.fans.gpu_percent.is_none() && tel.fans.gpu_percent.is_some() {
                        agg.fans.gpu_percent = tel.fans.gpu_percent;
                    }
                    if agg.fans.mode.is_none() && tel.fans.mode.is_some() {
                        agg.fans.mode = tel.fans.mode;
                    }
                    if agg.power.consumption_w.is_none() && tel.power.consumption_w.is_some() {
                        agg.power.consumption_w = tel.power.consumption_w;
                    }
                    if agg.battery.percentage.is_none() && tel.battery.percentage.is_some() {
                        agg.battery.percentage = tel.battery.percentage;
                    }
                    if agg.battery.is_charging.is_none() && tel.battery.is_charging.is_some() {
                        agg.battery.is_charging = tel.battery.is_charging;
                    }
                    if agg.battery.charge_limit.is_none() && tel.battery.charge_limit.is_some() {
                        agg.battery.charge_limit = tel.battery.charge_limit;
                    }
                    if agg.system.cpu_usage.is_none() && tel.system.cpu_usage.is_some() {
                        agg.system.cpu_usage = tel.system.cpu_usage;
                    }
                    if agg.system.gpu_usage.is_none() && tel.system.gpu_usage.is_some() {
                        agg.system.gpu_usage = tel.system.gpu_usage;
                    }
                    if agg.system.ram_used_gb.is_none() && tel.system.ram_used_gb.is_some() {
                        agg.system.ram_used_gb = tel.system.ram_used_gb;
                    }
                    if agg.system.ram_total_gb.is_none() && tel.system.ram_total_gb.is_some() {
                        agg.system.ram_total_gb = tel.system.ram_total_gb;
                    }
                }
                Err(e) => {
                    warn!("[BackendManager] telemetry error from '{}': {}", backend.name(), e);
                }
            }
        }
        Ok(agg)
    }

    /// Route a set_performance_mode to the first capable backend.
    /// Returns the backend's actual error if it fails.
    pub async fn set_performance_mode(&self, mode: ThermalProfile) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) { continue; }
            if backend.capabilities().await.contains(&Capability::PerformanceMode) {
                return backend.set_performance_mode(mode).await;
            }
        }
        Err(ErrorInfo::not_supported("No backend supports performance modes"))
    }

    pub async fn set_fan_mode(&self, mode: FanMode) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) { continue; }
            if backend.capabilities().await.contains(&Capability::FanControl) {
                return backend.set_fan_mode(mode).await;
            }
        }
        Err(ErrorInfo::not_supported("No backend supports fan control"))
    }

    pub async fn set_fan_speed(&self, cpu: u8, gpu: u8) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) { continue; }
            if backend.capabilities().await.contains(&Capability::FanControl) {
                return backend.set_fan_speed(cpu, gpu).await;
            }
        }
        Err(ErrorInfo::not_supported("No backend supports fan speed control"))
    }

    pub async fn set_battery_limit(&self, limit: u8) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) { continue; }
            if backend.capabilities().await.contains(&Capability::BatteryChargeLimit) {
                return backend.set_battery_limit(limit).await;
            }
        }
        Err(ErrorInfo::not_supported("No backend supports battery limit"))
    }

    pub async fn set_keyboard_timeout(&self, timeout_s: u32) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) { continue; }
            if backend.capabilities().await.contains(&Capability::KeyboardBacklight) {
                return backend.set_keyboard_timeout(timeout_s).await;
            }
        }
        Err(ErrorInfo::not_supported("No backend supports keyboard backlight"))
    }

    /// Get the current platform profile string.
    pub async fn get_thermal_profile(&self) -> Result<ThermalProfile> {
        if let Ok(val) = std::fs::read_to_string("/sys/firmware/acpi/platform_profile") {
            let profile = match val.trim() {
                "low-power" | "quiet" => ThermalProfile::Silent,
                "balanced" => ThermalProfile::Balanced,
                "balanced-performance" | "performance" => ThermalProfile::Performance,
                _ => ThermalProfile::Balanced,
            };
            return Ok(profile);
        }
        Ok(ThermalProfile::Balanced)
    }
}
