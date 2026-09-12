use super::traits::*;
use std::time::Duration;

pub struct MacOsPlatform;

impl MacOsPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveApplicationProvider for MacOsPlatform {
    fn get_active_window(&self) -> Option<ActiveWindowInfo> {
        None
    }
}

impl UserActivityProvider for MacOsPlatform {
    fn get_idle_time(&self) -> Result<Duration, PlatformError> {
        Err(PlatformError::Unsupported(
            "macOS idle time requires native build".to_string(),
        ))
    }

    fn is_session_locked(&self) -> bool {
        false
    }
}

impl RawInputHookProvider for MacOsPlatform {
    fn start_hook(
        &self,
        _sender: crossbeam_channel::Sender<TypingInputEvent>,
    ) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(
            "macOS input monitoring requires native build".to_string(),
        ))
    }

    fn stop_hook(&self) {}

    fn is_hook_active(&self) -> bool {
        false
    }
}

impl BatteryProvider for MacOsPlatform {
    fn get_battery_info(&self) -> Option<BatteryInfo> {
        None
    }
}

impl GpuProvider for MacOsPlatform {
    fn get_gpu_info(&self) -> Option<GpuInfo> {
        None
    }
}
