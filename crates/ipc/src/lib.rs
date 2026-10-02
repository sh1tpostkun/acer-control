use serde::{Deserialize, Serialize};
use acercontrol_core::{
    FanMode, ThermalProfile, Capability, Telemetry, SystemIdentification,
    CapabilityStatus,
};

pub const SOCKET_PATH: &str = "/var/run/acercontrol.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcRequest {
    Ping,
    Doctor,
    GetCapabilities,
    GetTelemetry,
    GetSystemIdentification,
    SetThermalProfile(ThermalProfile),
    SetFanMode(FanMode),
    SetBatteryLimit(u8),
    SubscribeEvents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    Ok,
    Pong,
    Error(String),
    DoctorReport(String),
    Capabilities(Vec<Capability>),
    Telemetry(Telemetry),
    SystemIdentification(SystemIdentification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcEvent {
    TelemetryUpdate(Telemetry),
    ProfileChanged(ThermalProfile),
}
