use clap::{Parser, Subcommand};
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use acercontrol_ipc::{IpcRequest, IpcResponse};
use acercontrol_core::{FanMode, ThermalProfile};

#[derive(Parser, Debug)]
#[command(author, version, about = "AcerControl CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    socket: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Status,
    Doctor {
        #[arg(long, help = "Output raw diagnostic data in JSON format")]
        json: bool,
        #[arg(long, help = "Generate anonymized hardware compatibility report")]
        share: bool,
    },
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
    Max,
    Set {
        #[arg(short, long, default_value_t = 100)]
        cpu: u8,
        #[arg(short, long, default_value_t = 100)]
        gpu: u8,
    },
}

#[derive(Subcommand, Debug)]
enum ProfileAction {
    Set { profile: String },
}

#[derive(Subcommand, Debug)]
enum BatteryAction {
    #[command(about = "Set battery charge limit (80 for on, 100 for off)")]
    Limit { percent: u8 },
}

async fn send_request(req: IpcRequest, socket_path: &str) -> std::result::Result<IpcResponse, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path).await?;
    let json = serde_json::to_string(&req)?;
    stream.write_all(format!("{}\n", json).as_bytes()).await?;

    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await?;
    let response_str = String::from_utf8_lossy(&buf[..n]);
    
    let res: IpcResponse = serde_json::from_str(response_str.trim())?;
    Ok(res)
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let socket_path = cli.socket.unwrap_or_else(|| acercontrol_ipc::SOCKET_PATH.to_string());

    match cli.command {
        Commands::Status => {
            let res = send_request(IpcRequest::GetTelemetry, &socket_path).await?;
            println!("{:#?}", res);
        }
        Commands::Doctor { json, share } => {
            let res = send_request(IpcRequest::Doctor, &socket_path).await?;
            match res {
                IpcResponse::DoctorReport { text, data } => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&data)?);
                    } else if share {
                        println!("AcerControl Anonymized Diagnostic Report");
                        println!("========================================");
                        println!("Model:    {} {}", data.system.vendor, data.system.product);
                        println!("Board:    {}", data.system.board);
                        println!("Kernel:   {}", data.kernel);
                        println!("Backends: {:?}", data.active_backends);
                        println!("\nCapabilities JSON:\n{}", serde_json::to_string_pretty(&data.capabilities)?);
                        println!("\n(You can copy and submit this report to: https://github.com/sh1tpostkun/acer-control/issues)");
                    } else {
                        println!("{}", text);
                    }
                }
                IpcResponse::Error(e) => eprintln!("Daemon error: {}", e),
                other => eprintln!("Unexpected daemon response: {:?}", other),
            }
        }
        Commands::Capabilities => {
            let res = send_request(IpcRequest::GetCapabilities, &socket_path).await?;
            println!("{:#?}", res);
        }
        Commands::Fan { action } => match action {
            FanAction::Status => {
                let res = send_request(IpcRequest::GetTelemetry, &socket_path).await?;
                println!("{:#?}", res);
            }
            FanAction::Auto => {
                let res = send_request(IpcRequest::SetFanMode { mode: FanMode::Auto }, &socket_path).await?;
                println!("{:?}", res);
            }
            FanAction::Max => {
                let res = send_request(IpcRequest::SetFanSpeed { cpu_percent: 100, gpu_percent: 100 }, &socket_path).await?;
                println!("Fans set to maximum (100%): {:?}", res);
            }
            FanAction::Set { cpu, gpu } => {
                let res = send_request(IpcRequest::SetFanSpeed { cpu_percent: cpu, gpu_percent: gpu }, &socket_path).await?;
                println!("Fans set to CPU: {}%, GPU: {}%: {:?}", cpu, gpu, res);
            }
        },
        Commands::Profile { action } => match action {
            ProfileAction::Set { profile } => {
                let p = match profile.to_lowercase().as_str() {
                    "silent" | "quiet" => ThermalProfile::Silent,
                    "balanced" => ThermalProfile::Balanced,
                    "performance" => ThermalProfile::Performance,
                    "turbo" => ThermalProfile::Turbo,
                    _ => {
                        eprintln!("Invalid profile. Supported: silent, balanced, performance, turbo");
                        return Ok(());
                    }
                };
                let res = send_request(IpcRequest::SetThermalProfile { profile: p }, &socket_path).await?;
                println!("{:?}", res);
            }
        },
        Commands::Battery { action } => match action {
            BatteryAction::Limit { percent } => {
                let res = send_request(IpcRequest::SetBatteryLimit { limit: percent }, &socket_path).await?;
                println!("{:?}", res);
            }
        },
    }

    Ok(())
}
