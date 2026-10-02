use log::{info, warn};
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;
use acercontrol_ipc::IpcEvent;

#[allow(dead_code)]
#[repr(C)]
struct InputEvent {
    tv_sec: i64,
    tv_usec: i64,
    type_: u16,
    code: u16,
    value: i32,
}

pub fn start_hotkey_listener(event_tx: broadcast::Sender<String>) {
    thread::spawn(move || {
        loop {
            let mut found = false;
            for i in 0..32 {
                let name_path = format!("/sys/class/input/event{}/device/name", i);
                if let Ok(name) = std::fs::read_to_string(&name_path) {
                    if name.contains("AT Translated Set 2 keyboard") || name.contains("Acer WMI hotkeys") {
                        info!("Found keyboard/hotkey device at /dev/input/event{}: {}", i, name.trim());
                        let tx_clone = event_tx.clone();
                        let dev_path = format!("/dev/input/event{}", i);
                        thread::spawn(move || {
                            listen_to_device(&dev_path, tx_clone);
                        });
                        found = true;
                    }
                }
            }
            if found {
                break;
            }
            thread::sleep(Duration::from_secs(5));
        }
    });
}

fn listen_to_device(path: &str, event_tx: broadcast::Sender<String>) {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            warn!("Failed to open {}: {}", path, e);
            return;
        }
    };

    let mut buffer = [0u8; 24];
    loop {
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                let type_ = u16::from_ne_bytes([buffer[16], buffer[17]]);
                let code = u16::from_ne_bytes([buffer[18], buffer[19]]);
                let value = i32::from_ne_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]);

                if type_ == 1 && value == 1 {
                    if code == 425 || code == 148 {
                        info!("NitroSense/Acer key pressed! Code: {}", code);
                        if event_tx.receiver_count() > 0 {
                            if let Ok(json) = serde_json::to_string(&IpcEvent::ToggleGui {}) {
                                let _ = event_tx.send(json);
                            }
                        } else {
                            info!("GUI not connected. Spawning acercontrol-toggle");
                            let _ = std::process::Command::new("/usr/local/bin/acercontrol-toggle").spawn();
                        }
                    }
                }
            }
            Err(_) => {
                warn!("Lost connection to {}", path);
                break;
            }
        }
    }
}
