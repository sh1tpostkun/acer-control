use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FanMode {
    Auto,
    Manual,
    CustomCurve,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThermalProfile {
    Silent,
    Balanced,
    Performance,
    Turbo,
    Custom,
}

impl std::fmt::Display for ThermalProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThermalProfile::Silent => write!(f, "Silent"),
            ThermalProfile::Balanced => write!(f, "Balanced"),
            ThermalProfile::Performance => write!(f, "Performance"),
            ThermalProfile::Turbo => write!(f, "Turbo"),
            ThermalProfile::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SystemTelemetry {
    pub cpu_usage: Option<f32>,
    pub gpu_usage: Option<f32>,
    pub ram_used_gb: Option<f32>,
    pub ram_total_gb: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TempTelemetry {
    pub cpu_c: Option<u8>,
    pub gpu_c: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FanTelemetry {
    pub mode: Option<FanMode>,
    pub cpu_rpm: Option<u32>,
    pub gpu_rpm: Option<u32>,
    pub cpu_percent: Option<u8>,
    pub gpu_percent: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PowerTelemetry {
    pub consumption_w: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BatteryTelemetry {
    pub percentage: Option<u8>,
    pub is_charging: Option<bool>,
    pub charge_limit: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Telemetry {
    pub system: SystemTelemetry,
    pub temps: TempTelemetry,
    pub fans: FanTelemetry,
    pub power: PowerTelemetry,
    pub battery: BatteryTelemetry,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemIdentification {
    pub vendor: String,
    pub product: String,
    pub version: String,
    pub board: String,
}

impl SystemIdentification {
    pub fn probe() -> Self {
        let read_dmi = |name: &str| -> String {
            std::fs::read_to_string(format!("/sys/devices/virtual/dmi/id/{}", name))
                .unwrap_or_default()
                .trim()
                .to_string()
        };

        Self {
            vendor: read_dmi("sys_vendor"),
            product: read_dmi("product_name"),
            version: read_dmi("product_version"),
            board: read_dmi("board_name"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProbeResult {
    pub available: bool,
    pub active_capabilities: Vec<crate::capabilities::Capability>,
}
