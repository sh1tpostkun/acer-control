use acercontrol_core::*;
use crate::HardwareBackend;

/// Stub: acer-wmi direct WMI interface.
/// Currently linuwu-sense provides the same functionality via its sysfs bridge.
/// This backend exists for potential future direct WMI access.
pub struct Backend;

#[async_trait::async_trait]
impl HardwareBackend for Backend {
    fn name(&self) -> &'static str { "acer-wmi" }

    async fn probe(&self) -> ProbeResult {
        // linuwu-sense already wraps acer-wmi, so we don't double-register
        ProbeResult::simple(false, vec![])
    }

    async fn capabilities(&self) -> Vec<Capability> { vec![] }
    async fn telemetry(&self) -> Result<Telemetry> { Ok(Telemetry::default()) }
    async fn get_thermal_profile(&self) -> Result<ThermalProfile> { Err(ErrorInfo::not_supported("acer-wmi direct access")) }
    async fn set_performance_mode(&self, _mode: ThermalProfile) -> Result<()> { Err(ErrorInfo::not_supported("acer-wmi direct access")) }
    async fn set_fan_mode(&self, _mode: FanMode) -> Result<()> { Err(ErrorInfo::not_supported("acer-wmi direct access")) }
    async fn set_fan_speed(&self, _cpu: u8, _gpu: u8) -> Result<()> { Err(ErrorInfo::not_supported("acer-wmi direct access")) }
    async fn set_battery_limit(&self, _limit: u8) -> Result<()> { Err(ErrorInfo::not_supported("acer-wmi direct access")) }
    async fn set_keyboard_timeout(&self, _t: u32) -> Result<()> { Err(ErrorInfo::not_supported("acer-wmi direct access")) }
}
