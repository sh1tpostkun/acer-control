use log::{error, info, warn};
use std::fs;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use std::time::Duration;
use clap::Parser;

use acercontrol_ipc::{DoctorData, IpcEvent, IpcRequest, IpcResponse};
use acercontrol_backends::{BackendManager, hwmon, acer_wmi, linuwu_sense, nvml, sysfs};
use acercontrol_core::*;

#[derive(Parser, Debug)]
#[command(author, version, about = "AcerControl Daemon", long_about = None)]
struct Args {
    #[arg(short, long)]
    socket: Option<String>,
}

struct AppState {
    manager: tokio::sync::RwLock<BackendManager>,
    event_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let args = Args::parse();

    info!("Starting AcerControl Daemon...");

    let sys_id = SystemIdentification::probe();
    info!("System: {} {} (Board: {})", sys_id.vendor, sys_id.product, sys_id.board);

    // Register backends in priority order:
    // 1. NVML (dedicated NVIDIA GPU telemetry)
    // 2. linuwu-sense (fan control, profiles, battery limit, keyboard timeout)
    // 3. hwmon (sensors: CPU/GPU temps, fan RPM, battery status)
    // 4. acer-wmi & sysfs (fallback / future drivers)
    let mut manager = BackendManager::new();
    manager.add_backend(Box::new(nvml::Backend::new()));
    manager.add_backend(Box::new(linuwu_sense::Backend::new()));
    manager.add_backend(Box::new(hwmon::Backend::new()));
    manager.add_backend(Box::new(acer_wmi::Backend));
    manager.add_backend(Box::new(sysfs::Backend));

    manager.initialize().await;

    let socket_path = args.socket.unwrap_or_else(|| {
        acercontrol_ipc::SOCKET_PATH.to_string()
    });

    if let Some(parent) = std::path::Path::new(&socket_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if fs::metadata(&socket_path).is_ok() {
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    let _ = fs::set_permissions(
        &socket_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o660),
    );
    let _ = std::process::Command::new("chgrp")
        .arg("acercontrol")
        .arg(&socket_path)
        .stderr(std::process::Stdio::null())
        .status();

    info!("Listening on {}", socket_path);

    let (event_tx, _) = broadcast::channel(16);
    let state = Arc::new(AppState {
        manager: tokio::sync::RwLock::new(manager),
        event_tx: event_tx.clone(),
    });

    // Telemetry broadcast loop
    let telemetry_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            if telemetry_state.event_tx.receiver_count() > 0 {
                let manager = telemetry_state.manager.read().await;
                match manager.get_telemetry().await {
                    Ok(telemetry) => {
                        let event = IpcEvent::TelemetryUpdate(telemetry);
                        if let Ok(json) = serde_json::to_string(&event) {
                            let _ = telemetry_state.event_tx.send(json);
                        }
                    }
                    Err(e) => {
                        warn!("Telemetry collection error: {}", e);
                    }
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

async fn handle_client(
    mut stream: UnixStream,
    state: Arc<AppState>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0u8; 8192];
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

async fn process_request(
    req: &IpcRequest,
    state: &AppState,
    is_subscribed: &mut bool,
) -> IpcResponse {
    let manager = state.manager.read().await;
    match req {
        IpcRequest::Ping => IpcResponse::Pong,

        IpcRequest::Doctor => {
            let sys = SystemIdentification::probe();
            let caps = manager.get_capabilities().await;
            let tel = manager.get_telemetry().await.unwrap_or_default();
            let kernel = std::fs::read_to_string("/proc/version")
                .unwrap_or_default()
                .split_whitespace()
                .nth(2)
                .unwrap_or("unknown")
                .to_string();

            let mut r = String::new();
            r.push_str("AcerControl Hardware Diagnostic\n");
            r.push_str("────────────────────────────────\n\n");

            r.push_str("SYSTEM\n");
            r.push_str(&format!("  Vendor:             {}\n", sys.vendor));
            r.push_str(&format!("  Model:              {}\n", sys.product));
            r.push_str(&format!("  Board:              {}\n", sys.board));
            r.push_str(&format!("  Kernel:             {}\n\n", kernel));

            r.push_str("BACKENDS\n");
            for b in manager.active_backends() {
                r.push_str(&format!("  {:20} active\n", b));
            }

            r.push_str("\nCAPABILITIES\n");
            let fmt_cap = |s: &CapabilityStatus| -> String {
                match s {
                    CapabilityStatus::Supported { backend, source } => {
                        if let Some(src) = source {
                            format!("✓  ({} -> {})", backend, src)
                        } else {
                            format!("✓  ({})", backend)
                        }
                    }
                    CapabilityStatus::Unsupported => "—".to_string(),
                    CapabilityStatus::Unavailable => "unavailable".to_string(),
                    CapabilityStatus::PermissionDenied => "permission denied".to_string(),
                    CapabilityStatus::DriverMissing => "driver missing".to_string(),
                }
            };
            r.push_str(&format!("  Fan telemetry:      {}\n", fmt_cap(&caps.fan_telemetry)));
            r.push_str(&format!("  GPU telemetry:      {}\n", fmt_cap(&caps.gpu_telemetry)));
            r.push_str(&format!("  Fan control:        {}\n", fmt_cap(&caps.fan_control)));
            r.push_str(&format!("  Thermal profiles:   {}\n", fmt_cap(&caps.thermal_profile)));
            r.push_str(&format!("  Battery limit:      {}\n", fmt_cap(&caps.battery_limit)));
            r.push_str(&format!("  Keyboard backlight: {}\n", fmt_cap(&caps.keyboard_backlight)));
            r.push_str(&format!("  GPU mode:           {}\n", fmt_cap(&caps.gpu_mode)));
            r.push_str(&format!("  CPU power limit:    {}\n", fmt_cap(&caps.cpu_power_limit)));
            r.push_str(&format!("  GPU power limit:    {}\n", fmt_cap(&caps.gpu_power_limit)));

            let doctor_data = DoctorData {
                system: sys,
                kernel,
                active_backends: manager.active_backends().to_vec(),
                capabilities: caps,
                telemetry: tel,
            };

            IpcResponse::DoctorReport {
                text: r,
                data: doctor_data,
            }
        }

        IpcRequest::GetCapabilities => {
            IpcResponse::Capabilities(manager.get_capabilities().await)
        }

        IpcRequest::GetTelemetry => {
            match manager.get_telemetry().await {
                Ok(t) => IpcResponse::Telemetry(t),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }

        IpcRequest::GetSystemIdentification => {
            IpcResponse::SystemIdentification(SystemIdentification::probe())
        }

        IpcRequest::GetThermalProfile => {
            match manager.get_thermal_profile().await {
                Ok(p) => IpcResponse::ThermalProfile(p),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }

        IpcRequest::SetThermalProfile { profile } => {
            match manager.set_performance_mode(profile.clone()).await {
                Ok(()) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }

        IpcRequest::SetFanMode { mode } => {
            match manager.set_fan_mode(mode.clone()).await {
                Ok(()) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }

        IpcRequest::SetFanSpeed { cpu_percent, gpu_percent } => {
            match manager.set_fan_speed(*cpu_percent, *gpu_percent).await {
                Ok(()) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }

        IpcRequest::SetBatteryLimit { limit } => {
            match manager.set_battery_limit(*limit).await {
                Ok(()) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }

        IpcRequest::SetKeyboardTimeout { timeout_s } => {
            match manager.set_keyboard_timeout(*timeout_s).await {
                Ok(()) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }

        IpcRequest::SetWifiEnabled { enabled } => {
            let cmd = if *enabled { "unblock" } else { "block" };
            match std::process::Command::new("rfkill").arg(cmd).arg("wifi").status() {
                Ok(s) if s.success() => IpcResponse::Ok,
                Ok(s) => IpcResponse::Error(format!("rfkill exited with {}", s)),
                Err(e) => IpcResponse::Error(format!("rfkill: {}", e)),
            }
        }

        IpcRequest::SetBluetoothEnabled { enabled } => {
            let cmd = if *enabled { "unblock" } else { "block" };
            match std::process::Command::new("rfkill").arg(cmd).arg("bluetooth").status() {
                Ok(s) if s.success() => IpcResponse::Ok,
                Ok(s) => IpcResponse::Error(format!("rfkill exited with {}", s)),
                Err(e) => IpcResponse::Error(format!("rfkill: {}", e)),
            }
        }

        IpcRequest::DropCaches => {
            match std::process::Command::new("sh")
                .arg("-c")
                .arg("sync; echo 3 > /proc/sys/vm/drop_caches")
                .status()
            {
                Ok(s) if s.success() => IpcResponse::Ok,
                Ok(s) => IpcResponse::Error(format!("drop_caches exited with {}", s)),
                Err(e) => IpcResponse::Error(format!("drop_caches: {}", e)),
            }
        }

        IpcRequest::SubscribeEvents => {
            *is_subscribed = true;
            IpcResponse::Ok
        }
    }
}
