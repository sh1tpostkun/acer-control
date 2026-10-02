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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityStatus {
    Supported { backend: String },
    Unsupported,
    Unavailable,
    PermissionDenied,
    DriverMissing,
}

impl CapabilityStatus {
    pub fn is_supported(&self) -> bool {
        matches!(self, CapabilityStatus::Supported { .. })
    }
}

/// Rich capabilities map sent to GUI — each field carries status + backend info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesMap {
    pub fan_control: CapabilityStatus,
    pub fan_telemetry: CapabilityStatus,
    pub thermal_profile: CapabilityStatus,
    pub battery_limit: CapabilityStatus,
    pub keyboard_backlight: CapabilityStatus,
    pub gpu_mode: CapabilityStatus,
    pub cpu_power_limit: CapabilityStatus,
    pub gpu_power_limit: CapabilityStatus,
}

impl Default for CapabilitiesMap {
    fn default() -> Self {
        Self {
            fan_control: CapabilityStatus::Unsupported,
            fan_telemetry: CapabilityStatus::Unsupported,
            thermal_profile: CapabilityStatus::Unsupported,
            battery_limit: CapabilityStatus::Unsupported,
            keyboard_backlight: CapabilityStatus::Unsupported,
            gpu_mode: CapabilityStatus::Unsupported,
            cpu_power_limit: CapabilityStatus::Unsupported,
            gpu_power_limit: CapabilityStatus::Unsupported,
        }
    }
}
