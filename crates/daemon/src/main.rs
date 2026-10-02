use log::{error, info};
use std::fs;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use std::time::Duration;
use clap::Parser;

use acercontrol_ipc::{IpcEvent, IpcRequest, IpcResponse};
use acercontrol_backends::{BackendManager, hwmon, acer_wmi, linuwu_sense, nvml, sysfs};
use acercontrol_core::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    socket: Option<String>,
}

struct AppState {
    manager: tokio::sync::RwLock<BackendManager>,
    event_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let args = Args::parse();

    info!("Starting AcerControl Daemon...");
    
    let sys_id = SystemIdentification::probe();
    info!("Detected System: {} {} (Board: {})", sys_id.vendor, sys_id.product, sys_id.board);

    let mut manager = BackendManager::new();
    manager.add_backend(Box::new(acer_wmi::Backend));
    manager.add_backend(Box::new(linuwu_sense::Backend));
    manager.add_backend(Box::new(hwmon::Backend));
    manager.add_backend(Box::new(nvml::Backend));
    manager.add_backend(Box::new(sysfs::Backend));

    manager.initialize().await;
    info!("Active backends: {:?}", manager.active_backends());

    let socket_path = args.socket.unwrap_or_else(|| {
        acercontrol_ipc::SOCKET_PATH.to_string()
    });

    if let Some(parent) = std::path::Path::new(&socket_path).parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }

    if fs::metadata(&socket_path).is_ok() {
        fs::remove_file(&socket_path).unwrap();
    }

    let listener = UnixListener::bind(&socket_path).unwrap();
    let _ = fs::set_permissions(&socket_path, std::os::unix::fs::PermissionsExt::from_mode(0o660));
    let _ = std::process::Command::new("chgrp").arg("acercontrol").arg(&socket_path).status();

    info!("Listening on {}", socket_path);

    let (event_tx, _) = broadcast::channel(16);
    let state = Arc::new(AppState {
        manager: tokio::sync::RwLock::new(manager),
        event_tx: event_tx.clone(),
    });

    let telemetry_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            if telemetry_state.event_tx.receiver_count() > 0 {
                let manager = telemetry_state.manager.read().await;
                // Currently returning mock telemetry
                let telemetry = manager.get_telemetry().await.unwrap_or_default();
                let event = IpcEvent::TelemetryUpdate(telemetry);
                if let Ok(json) = serde_json::to_string(&event) {
                    let _ = telemetry_state.event_tx.send(json);
                }
            }
        }
    });

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, state_clone).await {
                        error!("Client error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept failed: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, state: Arc<AppState>) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0u8; 4096];
    let mut rx = state.event_tx.subscribe();
    let mut is_subscribed = false;

    loop {
        tokio::select! {
            result = stream.read(&mut buf) => {
                let n = result?;
                if n == 0 { break; }
                let messages = String::from_utf8_lossy(&buf[..n]);
                for msg in messages.lines() {
                    if msg.trim().is_empty() { continue; }
                    match serde_json::from_str::<IpcRequest>(msg) {
                        Ok(req) => {
                            let res = process_request(&req, &state, &mut is_subscribed).await;
                            let res_json = serde_json::to_string(&res)?;
                            stream.write_all(format!("{}\n", res_json).as_bytes()).await?;
                        }
                        Err(e) => {
                            let res_json = serde_json::to_string(&IpcResponse::Error(e.to_string()))?;
                            stream.write_all(format!("{}\n", res_json).as_bytes()).await?;
                        }
                    }
                }
            }
            event_json = rx.recv(), if is_subscribed => {
                if let Ok(json) = event_json {
                    if stream.write_all(format!("{}\n", json).as_bytes()).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn process_request(req: &IpcRequest, state: &AppState, is_subscribed: &mut bool) -> IpcResponse {
    let manager = state.manager.read().await;
    match req {
        IpcRequest::Ping => IpcResponse::Pong,
        IpcRequest::Doctor => {
            let sys = SystemIdentification::probe();
            let mut report = format!("AcerControl Hardware Diagnostic\n");
            report.push_str("────────────────────────────────\n\n");
            report.push_str("SYSTEM\n");
            report.push_str(&format!("Vendor:       {}\n", sys.vendor));
            report.push_str(&format!("Model:        {}\n", sys.product));
            report.push_str(&format!("Version:      {}\n", sys.version));
            report.push_str(&format!("Board:        {}\n\n", sys.board));
            report.push_str("BACKENDS\n");
            for b in manager.active_backends() {
                report.push_str(&format!("{:14} active\n", b));
            }
            report.push_str("\nCAPABILITIES\n");
            let caps = manager.get_capabilities().await;
            report.push_str(&format!("Fan telemetry:      {}\n", if caps.fan_telemetry { "✓" } else { "—" }));
            report.push_str(&format!("Fan control:        {}\n", if caps.fan_control { "✓" } else { "—" }));
            report.push_str(&format!("Thermal profiles:   {}\n", if caps.thermal_profile { "✓" } else { "—" }));
            report.push_str(&format!("Battery limit:      {}\n", if caps.battery_limit { "✓" } else { "—" }));
            report.push_str(&format!("Keyboard backlight: {}\n", if caps.keyboard_backlight { "✓" } else { "—" }));
            report.push_str(&format!("GPU mode:           {}\n", if caps.gpu_mode { "✓" } else { "—" }));
            report.push_str(&format!("CPU power limit:    {}\n", if caps.cpu_power_limit { "✓" } else { "—" }));
            
            report.push_str("\nREASONS\n");
            if !caps.keyboard_backlight {
                report.push_str("RGB                  Driver missing or unsupported\n");
            }
            if !caps.gpu_mode {
                report.push_str("GPU mode             Unsupported by detected firmware\n");
            }
            
            IpcResponse::DoctorReport(report)
        }
        IpcRequest::GetCapabilities => {
            IpcResponse::Capabilities(manager.get_capabilities().await)
        }
        IpcRequest::GetTelemetry => IpcResponse::Telemetry(manager.get_telemetry().await.unwrap_or_default()),
        IpcRequest::GetSystemIdentification => IpcResponse::SystemIdentification(SystemIdentification::probe()),
        IpcRequest::GetThermalProfile => IpcResponse::ThermalProfile(ThermalProfile::Balanced),
        IpcRequest::SetThermalProfile { profile } => {
            let _ = manager.set_performance_mode(profile.clone()).await;
            IpcResponse::Ok
        },
        IpcRequest::SetFanMode { mode } => {
            let _ = manager.set_fan_mode(mode.clone()).await;
            IpcResponse::Ok
        },
        IpcRequest::SetFanSpeed { cpu_percent: _, gpu_percent: _ } => {
            IpcResponse::Ok
        },
        IpcRequest::SetBatteryLimit { limit } => {
            let _ = manager.set_battery_limit(*limit).await;
            IpcResponse::Ok
        },
        IpcRequest::SetKeyboardTimeout { timeout_s: _ } => {
            // manager.set_keyboard_timeout(*timeout_s).await;
            IpcResponse::Ok
        },
        IpcRequest::SetWifiEnabled { enabled } => {
            let cmd = if *enabled { "unblock" } else { "block" };
            let _ = std::process::Command::new("rfkill").arg(cmd).arg("wifi").spawn();
            IpcResponse::Ok
        },
        IpcRequest::SetBluetoothEnabled { enabled } => {
            let cmd = if *enabled { "unblock" } else { "block" };
            let _ = std::process::Command::new("rfkill").arg(cmd).arg("bluetooth").spawn();
            IpcResponse::Ok
        },
        IpcRequest::DropCaches => {
            let _ = std::process::Command::new("sh").arg("-c").arg("sync; echo 3 > /proc/sys/vm/drop_caches").spawn();
            IpcResponse::Ok
        },
        IpcRequest::SubscribeEvents => {
            *is_subscribed = true;
            IpcResponse::Ok
        }
    }
}
