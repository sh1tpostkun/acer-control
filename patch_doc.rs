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
