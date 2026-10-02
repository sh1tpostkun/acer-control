use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use acercontrol_core::*;
use crate::HardwareBackend;

pub struct MockBackend {
    fan_mode: Arc<Mutex<FanMode>>,
    thermal_profile: Arc<Mutex<ThermalProfile>>,
    battery_limit: Arc<Mutex<u8>>,
    usb_charging: Arc<Mutex<bool>>,
    cpu_fan_percent: Arc<Mutex<u8>>,
    gpu_fan_percent: Arc<Mutex<u8>>,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            fan_mode: Arc::new(Mutex::new(FanMode::Auto)),
            thermal_profile: Arc::new(Mutex::new(ThermalProfile::Balanced)),
            battery_limit: Arc::new(Mutex::new(80)),
            usb_charging: Arc::new(Mutex::new(true)),
            cpu_fan_percent: Arc::new(Mutex::new(50)),
            gpu_fan_percent: Arc::new(Mutex::new(50)),
        }
    }
}

#[async_trait]
impl HardwareBackend for MockBackend {
    async fn detect(&self) -> Result<bool> {
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "Mock Backend"
    }

    async fn capabilities(&self) -> Result<HardwareCapabilities> {
        Ok(HardwareCapabilities {
            fan_control: true,
            thermal_profile: true,
            battery_limit: true,
            usb_charging: true,
            keyboard_rgb: false,
            battery_calibration: true,
        })
    }

    async fn get_fan_status(&self) -> Result<FanStatus> {
        let mode = self.fan_mode.lock().await.clone();
        let cpu_p = *self.cpu_fan_percent.lock().await;
        let gpu_p = *self.gpu_fan_percent.lock().await;
        
        let cpu_rpm = 2000 + (cpu_p as u32 * 30);
        let gpu_rpm = 2200 + (gpu_p as u32 * 35);

        Ok(FanStatus {
            mode,
            cpu_rpm,
            gpu_rpm,
            cpu_speed_percent: cpu_p,
            gpu_speed_percent: gpu_p,
        })
    }

    async fn set_fan_mode(&self, mode: FanMode) -> Result<()> {
        *self.fan_mode.lock().await = mode;
        Ok(())
    }

    async fn set_fan_speed(&self, cpu_percent: u8, gpu_percent: u8) -> Result<()> {
        if *self.fan_mode.lock().await != FanMode::Manual {
            return Err(ErrorInfo { message: "Fan mode must be Manual to set speed".into() });
        }
        if cpu_percent > 100 || gpu_percent > 100 {
            return Err(ErrorInfo { message: "Invalid fan percent".into() });
        }
        *self.cpu_fan_percent.lock().await = cpu_percent;
        *self.gpu_fan_percent.lock().await = gpu_percent;
        Ok(())
    }

    async fn get_thermal_profile(&self) -> Result<ThermalProfile> {
        Ok(self.thermal_profile.lock().await.clone())
    }

    async fn set_thermal_profile(&self, profile: ThermalProfile) -> Result<()> {
        *self.thermal_profile.lock().await = profile;
        Ok(())
    }

    async fn get_battery_status(&self) -> Result<BatteryStatus> {
        Ok(BatteryStatus {
            percentage: 82,
            is_charging: false,
            ac_connected: true,
            health: 98,
            cycle_count: 42,
            charge_limit: *self.battery_limit.lock().await,
            usb_charging_enabled: *self.usb_charging.lock().await,
        })
    }

    async fn set_battery_limit(&self, limit_percentage: u8) -> Result<()> {
        if limit_percentage > 100 {
            return Err(ErrorInfo { message: "Limit must be <= 100".into() });
        }
        *self.battery_limit.lock().await = limit_percentage;
        Ok(())
    }

    async fn set_usb_charging(&self, enabled: bool) -> Result<()> {
        *self.usb_charging.lock().await = enabled;
        Ok(())
    }

    async fn get_temperatures(&self) -> Result<TemperatureStatus> {
        
        
        Ok(TemperatureStatus {
            cpu_temp_c: 45,
            gpu_temp_c: 40,
        })
    }

    async fn get_power(&self) -> Result<PowerStatus> {
        
        
        Ok(PowerStatus {
            power_consumption_w: 25.5,
        })
    }

    async fn get_hardware_info(&self) -> Result<SystemInfo> {
        
        
        Ok(SystemInfo {
            device_name: "Acer Nitro V 15 ANV15-51 (Mock)".into(),
            laptop_model: "Mock Laptop".into(),
            gpu_name: "Mock GPU".into(),
            disk_name: "Mock Disk".into(),
            cpu_usage: 15.0,
            gpu_usage: 2.0,
            ram_total: 32.0,
            ram_used: 9.5,
            vram_total: 8.0,
            vram_used: 1.2,
            disk_total: 1000.0,
            disk_used: 200.0,
            cpu_freq_mhz: 2400,
            gpu_freq_mhz: 800,
        })
    }
}
