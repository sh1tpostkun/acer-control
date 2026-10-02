use serde::{Deserialize, Serialize};
use acercontrol_core::{
    FanMode, ThermalProfile, Telemetry, SystemIdentification, CapabilitiesMap,
};

pub const SOCKET_PATH: &str = "/run/acercontrol/daemon.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorData {
    pub system: SystemIdentification,
    pub kernel: String,
    pub active_backends: Vec<String>,
    pub capabilities: CapabilitiesMap,
    pub telemetry: Telemetry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcRequest {
    Ping,
    Doctor,
    GetCapabilities,
    GetTelemetry,
    GetSystemIdentification,
    GetThermalProfile,
    SetThermalProfile { profile: ThermalProfile },
    SetFanMode { mode: FanMode },
    SetFanSpeed { cpu_percent: u8, gpu_percent: u8 },
    SetBatteryLimit { limit: u8 },
    SetKeyboardTimeout { timeout_s: u32 },
    SetWifiEnabled { enabled: bool },
    SetBluetoothEnabled { enabled: bool },
    DropCaches,
    SubscribeEvents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    Ok,
    Pong,
    Error(String),
    DoctorReport {
        text: String,
        data: DoctorData,
    },
    Capabilities(CapabilitiesMap),
    Telemetry(Telemetry),
    SystemIdentification(SystemIdentification),
    ThermalProfile(ThermalProfile),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcEvent {
    TelemetryUpdate(Telemetry),
    ProfileChanged(ThermalProfile),
}
