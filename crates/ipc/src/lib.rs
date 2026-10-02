use serde::{Deserialize, Serialize};
use acercontrol_core::{
    FanMode, ThermalProfile, Capability, Telemetry, SystemIdentification, CapabilitiesMap,
};

pub const SOCKET_PATH: &str = "/tmp/acercontrol.sock";

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
    DoctorReport(String),
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
