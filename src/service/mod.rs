pub mod daemon;
pub mod linux_systemd;
pub mod macos_launchd;
pub mod windows_task;

pub use daemon::Daemon;
pub use linux_systemd::LinuxSystemdManager;
pub use macos_launchd::MacOsLaunchdManager;
pub use windows_task::WindowsTaskManager;

use crate::error::Result;

pub struct ServiceManager;

impl ServiceManager {
    pub fn install() -> Result<()> {
        #[cfg(windows)]
        return WindowsTaskManager::install();
        #[cfg(target_os = "macos")]
        return MacOsLaunchdManager::install();
        #[cfg(target_os = "linux")]
        return LinuxSystemdManager::install();
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        Ok(())
    }

    pub fn start() -> Result<()> {
        #[cfg(windows)]
        return WindowsTaskManager::start();
        #[cfg(target_os = "macos")]
        return MacOsLaunchdManager::start();
        #[cfg(target_os = "linux")]
        return LinuxSystemdManager::start();
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        Ok(())
    }

    pub fn stop() -> Result<()> {
        #[cfg(windows)]
        return WindowsTaskManager::stop();
        #[cfg(target_os = "macos")]
        return MacOsLaunchdManager::stop();
        #[cfg(target_os = "linux")]
        return LinuxSystemdManager::stop();
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        Ok(())
    }

    pub fn status() -> Result<String> {
        #[cfg(windows)]
        return WindowsTaskManager::status();
        #[cfg(target_os = "macos")]
        return MacOsLaunchdManager::status();
        #[cfg(target_os = "linux")]
        return LinuxSystemdManager::status();
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        Ok("Status: Unsupported platform".to_string())
    }

    pub fn uninstall() -> Result<()> {
        #[cfg(windows)]
        return WindowsTaskManager::uninstall();
        #[cfg(target_os = "macos")]
        return MacOsLaunchdManager::uninstall();
        #[cfg(target_os = "linux")]
        return LinuxSystemdManager::uninstall();
        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        Ok(())
    }
}
