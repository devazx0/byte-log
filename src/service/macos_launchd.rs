use crate::error::{PcStatsError, Result};
use std::fs;
use std::path::PathBuf;
use tracing::info;

pub struct MacOsLaunchdManager;

impl MacOsLaunchdManager {
    pub fn get_plist_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").map_err(|_| {
            PcStatsError::ServiceManager("HOME environment variable not set".to_string())
        })?;
        Ok(PathBuf::from(home).join("Library/LaunchAgents/com.bytelog.daemon.plist"))
    }

    pub fn install() -> Result<()> {
        let plist_path = Self::get_plist_path()?;
        if let Some(parent) = plist_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let exe_path = std::env::current_exe().map_err(|e| {
            PcStatsError::ServiceManager(format!("Failed to determine exe path: {}", e))
        })?;

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.bytelog.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>daemon</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>"#,
            exe_path.display()
        );

        fs::write(&plist_path, plist_content)?;
        info!("LaunchAgent written to {}", plist_path.display());
        Ok(())
    }

    pub fn start() -> Result<()> {
        let plist_path = Self::get_plist_path()?;
        let _ = std::process::Command::new("launchctl")
            .args(["load", &plist_path.to_string_lossy()])
            .output();
        Ok(())
    }

    pub fn stop() -> Result<()> {
        let plist_path = Self::get_plist_path()?;
        let _ = std::process::Command::new("launchctl")
            .args(["unload", &plist_path.to_string_lossy()])
            .output();
        Ok(())
    }

    pub fn status() -> Result<String> {
        let output = std::process::Command::new("launchctl")
            .args(["list", "com.bytelog.daemon"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
            }
            _ => Ok("Status: Inactive or not registered".to_string()),
        }
    }

    pub fn uninstall() -> Result<()> {
        let plist_path = Self::get_plist_path()?;
        let _ = Self::stop();
        if plist_path.exists() {
            fs::remove_file(plist_path)?;
        }
        Ok(())
    }
}
