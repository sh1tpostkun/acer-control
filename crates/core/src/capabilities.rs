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
pub struct CapabilityDetail {
    pub capability: Capability,
    pub source: Option<String>,
}

impl CapabilityDetail {
    pub fn new(capability: Capability, source: Option<impl Into<String>>) -> Self {
        Self {
            capability,
            source: source.map(|s| s.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum CapabilityStatus {
    Supported {
        backend: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<String>,
    },
    Unsupported,
    Unavailable,
    PermissionDenied,
    DriverMissing,
}

impl CapabilityStatus {
    pub fn is_supported(&self) -> bool {
        matches!(self, CapabilityStatus::Supported { .. })
    }

    pub fn backend(&self) -> Option<&str> {
        match self {
            CapabilityStatus::Supported { backend, .. } => Some(backend),
            _ => None,
        }
    }

    pub fn source(&self) -> Option<&str> {
        match self {
            CapabilityStatus::Supported { source: Some(s), .. } => Some(s),
            _ => None,
        }
    }
}

/// Rich capabilities map sent to GUI and CLI — each field carries status, backend, and source info.
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
