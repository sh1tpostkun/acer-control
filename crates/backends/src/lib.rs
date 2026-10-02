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

// We'll have a backend manager that probes and routes requests to active backends.
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
}
