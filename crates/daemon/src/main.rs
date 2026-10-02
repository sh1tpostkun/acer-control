mod hotkey;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use clap::Parser;
use log::{info, warn, error};
use std::fs;

use acercontrol_hardware::{HardwareBackend, linuwu_sense::LinuwuSenseBackend, mock::MockBackend};
use acercontrol_ipc::{IpcRequest, IpcResponse, IpcEvent};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    mock: bool,

    #[arg(short, long)]
    socket: Option<String>,
}

struct AppState {
    backend: Arc<dyn HardwareBackend>,
    event_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let args = Args::parse();

    info!("Starting AcerControl Daemon...");

    let backend: Arc<dyn HardwareBackend> = if args.mock {
        info!("Using MockBackend");
        Arc::new(MockBackend::new())
    } else {
        let linuwu = LinuwuSenseBackend::new();
        if linuwu.detect().await.unwrap_or(false) {
            info!("Detected LinuwuSenseBackend");
            Arc::new(linuwu)
        } else {
            warn!("Linuwu-Sense not detected. Falling back to MockBackend.");
            Arc::new(MockBackend::new())
        }
    };

    let caps = backend.capabilities().await?;
    info!("Hardware capabilities: {:?}", caps);

    let socket_path = args.socket.unwrap_or_else(|| {
        if args.mock {
            format!("/tmp/acercontrol_mock.sock")
        } else {
            acercontrol_ipc::SOCKET_PATH.to_string()
        }
    });

    if fs::metadata(&socket_path).is_ok() {
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    if !args.mock {
        // In real setup, we might want to set permissions on the socket
        // so that users in a specific group can access it
        let _ = fs::set_permissions(&socket_path, std::os::unix::fs::PermissionsExt::from_mode(0o666));
    }

    info!("Listening on {}", socket_path);

    let (event_tx, _) = broadcast::channel(16);
    hotkey::start_hotkey_listener(event_tx.clone());

    let state = Arc::new(AppState {
        backend: backend.clone(),
        event_tx: event_tx.clone(),
    });

    // Start telemetry loop
    let telemetry_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            if telemetry_state.event_tx.receiver_count() > 0 {
                if let (Ok(temps), Ok(fans), Ok(power), Ok(battery), Ok(system)) = tokio::join!(
                    telemetry_state.backend.get_temperatures(),
                    telemetry_state.backend.get_fan_status(),
                    telemetry_state.backend.get_power(),
                    telemetry_state.backend.get_battery_status(),
                    telemetry_state.backend.get_hardware_info(),
                ) {
                    let event = IpcEvent::Telemetry {
                        temps,
                        fans,
                        power,
                        battery,
                        system,
                    };
                    if let Ok(json) = serde_json::to_string(&event) {
                        let _ = telemetry_state.event_tx.send(json);
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

async fn handle_client(mut stream: UnixStream, state: Arc<AppState>) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0u8; 4096];
    let mut rx = state.event_tx.subscribe();
    let mut is_subscribed = false;

    loop {
        tokio::select! {
            result = stream.read(&mut buf) => {
                let n = result?;
                if n == 0 {
                    break;
                }
                
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
                            error!("Failed to parse request: {}", e);
                            let err_res = IpcResponse::Error("Invalid JSON".into());
                            let res_json = serde_json::to_string(&err_res)?;
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
    match req {
        IpcRequest::Ping => IpcResponse::Pong,
        IpcRequest::GetCapabilities => {
            match state.backend.capabilities().await {
                Ok(caps) => IpcResponse::Capabilities(caps),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::GetFanStatus => {
            match state.backend.get_fan_status().await {
                Ok(status) => IpcResponse::FanStatus(status),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::SetFanMode(mode) => {
            match state.backend.set_fan_mode(mode.clone()).await {
                Ok(_) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::SetFanSpeed { cpu_percent, gpu_percent } => {
            match state.backend.set_fan_speed(*cpu_percent, *gpu_percent).await {
                Ok(_) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::GetThermalProfile => {
            match state.backend.get_thermal_profile().await {
                Ok(p) => IpcResponse::ThermalProfile(p),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::SetThermalProfile(profile) => {
            match state.backend.set_thermal_profile(profile.clone()).await {
                Ok(_) => {
                    // Send an event to update listeners if there are any
                    if let Ok(json) = serde_json::to_string(&IpcEvent::ProfileChanged(profile.clone())) {
                        let _ = state.event_tx.send(json);
                    }
                    IpcResponse::Ok
                },
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::GetBatteryStatus => {
            match state.backend.get_battery_status().await {
                Ok(b) => IpcResponse::BatteryStatus(b),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::SetBatteryLimit(limit) => {
            match state.backend.set_battery_limit(*limit).await {
                Ok(_) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::SetUsbCharging(enabled) => {
            match state.backend.set_usb_charging(*enabled).await {
                Ok(_) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::GetTemperatures => {
            match state.backend.get_temperatures().await {
                Ok(t) => IpcResponse::Temperatures(t),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::GetPower => {
            match state.backend.get_power().await {
                Ok(p) => IpcResponse::Power(p),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::GetSystemInfo => {
            match state.backend.get_hardware_info().await {
                Ok(i) => IpcResponse::SystemInfo(i),
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcRequest::SubscribeEvents => {
            *is_subscribed = true;
            IpcResponse::Ok
        }
    }
}
