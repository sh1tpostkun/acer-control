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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Telemetry {
    pub cpu_temp_c: Option<u8>,
    pub gpu_temp_c: Option<u8>,
    pub cpu_rpm: Option<u32>,
    pub gpu_rpm: Option<u32>,
    pub cpu_speed_percent: Option<u8>,
    pub gpu_speed_percent: Option<u8>,
    pub power_consumption_w: Option<f32>,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            cpu_temp_c: None,
            gpu_temp_c: None,
            cpu_rpm: None,
            gpu_rpm: None,
            cpu_speed_percent: None,
            gpu_speed_percent: None,
            power_consumption_w: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemIdentification {
    pub vendor: String,
    pub product: String,
    pub version: String,
    pub board: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProbeResult {
    pub available: bool,
    pub active_capabilities: Vec<crate::capabilities::Capability>,
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
pub struct HardwareCapabilities {
    pub fan_control: bool,
    pub thermal_profile: bool,
    pub battery_limit: bool,
    pub usb_charging: bool,
    pub keyboard_rgb: bool,
    pub battery_calibration: bool,
}

impl Default for HardwareCapabilities {
    fn default() -> Self {
        Self { fan_control: false, thermal_profile: false, battery_limit: false, usb_charging: false, keyboard_rgb: false, battery_calibration: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FanStatus {
    pub mode: FanMode,
    pub cpu_rpm: u32,
    pub gpu_rpm: u32,
    pub cpu_speed_percent: u8,
    pub gpu_speed_percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BatteryStatus {
    pub percentage: u8,
    pub is_charging: bool,
    pub ac_connected: bool,
    pub health: u8,
    pub cycle_count: u32,
    pub charge_limit: u8,
    pub usb_charging_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PowerStatus {
    pub power_consumption_w: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemperatureStatus {
    pub cpu_temp_c: u8,
    pub gpu_temp_c: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInfo {
    pub device_name: String,
    pub laptop_model: String,
    pub gpu_name: String,
    pub disk_name: String,
    pub cpu_usage: f32,
    pub gpu_usage: f32,
    pub ram_total: f32,
    pub ram_used: f32,
    pub vram_total: f32,
    pub vram_used: f32,
    pub disk_total: f32,
    pub disk_used: f32,
    pub cpu_freq_mhz: u32,
    pub gpu_freq_mhz: u32,
}
