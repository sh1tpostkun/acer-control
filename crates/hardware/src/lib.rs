use acercontrol_core::{
    BatteryStatus, ErrorInfo, FanMode, FanStatus, HardwareCapabilities, PowerStatus, SystemInfo,
    TemperatureStatus, ThermalProfile,
};
use async_trait::async_trait;

#[async_trait]
pub trait HardwareBackend: Send + Sync {
    /// Initialize and detect if the backend is available
    async fn detect(&self) -> Result<bool, ErrorInfo>;
    
    /// Get the backend name
    fn name(&self) -> &'static str;

    /// Get hardware capabilities of the device
    async fn capabilities(&self) -> Result<HardwareCapabilities, ErrorInfo>;

    // Fan Control
    async fn get_fan_status(&self) -> Result<FanStatus, ErrorInfo>;
    async fn set_fan_mode(&self, mode: FanMode) -> Result<(), ErrorInfo>;
    async fn set_fan_speed(&self, cpu_percent: u8, gpu_percent: u8) -> Result<(), ErrorInfo>;

    // Thermal Profiles
    async fn get_thermal_profile(&self) -> Result<ThermalProfile, ErrorInfo>;
    async fn set_thermal_profile(&self, profile: ThermalProfile) -> Result<(), ErrorInfo>;

    // Battery Control
    async fn get_battery_status(&self) -> Result<BatteryStatus, ErrorInfo>;
    async fn set_battery_limit(&self, limit_percentage: u8) -> Result<(), ErrorInfo>;
    async fn set_usb_charging(&self, enabled: bool) -> Result<(), ErrorInfo>;

    // Monitoring
    async fn get_temperatures(&self) -> Result<TemperatureStatus, ErrorInfo>;
    async fn get_power(&self) -> Result<PowerStatus, ErrorInfo>;
    async fn get_hardware_info(&self) -> Result<SystemInfo, ErrorInfo>;
}
pub mod mock;
pub mod linuwu_sense;
