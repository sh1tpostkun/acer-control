use serde::{Deserialize, Serialize};
use acercontrol_core::{
    BatteryStatus, FanMode, FanStatus, HardwareCapabilities, PowerStatus, SystemInfo,
    TemperatureStatus, ThermalProfile,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IpcRequest {
    GetCapabilities,
    
    GetFanStatus,
    SetFanMode(FanMode),
    SetFanSpeed { cpu_percent: u8, gpu_percent: u8 },
    
    GetThermalProfile,
    SetThermalProfile(ThermalProfile),
    
    GetBatteryStatus,
    SetBatteryLimit(u8),
    SetUsbCharging(bool),
    
    GetTemperatures,
    GetPower,
    GetSystemInfo,
    
    SubscribeEvents,
    Ping,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IpcResponse {
    Ok,
    Error(String),
    Capabilities(HardwareCapabilities),
    FanStatus(FanStatus),
    ThermalProfile(ThermalProfile),
    BatteryStatus(BatteryStatus),
    Temperatures(TemperatureStatus),
    Power(PowerStatus),
    SystemInfo(SystemInfo),
    Pong,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IpcEvent {
    Telemetry {
        temps: TemperatureStatus,
        fans: FanStatus,
        power: PowerStatus,
        battery: BatteryStatus,
        system: SystemInfo,
    },
    ProfileChanged(ThermalProfile),
    ToggleGui {},
}

pub const SOCKET_PATH: &str = "/tmp/acercontrol.sock";
