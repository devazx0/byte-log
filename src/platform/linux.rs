use super::traits::*;
use std::time::Duration;

pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LinuxPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveApplicationProvider for LinuxPlatform {
    fn get_active_window(&self) -> Option<ActiveWindowInfo> {
        None
    }
}

impl UserActivityProvider for LinuxPlatform {
    fn get_idle_time(&self) -> Result<Duration, PlatformError> {
        Err(PlatformError::Unsupported(
            "Linux idle detection requires native build".to_string(),
        ))
    }

    fn is_session_locked(&self) -> bool {
        false
    }
}

impl RawInputHookProvider for LinuxPlatform {
    fn start_hook(
        &self,
        _sender: crossbeam_channel::Sender<TypingInputEvent>,
    ) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(
            "Linux evdev/XRecord hook requires native build".to_string(),
        ))
    }

    fn stop_hook(&self) {}

    fn is_hook_active(&self) -> bool {
        false
    }
}

impl BatteryProvider for LinuxPlatform {
    fn get_battery_info(&self) -> Option<BatteryInfo> {
        None
    }
}

impl GpuProvider for LinuxPlatform {
    fn get_gpu_info(&self) -> Option<GpuInfo> {
        None
    }
}
