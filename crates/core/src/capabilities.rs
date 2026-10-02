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
