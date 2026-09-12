use crate::error::{PcStatsError, Result};
use std::process::Command;
use tracing::info;

const TASK_NAME: &str = "ByteLog";

pub struct WindowsTaskManager;

impl WindowsTaskManager {
    pub fn install() -> Result<()> {
        let exe_path = std::env::current_exe().map_err(|e| {
            PcStatsError::ServiceManager(format!("Failed to determine exe path: {}", e))
        })?;
        let exe_str = exe_path.to_string_lossy();

        info!(
            "Registering Windows logon task '{}' for {}...",
            TASK_NAME, exe_str
        );

        let output = Command::new("schtasks")
            .args([
                "/create",
                "/tn",
                TASK_NAME,
                "/tr",
                &format!("\"{}\" daemon run", exe_str),
                "/sc",
                "onlogon",
                "/rl",
                "limited",
                "/f",
            ])
            .output()
            .map_err(|e| {
                PcStatsError::ServiceManager(format!("Failed to execute schtasks: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PcStatsError::ServiceManager(format!(
                "schtasks failed: {}",
                stderr
            )));
        }

        info!("Task '{}' registered successfully.", TASK_NAME);
        Ok(())
    }

    pub fn start() -> Result<()> {
        let output = Command::new("schtasks")
            .args(["/run", "/tn", TASK_NAME])
            .output()
            .map_err(|e| PcStatsError::ServiceManager(format!("Failed to run schtasks: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PcStatsError::ServiceManager(format!(
                "Failed to start task: {}",
                stderr
            )));
        }

        info!("Task '{}' started.", TASK_NAME);
        Ok(())
    }

    pub fn stop() -> Result<()> {
        let output = Command::new("schtasks")
            .args(["/end", "/tn", TASK_NAME])
            .output()
            .map_err(|e| PcStatsError::ServiceManager(format!("Failed to end schtasks: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PcStatsError::ServiceManager(format!(
                "Failed to stop task: {}",
                stderr
            )));
        }

        info!("Task '{}' stopped.", TASK_NAME);
        Ok(())
    }

    pub fn status() -> Result<String> {
        let output = Command::new("schtasks")
            .args(["/query", "/tn", TASK_NAME, "/fo", "LIST"])
            .output()
            .map_err(|e| {
                PcStatsError::ServiceManager(format!("Failed to query schtasks: {}", e))
            })?;

        if !output.status.success() {
            return Ok("Status: Not installed or inactive".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn uninstall() -> Result<()> {
        let output = Command::new("schtasks")
            .args(["/delete", "/tn", TASK_NAME, "/f"])
            .output()
            .map_err(|e| {
                PcStatsError::ServiceManager(format!("Failed to delete schtasks: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PcStatsError::ServiceManager(format!(
                "Failed to delete task: {}",
                stderr
            )));
        }

        info!("Task '{}' unregistered successfully.", TASK_NAME);
        Ok(())
    }
}
