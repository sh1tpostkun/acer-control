use std::path::Path;
use tokio::fs;
use async_trait::async_trait;
use acercontrol_core::*;
use crate::HardwareBackend;
use sysinfo::System;

pub struct LinuwuSenseBackend {
    sys: tokio::sync::Mutex<System>,
}

impl LinuwuSenseBackend {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys: tokio::sync::Mutex::new(sys),
        }
    }

    async fn read_sysfs_u32(path: &str) -> Result<u32> {
        let content = fs::read_to_string(path).await.map_err(|_| ErrorInfo { message: format!("Failed to read {}", path) })?;
        content.trim().parse().map_err(|_| ErrorInfo { message: format!("Failed to parse {}", path) })
    }

    async fn write_sysfs(path: &str, value: &str) -> Result<()> {
        if !Path::new(path).exists() {
            return Err(ErrorInfo { message: format!("Path {} does not exist", path) });
        }
        fs::write(path, value).await.map_err(|_| ErrorInfo { message: format!("Failed to write to {}", path) })
    }
    
    async fn check_capability(path: &str) -> bool {
        Path::new(path).exists()
    }

    fn find_acer_hwmon() -> String {
        for i in 0..10 {
            let path = format!("/sys/class/hwmon/hwmon{}", i);
            let name_path = format!("{}/name", path);
            if let Ok(name) = std::fs::read_to_string(&name_path) {
                if name.trim() == "acer" {
                    return path;
                }
            }
        }
        "/sys/class/hwmon/hwmon0".to_string() // Fallback
    }

    fn get_nitro_sense_path() -> String {
        let path = "/sys/module/linuwu_sense/drivers/platform:acer-wmi/acer-wmi/nitro_sense";
        if Path::new(path).exists() {
            path.to_string()
        } else {
            "/sys/module/linuwu_sense/drivers/platform:acer-wmi/acer-wmi/predator_sense".to_string()
        }
    }
}

#[async_trait]
impl HardwareBackend for LinuwuSenseBackend {
    async fn detect(&self) -> Result<bool> {
        Ok(Path::new("/sys/module/linuwu_sense").exists())
    }

    fn name(&self) -> &'static str {
        "Linuwu-Sense Backend"
    }

    async fn capabilities(&self) -> Result<HardwareCapabilities> {
        let ns_path = Self::get_nitro_sense_path();
        Ok(HardwareCapabilities {
            fan_control: Self::check_capability(&format!("{}/fan_speed", ns_path)).await,
            thermal_profile: Self::check_capability("/sys/firmware/acpi/platform_profile").await,
            battery_limit: Self::check_capability(&format!("{}/battery_limiter", ns_path)).await,
            usb_charging: Self::check_capability(&format!("{}/usb_charging", ns_path)).await,
            keyboard_rgb: false, // basic backlight
            battery_calibration: Self::check_capability(&format!("{}/battery_calibration", ns_path)).await,
        })
    }

    async fn get_fan_status(&self) -> Result<FanStatus> {
        let hwmon = Self::find_acer_hwmon();
        let ns_path = Self::get_nitro_sense_path();

        let cpu_rpm = Self::read_sysfs_u32(&format!("{}/fan1_input", hwmon)).await.unwrap_or(0);
        let gpu_rpm = Self::read_sysfs_u32(&format!("{}/fan2_input", hwmon)).await.unwrap_or(0);
        
        // Read fan_speed. If it is 0,0, then mode is Auto. Otherwise Manual.
        let fan_speed_str = fs::read_to_string(format!("{}/fan_speed", ns_path)).await.unwrap_or_else(|_| "0,0".into());
        let parts: Vec<&str> = fan_speed_str.trim().split(',').collect();
        
        let mut cpu_percent = 0;
        let mut gpu_percent = 0;
        
        if parts.len() == 2 {
            cpu_percent = parts[0].parse().unwrap_or(0);
            gpu_percent = parts[1].parse().unwrap_or(0);
        }

        let mode = if cpu_percent == 0 && gpu_percent == 0 {
            FanMode::Auto
        } else {
            FanMode::Manual
        };
        
        Ok(FanStatus {
            mode,
            cpu_rpm,
            gpu_rpm,
            cpu_speed_percent: cpu_percent,
            gpu_speed_percent: gpu_percent,
        })
    }

    async fn set_fan_mode(&self, mode: FanMode) -> Result<()> {
        let ns_path = Self::get_nitro_sense_path();
        match mode {
            FanMode::Auto => {
                Self::write_sysfs(&format!("{}/fan_speed", ns_path), "0,0").await?;
            },
            FanMode::Manual | FanMode::CustomCurve => {
                // To switch to manual, we just set it to some initial manual speed or leave it to set_fan_speed
                // Let's set 50,50 as default when switching to manual if we don't have current percentages
                Self::write_sysfs(&format!("{}/fan_speed", ns_path), "50,50").await?;
            }
        };
        Ok(())
    }

    async fn set_fan_speed(&self, cpu_percent: u8, gpu_percent: u8) -> Result<()> {
        let ns_path = Self::get_nitro_sense_path();
        if cpu_percent > 100 || gpu_percent > 100 {
            return Err(ErrorInfo { message: "Percentage out of bounds".into() });
        }
        
        Self::write_sysfs(&format!("{}/fan_speed", ns_path), &format!("{},{}", cpu_percent, gpu_percent)).await?;
        Ok(())
    }

    async fn get_thermal_profile(&self) -> Result<ThermalProfile> {
        let profile_str = fs::read_to_string("/sys/firmware/acpi/platform_profile")
            .await
            .map_err(|_| ErrorInfo { message: "Could not read platform_profile".into() })?;
        
        match profile_str.trim() {
            "quiet" => Ok(ThermalProfile::Silent),
            "balanced" => Ok(ThermalProfile::Balanced),
            "performance" | "balanced-performance" => Ok(ThermalProfile::Performance),
            _ => Ok(ThermalProfile::Balanced),
        }
    }

    async fn set_thermal_profile(&self, profile: ThermalProfile) -> Result<()> {
        let val = match profile {
            ThermalProfile::Silent => "quiet",
            ThermalProfile::Balanced => "balanced",
            ThermalProfile::Performance | ThermalProfile::Turbo => "balanced-performance",
            ThermalProfile::Custom => return Err(ErrorInfo { message: "Custom profile not supported natively".into() }),
        };
        Self::write_sysfs("/sys/firmware/acpi/platform_profile", val).await
    }

    async fn get_battery_status(&self) -> Result<BatteryStatus> {
        let ns_path = Self::get_nitro_sense_path();
        let percentage = Self::read_sysfs_u32("/sys/class/power_supply/BAT1/capacity").await.unwrap_or(0) as u8;
        let status = fs::read_to_string("/sys/class/power_supply/BAT1/status")
            .await.unwrap_or_else(|_| "Unknown".into());
        let is_charging = status.trim() == "Charging";
        let ac_connected = Self::read_sysfs_u32("/sys/class/power_supply/ACAD/online").await.unwrap_or(0) == 1;
        
        let charge_limit = Self::read_sysfs_u32(&format!("{}/battery_limiter", ns_path)).await.unwrap_or(100) as u8;
        
        let health = Self::read_sysfs_u32("/sys/class/power_supply/BAT1/capacity_level").await.unwrap_or(100) as u8; // Pseudo health
        let cycle_count = Self::read_sysfs_u32("/sys/class/power_supply/BAT1/cycle_count").await.unwrap_or(0);

        Ok(BatteryStatus {
            percentage,
            is_charging,
            ac_connected,
            health,
            cycle_count,
            charge_limit,
            usb_charging_enabled: Self::read_sysfs_u32(&format!("{}/usb_charging", ns_path)).await.unwrap_or(0) == 1,
        })
    }

    async fn set_battery_limit(&self, limit_percentage: u8) -> Result<()> {
        let ns_path = Self::get_nitro_sense_path();
        if limit_percentage > 100 || limit_percentage < 40 {
             return Err(ErrorInfo { message: "Limit must be between 40 and 100".into() });
        }
        Self::write_sysfs(&format!("{}/battery_limiter", ns_path), &limit_percentage.to_string()).await
    }

    async fn set_usb_charging(&self, enabled: bool) -> Result<()> {
        let ns_path = Self::get_nitro_sense_path();
        Self::write_sysfs(&format!("{}/usb_charging", ns_path), if enabled { "1" } else { "0" }).await
    }

    async fn get_temperatures(&self) -> Result<TemperatureStatus> {
        let hwmon = Self::find_acer_hwmon();
        // Typically temp1_input is in millidegrees Celsius
        let cpu_milli = Self::read_sysfs_u32(&format!("{}/temp1_input", hwmon)).await.unwrap_or(0);
        let gpu_milli = Self::read_sysfs_u32(&format!("{}/temp2_input", hwmon)).await.unwrap_or(0);
        Ok(TemperatureStatus {
            cpu_temp_c: (cpu_milli / 1000) as u8,
            gpu_temp_c: (gpu_milli / 1000) as u8,
        })
    }

    async fn get_power(&self) -> Result<PowerStatus> {
        let power_microwatts = Self::read_sysfs_u32("/sys/class/power_supply/BAT1/power_now").await.unwrap_or(0);
        Ok(PowerStatus {
            power_consumption_w: power_microwatts as f32 / 1_000_000.0,
        })
    }

    async fn get_hardware_info(&self) -> Result<SystemInfo> {
        eprintln!("get_hardware_info start"); let mut sys = self.sys.lock().await;
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut total_disk = 0;
        let mut used_disk = 0;
        let mut disk_name = "Диск".to_string();
        for disk in &disks {
            let path_str = disk.mount_point().to_string_lossy();
            if path_str == "/" || path_str == "C:\\" {
                total_disk = disk.total_space();
                used_disk = disk.total_space() - disk.available_space();
                disk_name = disk.name().to_string_lossy().to_string();
                if disk_name.is_empty() {
                    disk_name = "Root".to_string();
                }
                eprintln!("Found ROOT partition: {} GB", total_disk / 1024/1024/1024);
                break;
            }
        }
        
        // Fallback
        if total_disk == 0 && !disks.is_empty() {
            // Find the largest partition to avoid tmpfs
            let mut largest_disk = disks.first().unwrap();
            for disk in &disks {
                if disk.total_space() > largest_disk.total_space() {
                    largest_disk = disk;
                }
            }
            total_disk = largest_disk.total_space();
            used_disk = largest_disk.total_space() - largest_disk.available_space();
            disk_name = largest_disk.name().to_string_lossy().to_string();
            eprintln!("Fallback to LARGEST partition: {} GB", total_disk / 1024/1024/1024);
        }
        
        let mut cpu_name = "Unknown CPU".to_string();
        if let Some(global_cpu) = sys.cpus().first() {
            cpu_name = global_cpu.brand().to_string();
            // clean up name
            cpu_name = cpu_name.replace("(R)", "").replace("(TM)", "").replace(" CPU", "").trim().to_string();
        }
        let cpu_usage = sys.global_cpu_usage();
        
        let ram_total = sys.total_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
        let ram_used = sys.used_memory() as f32 / 1024.0 / 1024.0 / 1024.0;
        let disk_total_gb = total_disk as f32 / 1024.0 / 1024.0 / 1024.0;
        let disk_used_gb = used_disk as f32 / 1024.0 / 1024.0 / 1024.0;

        let mut gpu_name = "Unknown GPU".to_string();
        if let Ok(output) = std::process::Command::new("lspci").output() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            for line in out_str.lines() {
                if (line.contains("VGA") || line.contains("3D")) && (line.contains("NVIDIA") || line.contains("AMD")) {
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line[start..].find(']') {
                            gpu_name = line[start + 1..start + end].to_string();
                            break;
                        }
                    }
                }
            }
        }
        if gpu_name == "Unknown GPU" {
            gpu_name = "NVIDIA GeForce RTX 4060M".into(); // Fallback
        }

        let mut laptop_model = tokio::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").await.unwrap_or_else(|_| "Unknown Laptop".into());
        laptop_model = laptop_model.trim().to_string();

        Ok(SystemInfo {
            device_name: cpu_name,
            laptop_model,
            gpu_name,
            disk_name,
            cpu_usage: cpu_usage as f32,
            gpu_usage: 5.0, // Mocked for now, linux gpu usage requires nvml
            ram_total,
            ram_used,
            vram_total: 8.0,
            vram_used: 1.0,
            disk_total: disk_total_gb,
            disk_used: disk_used_gb,
            cpu_freq_mhz: sys.cpus().first().map(|c| c.frequency()).unwrap_or(0) as u32,
            gpu_freq_mhz: 800,
        })
    }
}
