use crate::error::{PcStatsError, Result};
use std::fs;
use std::path::PathBuf;
use tracing::info;

pub struct LinuxSystemdManager;

impl LinuxSystemdManager {
    pub fn get_service_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").map_err(|_| {
            PcStatsError::ServiceManager("HOME environment variable not set".to_string())
        })?;
        Ok(PathBuf::from(home).join(".config/systemd/user/bytelog.service"))
    }

    pub fn install() -> Result<()> {
        let service_path = Self::get_service_path()?;
        if let Some(parent) = service_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let exe_path = std::env::current_exe().map_err(|e| {
            PcStatsError::ServiceManager(format!("Failed to determine exe path: {}", e))
        })?;

        let unit_content = format!(
            r#"[Unit]
Description=ByteLog Local Telemetry Daemon
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart={} daemon run
Restart=always
RestartSec=5
Nice=10

[Install]
WantedBy=graphical-session.target
"#,
            exe_path.display()
        );

        fs::write(&service_path, unit_content)?;
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output();

        info!("Systemd user service written to {}", service_path.display());
        Ok(())
    }

    pub fn start() -> Result<()> {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "start", "bytelog"])
            .output();
        Ok(())
    }

    pub fn stop() -> Result<()> {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "stop", "bytelog"])
            .output();
        Ok(())
    }

    pub fn status() -> Result<String> {
        let output = std::process::Command::new("systemctl")
            .args(["--user", "status", "bytelog"])
            .output();

        match output {
            Ok(out) => Ok(String::from_utf8_lossy(&out.stdout).trim().to_string()),
            Err(_) => Ok("Status: Inactive or systemctl unavailable".to_string()),
        }
    }

    pub fn uninstall() -> Result<()> {
        let _ = Self::stop();
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "disable", "bytelog"])
            .output();

        let service_path = Self::get_service_path()?;
        if service_path.exists() {
            fs::remove_file(service_path)?;
        }
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output();
        Ok(())
    }
}
