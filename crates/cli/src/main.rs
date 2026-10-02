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
    Doctor {
        #[arg(long)]
        json: bool,
        #[arg(long)]
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
}

#[derive(Subcommand, Debug)]
enum ProfileAction {
    Set { profile: String },
}

#[derive(Subcommand, Debug)]
enum BatteryAction {
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
    let socket_path = cli.socket.unwrap_or_else(|| acercontrol_ipc::SOCKET_PATH.to_string());

    match cli.command {
        Commands::Status => {
            let res = send_request(IpcRequest::GetTelemetry, &socket_path).await?;
            println!("{:#?}", res);
        }
        Commands::Doctor { json, share } => {
            if share {
                println!("AcerControl diagnostic report would be shared here (Anonymized).");
                println!("Thank you for helping to improve compatibility!");
                return Ok(());
            }

            let res = send_request(IpcRequest::Doctor, &socket_path).await?;
            match res {
                IpcResponse::DoctorReport(report) => {
                    if json {
                        println!(r#"{{"diagnostic": "report"}}"#);
                    } else {
                        println!("{}", report);
                    }
                }
                _ => println!("Failed to get doctor report."),
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
                let res = send_request(IpcRequest::SetFanMode(FanMode::Auto), &socket_path).await?;
                println!("{:?}", res);
            }
        },
        Commands::Profile { action } => match action {
            ProfileAction::Set { profile } => {
                let p = match profile.to_lowercase().as_str() {
                    "silent" => ThermalProfile::Silent,
                    "balanced" => ThermalProfile::Balanced,
                    "performance" => ThermalProfile::Performance,
                    "turbo" => ThermalProfile::Turbo,
                    _ => {
                        println!("Invalid profile.");
                        return Ok(());
                    }
                };
                let res = send_request(IpcRequest::SetThermalProfile(p), &socket_path).await?;
                println!("{:?}", res);
            }
        },
        Commands::Battery { action } => match action {
            BatteryAction::Limit { percent } => {
                let res = send_request(IpcRequest::SetBatteryLimit(percent), &socket_path).await?;
                println!("{:?}", res);
            }
        },
    }

    Ok(())
}
