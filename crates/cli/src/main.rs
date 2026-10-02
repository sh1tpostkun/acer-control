use clap::{Parser, Subcommand};
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use acercontrol_ipc::{IpcRequest, IpcResponse};
use acercontrol_core::{FanMode, ThermalProfile};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    socket: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Status,
    Doctor,
    Capabilities,
    Fan {
        #[command(subcommand)]
        action: FanAction,
    },
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    Battery {
        #[command(subcommand)]
        action: BatteryAction,
    },
}

#[derive(Subcommand, Debug)]
enum FanAction {
    Status,
    Auto,
    Set { cpu: u8, gpu: u8 },
}

#[derive(Subcommand, Debug)]
enum ProfileAction {
    Get,
    Set { profile: String },
}

#[derive(Subcommand, Debug)]
enum BatteryAction {
    Status,
    Limit { percent: u8 },
}

async fn send_request(req: IpcRequest, socket_path: &str) -> std::result::Result<IpcResponse, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let json = serde_json::to_string(&req)?;
    stream.write_all(format!("{}\n", json).as_bytes()).await?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let response_str = String::from_utf8_lossy(&buf[..n]);
    
    let res: IpcResponse = serde_json::from_str(response_str.trim())?;
    Ok(res)
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    let socket_path = cli.socket.unwrap_or_else(|| {
        // Fallback to mock for testing if it exists, else default
        let mock = "/tmp/acercontrol_mock.sock";
        if std::path::Path::new(mock).exists() {
            mock.to_string()
        } else {
            acercontrol_ipc::SOCKET_PATH.to_string()
        }
    });

    match cli.command {
        Commands::Status => {
            let res = send_request(IpcRequest::GetSystemInfo, &socket_path).await?;
            println!("{:#?}", res);
        }
        Commands::Doctor => {
            println!("Running diagnostics...");
            match send_request(IpcRequest::Ping, &socket_path).await {
                Ok(IpcResponse::Pong) => println!("✓ Daemon is running and reachable"),
                Ok(other) => println!("✗ Daemon returned unexpected response: {:?}", other),
                Err(e) => println!("✗ Cannot connect to daemon: {}", e),
            }
            if let Ok(IpcResponse::Capabilities(caps)) = send_request(IpcRequest::GetCapabilities, &socket_path).await {
                println!("✓ Hardware capabilities detected:");
                println!("  - Fan Control: {}", if caps.fan_control { "Yes" } else { "No" });
                println!("  - Thermal Profile: {}", if caps.thermal_profile { "Yes" } else { "No" });
                println!("  - Battery Limit: {}", if caps.battery_limit { "Yes" } else { "No" });
                println!("  - USB Charging: {}", if caps.usb_charging { "Yes" } else { "No" });
            }
        }
        Commands::Capabilities => {
            let res = send_request(IpcRequest::GetCapabilities, &socket_path).await?;
            println!("{:#?}", res);
        }
        Commands::Fan { action } => match action {
            FanAction::Status => {
                let res = send_request(IpcRequest::GetFanStatus, &socket_path).await?;
                println!("{:#?}", res);
            }
            FanAction::Auto => {
                let res = send_request(IpcRequest::SetFanMode(FanMode::Auto), &socket_path).await?;
                println!("{:?}", res);
            }
            FanAction::Set { cpu, gpu } => {
                send_request(IpcRequest::SetFanMode(FanMode::Manual), &socket_path).await?;
                let res = send_request(IpcRequest::SetFanSpeed { cpu_percent: cpu, gpu_percent: gpu }, &socket_path).await?;
                println!("{:?}", res);
            }
        },
        Commands::Profile { action } => match action {
            ProfileAction::Get => {
                let res = send_request(IpcRequest::GetThermalProfile, &socket_path).await?;
                println!("{:#?}", res);
            }
            ProfileAction::Set { profile } => {
                let p = match profile.to_lowercase().as_str() {
                    "silent" => ThermalProfile::Silent,
                    "balanced" => ThermalProfile::Balanced,
                    "performance" => ThermalProfile::Performance,
                    "turbo" => ThermalProfile::Turbo,
                    _ => {
                        println!("Invalid profile. Valid options: silent, balanced, performance, turbo");
                        return Ok(());
                    }
                };
                let res = send_request(IpcRequest::SetThermalProfile(p), &socket_path).await?;
                println!("{:?}", res);
            }
        },
        Commands::Battery { action } => match action {
            BatteryAction::Status => {
                let res = send_request(IpcRequest::GetBatteryStatus, &socket_path).await?;
                println!("{:#?}", res);
            }
            BatteryAction::Limit { percent } => {
                let res = send_request(IpcRequest::SetBatteryLimit(percent), &socket_path).await?;
                println!("{:?}", res);
            }
        },
    }

    Ok(())
}
