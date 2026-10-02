use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    PerformanceMode,
    FanControl,
    FanTelemetry,
    BatteryChargeLimit,
    KeyboardBacklight,
    GpuMode,
    CpuPowerLimit,
    GpuPowerLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityStatus {
    Supported,
    Unsupported,
    Unavailable,
    PermissionDenied,
    DriverMissing,
}

impl CapabilityStatus {
    pub fn is_supported(&self) -> bool {
        matches!(self, CapabilityStatus::Supported)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesMap {
    pub fan_control: bool,
    pub fan_telemetry: bool,
    pub thermal_profile: bool,
    pub battery_limit: bool,
    pub keyboard_backlight: bool,
    pub gpu_mode: bool,
    pub cpu_power_limit: bool,
}
