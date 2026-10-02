use acercontrol_core::*;
use crate::HardwareBackend;

pub struct Backend;

#[async_trait::async_trait]
impl HardwareBackend for Backend {
    fn name(&self) -> &'static str { "acer_wmi" }
    async fn probe(&self) -> ProbeResult { ProbeResult { available: false, active_capabilities: vec![] } }
    async fn capabilities(&self) -> Vec<Capability> { vec![] }
    async fn telemetry(&self) -> Result<Telemetry> { Ok(Telemetry::default()) }
    async fn set_performance_mode(&self, _mode: ThermalProfile) -> Result<()> { Err(ErrorInfo { message: "Not supported".into() }) }
    async fn set_fan_mode(&self, _mode: FanMode) -> Result<()> { Err(ErrorInfo { message: "Not supported".into() }) }
    async fn set_battery_limit(&self, _limit: u8) -> Result<()> { Err(ErrorInfo { message: "Not supported".into() }) }
}
