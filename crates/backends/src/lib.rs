use acercontrol_core::*;
use log::{info, warn};
use std::collections::HashMap;

pub mod acer_wmi;
pub mod hwmon;
pub mod linuwu_sense;
pub mod nvml;
pub mod sysfs;
pub mod os_stats;

#[async_trait::async_trait]
pub trait HardwareBackend: Send + Sync {
    fn name(&self) -> &'static str;
    async fn probe(&self) -> ProbeResult;
    async fn capabilities(&self) -> Vec<Capability>;
    async fn telemetry(&self) -> Result<Telemetry>;
    async fn get_thermal_profile(&self) -> Result<ThermalProfile>;
    async fn set_performance_mode(&self, mode: ThermalProfile) -> Result<()>;
    async fn set_fan_mode(&self, mode: FanMode) -> Result<()>;
    async fn set_fan_speed(&self, cpu_percent: u8, gpu_percent: u8) -> Result<()>;
    async fn set_battery_limit(&self, limit: u8) -> Result<()>;
    async fn set_keyboard_timeout(&self, timeout_s: u32) -> Result<()>;
}

pub struct BackendManager {
    backends: Vec<Box<dyn HardwareBackend>>,
    active_backends: Vec<String>,
    capability_details: HashMap<String, Vec<CapabilityDetail>>,
}

impl BackendManager {
    pub fn new() -> Self {
        Self {
            backends: vec![],
            active_backends: vec![],
            capability_details: HashMap::new(),
        }
    }

    pub fn add_backend(&mut self, backend: Box<dyn HardwareBackend>) {
        self.backends.push(backend);
    }

    pub async fn initialize(&mut self) {
        for backend in &self.backends {
            let probe = backend.probe().await;
            if probe.available {
                info!(
                    "[BackendManager] '{}' is active, capabilities: {:?}",
                    backend.name(),
                    probe.active_capabilities
                );
                self.active_backends.push(backend.name().to_string());
                self.capability_details
                    .insert(backend.name().to_string(), probe.details);
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

    /// Build a rich capabilities map with status, backend attribution, and source path.
    pub async fn get_capabilities(&self) -> CapabilitiesMap {
        let mut map = CapabilitiesMap::default();

        for backend in &self.backends {
            let bname = backend.name();
            if !self.is_active(bname) {
                continue;
            }

            let details = self.capability_details.get(bname);
            let caps = backend.capabilities().await;

            for cap in &caps {
                let source = details.and_then(|d_list| {
                    d_list.iter().find(|d| d.capability == *cap).and_then(|d| d.source.clone())
                });

                let status = CapabilityStatus::Supported {
                    backend: bname.to_string(),
                    source,
                };

                match cap {
                    Capability::FanControl => {
                        if !map.fan_control.is_supported() {
                            map.fan_control = status;
                        }
                    }
                    Capability::FanTelemetry => {
                        if !map.fan_telemetry.is_supported() {
                            map.fan_telemetry = status;
                        }
                    }
                    Capability::GpuTelemetry => {
                        if !map.gpu_telemetry.is_supported() {
                            map.gpu_telemetry = status;
                        }
                    }
                    Capability::PerformanceMode => {
                        if !map.thermal_profile.is_supported() {
                            map.thermal_profile = status;
                        }
                    }
                    Capability::BatteryChargeLimit => {
                        if !map.battery_limit.is_supported() {
                            map.battery_limit = status;
                        }
                    }
                    Capability::KeyboardBacklight => {
                        if !map.keyboard_backlight.is_supported() {
                            map.keyboard_backlight = status;
                        }
                    }
                    Capability::GpuMode => {
                        if !map.gpu_mode.is_supported() {
                            map.gpu_mode = status;
                        }
                    }
                    Capability::CpuPowerLimit => {
                        if !map.cpu_power_limit.is_supported() {
                            map.cpu_power_limit = status;
                        }
                    }
                    Capability::GpuPowerLimit => {
                        if !map.gpu_power_limit.is_supported() {
                            map.gpu_power_limit = status;
                        }
                    }
                }
            }
        }
        map
    }

    /// Aggregate telemetry from all active backends with source priority:
    /// - NVML provides authoritative GPU telemetry (name, temp, utilization, VRAM, power)
    /// - hwmon provides authoritative fan RPM and CPU temp, plus battery
    /// - linuwu-sense provides active fan speed percentages & mode
    pub async fn get_telemetry(&self) -> Result<Telemetry> {
        let mut agg = Telemetry::default();

        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            match backend.telemetry().await {
                Ok(tel) => {
                    // CPU Temperature (prefer hwmon)
                    if agg.temps.cpu_c.is_none() && tel.temps.cpu_c.is_some() {
                        agg.temps.cpu_c = tel.temps.cpu_c;
                    }

                    // GPU Temperature (prefer NVML if present, else fallback)
                    if backend.name() == "nvml" && tel.temps.gpu_c.is_some() {
                        agg.temps.gpu_c = tel.temps.gpu_c;
                    } else if agg.temps.gpu_c.is_none() && tel.temps.gpu_c.is_some() {
                        agg.temps.gpu_c = tel.temps.gpu_c;
                    }

                    // Fans RPM
                    if agg.fans.cpu_rpm.is_none() && tel.fans.cpu_rpm.is_some() {
                        agg.fans.cpu_rpm = tel.fans.cpu_rpm;
                    }
                    if agg.fans.gpu_rpm.is_none() && tel.fans.gpu_rpm.is_some() {
                        agg.fans.gpu_rpm = tel.fans.gpu_rpm;
                    }

                    // Fan Manual Speed Percentages
                    if agg.fans.cpu_percent.is_none() && tel.fans.cpu_percent.is_some() {
                        agg.fans.cpu_percent = tel.fans.cpu_percent;
                    }
                    if agg.fans.gpu_percent.is_none() && tel.fans.gpu_percent.is_some() {
                        agg.fans.gpu_percent = tel.fans.gpu_percent;
                    }
                    if agg.fans.mode.is_none() && tel.fans.mode.is_some() {
                        agg.fans.mode = tel.fans.mode;
                    }

                    // Power
                    if agg.power.consumption_w.is_none() && tel.power.consumption_w.is_some() {
                        agg.power.consumption_w = tel.power.consumption_w;
                    }

                    // Battery
                    if agg.battery.percentage.is_none() && tel.battery.percentage.is_some() {
                        agg.battery.percentage = tel.battery.percentage;
                    }
                    if agg.battery.is_charging.is_none() && tel.battery.is_charging.is_some() {
                        agg.battery.is_charging = tel.battery.is_charging;
                    }
                    if agg.battery.charge_limit.is_none() && tel.battery.charge_limit.is_some() {
                        agg.battery.charge_limit = tel.battery.charge_limit;
                    }

                    // System / GPU details
                    if agg.system.cpu_usage.is_none() && tel.system.cpu_usage.is_some() {
                        agg.system.cpu_usage = tel.system.cpu_usage;
                    }
                    if agg.system.gpu_usage.is_none() && tel.system.gpu_usage.is_some() {
                        agg.system.gpu_usage = tel.system.gpu_usage;
                    }
                    if agg.system.gpu_name.is_none() && tel.system.gpu_name.is_some() {
                        agg.system.gpu_name = tel.system.gpu_name;
                    }
                    if agg.system.vram_used_gb.is_none() && tel.system.vram_used_gb.is_some() {
                        agg.system.vram_used_gb = tel.system.vram_used_gb;
                    }
                    if agg.system.vram_total_gb.is_none() && tel.system.vram_total_gb.is_some() {
                        agg.system.vram_total_gb = tel.system.vram_total_gb;
                    }
                    if agg.system.ram_used_gb.is_none() && tel.system.ram_used_gb.is_some() {
                        agg.system.ram_used_gb = tel.system.ram_used_gb;
                    }
                    if agg.system.ram_total_gb.is_none() && tel.system.ram_total_gb.is_some() {
                        agg.system.ram_total_gb = tel.system.ram_total_gb;
                    }
                    if agg.system.disk_used_gb.is_none() && tel.system.disk_used_gb.is_some() {
                        agg.system.disk_used_gb = tel.system.disk_used_gb;
                    }
                    if agg.system.disk_total_gb.is_none() && tel.system.disk_total_gb.is_some() {
                        agg.system.disk_total_gb = tel.system.disk_total_gb;
                    }
                }
                Err(e) => {
                    warn!(
                        "[BackendManager] telemetry error from '{}': {}",
                        backend.name(),
                        e
                    );
                }
            }
        }

        agg.sync_all();
        Ok(agg)
    }

    /// Route a set_performance_mode to the first successfully responding capable backend.
    pub async fn set_performance_mode(&self, mode: ThermalProfile) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            if backend.capabilities().await.contains(&Capability::PerformanceMode) {
                match backend.set_performance_mode(mode.clone()).await {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        warn!("Backend {} failed to set performance mode: {}", backend.name(), e);
                        continue;
                    }
                }
            }
        }
        Err(ErrorInfo::not_supported(
            "No active backend successfully supported ACPI performance modes on this hardware",
        ))
    }

    pub async fn set_fan_mode(&self, mode: FanMode) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            if backend.capabilities().await.contains(&Capability::FanControl) {
                match backend.set_fan_mode(mode.clone()).await {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        warn!("Backend {} failed to set fan mode: {}", backend.name(), e);
                        continue;
                    }
                }
            }
        }
        Err(ErrorInfo::not_supported(
            "No active backend successfully supported fan mode control on this hardware",
        ))
    }

    pub async fn set_fan_speed(&self, cpu: u8, gpu: u8) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            if backend.capabilities().await.contains(&Capability::FanControl) {
                match backend.set_fan_speed(cpu, gpu).await {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        warn!("Backend {} failed to set fan speed: {}", backend.name(), e);
                        continue;
                    }
                }
            }
        }
        Err(ErrorInfo::not_supported(
            "No active backend successfully supported fan speed adjustment on this hardware",
        ))
    }

    pub async fn set_battery_limit(&self, limit: u8) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            if backend.capabilities().await.contains(&Capability::BatteryChargeLimit) {
                match backend.set_battery_limit(limit).await {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        warn!("Backend {} failed to set battery limit: {}", backend.name(), e);
                        continue;
                    }
                }
            }
        }
        Err(ErrorInfo::not_supported(
            "No active backend successfully supported battery charge limiting on this hardware",
        ))
    }

    pub async fn set_keyboard_timeout(&self, timeout_s: u32) -> Result<()> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            if backend.capabilities().await.contains(&Capability::KeyboardBacklight) {
                match backend.set_keyboard_timeout(timeout_s).await {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        warn!("Backend {} failed to set keyboard timeout: {}", backend.name(), e);
                        continue;
                    }
                }
            }
        }
        Err(ErrorInfo::not_supported(
            "No active backend successfully supported keyboard backlight timeout on this hardware",
        ))
    }

    /// Read the current active platform profile from the first successfully responding capable backend.
    pub async fn get_thermal_profile(&self) -> Result<ThermalProfile> {
        for backend in &self.backends {
            if !self.is_active(backend.name()) {
                continue;
            }
            if backend.capabilities().await.contains(&Capability::PerformanceMode) {
                match backend.get_thermal_profile().await {
                    Ok(profile) => return Ok(profile),
                    Err(e) => {
                        warn!("Backend {} failed to get thermal profile: {}", backend.name(), e);
                        continue;
                    }
                }
            }
        }
        Err(ErrorInfo::not_supported(
            "No active backend successfully provided the current thermal profile on this hardware",
        ))
    }
}

