use acercontrol_core::*;


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
    async fn set_battery_limit(&self, limit: u8) -> Result<()>;
}

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
                self.active_backends.push(backend.name().to_string());
            }
        }
    }

    pub fn active_backends(&self) -> &[String] {
        &self.active_backends
    }

    pub async fn get_capabilities(&self) -> CapabilitiesMap {
        let mut map = CapabilitiesMap {
            fan_control: false,
            fan_telemetry: false,
            thermal_profile: false,
            battery_limit: false,
            keyboard_backlight: false,
            gpu_mode: false,
            cpu_power_limit: false,
        };

        for backend in &self.backends {
            if self.active_backends.contains(&backend.name().to_string()) {
                let caps = backend.capabilities().await;
                if caps.contains(&Capability::FanControl) { map.fan_control = true; }
                if caps.contains(&Capability::FanTelemetry) { map.fan_telemetry = true; }
                if caps.contains(&Capability::PerformanceMode) { map.thermal_profile = true; }
                if caps.contains(&Capability::BatteryChargeLimit) { map.battery_limit = true; }
                if caps.contains(&Capability::KeyboardBacklight) { map.keyboard_backlight = true; }
                if caps.contains(&Capability::GpuMode) { map.gpu_mode = true; }
                if caps.contains(&Capability::CpuPowerLimit) { map.cpu_power_limit = true; }
            }
        }
        map
    }

    pub async fn get_telemetry(&self) -> Result<Telemetry> {
        let mut agg = Telemetry::default();
        for backend in &self.backends {
            if self.active_backends.contains(&backend.name().to_string()) {
                if let Ok(tel) = backend.telemetry().await {
                    if tel.temps.cpu_c.is_some() { agg.temps.cpu_c = tel.temps.cpu_c; }
                    if tel.temps.gpu_c.is_some() { agg.temps.gpu_c = tel.temps.gpu_c; }
                    if tel.fans.cpu_rpm.is_some() { agg.fans.cpu_rpm = tel.fans.cpu_rpm; }
                    if tel.fans.gpu_rpm.is_some() { agg.fans.gpu_rpm = tel.fans.gpu_rpm; }
                    if tel.power.consumption_w.is_some() { agg.power.consumption_w = tel.power.consumption_w; }
                }
            }
        }
        Ok(agg)
    }

    pub async fn set_performance_mode(&self, mode: ThermalProfile) -> Result<()> {
        for backend in &self.backends {
            if self.active_backends.contains(&backend.name().to_string()) {
                if backend.capabilities().await.contains(&Capability::PerformanceMode) {
                    let _ = backend.set_performance_mode(mode.clone()).await;
                }
            }
        }
        Ok(())
    }

    pub async fn set_fan_mode(&self, mode: FanMode) -> Result<()> {
        for backend in &self.backends {
            if self.active_backends.contains(&backend.name().to_string()) {
                if backend.capabilities().await.contains(&Capability::FanControl) {
                    let _ = backend.set_fan_mode(mode.clone()).await;
                }
            }
        }
        Ok(())
    }

    pub async fn set_battery_limit(&self, limit: u8) -> Result<()> {
        for backend in &self.backends {
            if self.active_backends.contains(&backend.name().to_string()) {
                if backend.capabilities().await.contains(&Capability::BatteryChargeLimit) {
                    let _ = backend.set_battery_limit(limit).await;
                }
            }
        }
        Ok(())
    }
}
