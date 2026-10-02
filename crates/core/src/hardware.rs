use serde::{Deserialize, Serialize};
use crate::capabilities::{Capability, CapabilityDetail};

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
    #[serde(rename = "ram_used")]
    pub ram_used_gb: Option<f32>,
    #[serde(rename = "ram_total")]
    pub ram_total_gb: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "vram_used")]
    pub vram_used_gb: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "vram_total")]
    pub vram_total_gb: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_name: Option<String>,
    #[serde(rename = "disk_used")]
    pub disk_used_gb: Option<f32>,
    #[serde(rename = "disk_total")]
    pub disk_total_gb: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TempTelemetry {
    pub cpu_c: Option<u8>,
    pub gpu_c: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_temp_c: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_temp_c: Option<u8>,
}

impl TempTelemetry {
    pub fn sync_aliases(&mut self) {
        if self.cpu_temp_c.is_none() { self.cpu_temp_c = self.cpu_c; }
        if self.cpu_c.is_none() { self.cpu_c = self.cpu_temp_c; }
        if self.gpu_temp_c.is_none() { self.gpu_temp_c = self.gpu_c; }
        if self.gpu_c.is_none() { self.gpu_c = self.gpu_temp_c; }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FanTelemetry {
    pub mode: Option<FanMode>,
    pub cpu_rpm: Option<u32>,
    pub gpu_rpm: Option<u32>,
    pub cpu_percent: Option<u8>,
    pub gpu_percent: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_speed_percent: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_speed_percent: Option<u8>,
}

impl FanTelemetry {
    pub fn sync_aliases(&mut self) {
        if self.cpu_speed_percent.is_none() { self.cpu_speed_percent = self.cpu_percent; }
        if self.cpu_percent.is_none() { self.cpu_percent = self.cpu_speed_percent; }
        if self.gpu_speed_percent.is_none() { self.gpu_speed_percent = self.gpu_percent; }
        if self.gpu_percent.is_none() { self.gpu_percent = self.gpu_speed_percent; }
    }
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

impl Telemetry {
    pub fn sync_all(&mut self) {
        self.temps.sync_aliases();
        self.fans.sync_aliases();
    }
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
    pub active_capabilities: Vec<Capability>,
    #[serde(default)]
    pub details: Vec<CapabilityDetail>,
}

impl ProbeResult {
    pub fn simple(available: bool, active_capabilities: Vec<Capability>) -> Self {
        let details = active_capabilities.iter().map(|&c| CapabilityDetail::new(c, None::<String>)).collect();
        Self { available, active_capabilities, details }
    }

    pub fn with_details(available: bool, details: Vec<CapabilityDetail>) -> Self {
        let active_capabilities = details.iter().map(|d| d.capability).collect();
        Self { available, active_capabilities, details }
    }
}
