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
