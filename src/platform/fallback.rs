use crate::platform::traits::*;
use crossbeam_channel::Sender;
use std::time::Duration;

pub struct FallbackPlatform;

impl FallbackPlatform {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FallbackPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveApplicationProvider for FallbackPlatform {
    fn get_active_window(&self) -> Option<ActiveWindowInfo> {
        None
    }
}

impl IdleDetector for FallbackPlatform {
    fn get_idle_time(&self) -> Result<Duration, PlatformError> {
        Ok(Duration::from_secs(0))
    }

    fn is_session_locked(&self) -> bool {
        false
    }
}

impl RawInputHookProvider for FallbackPlatform {
    fn start_hook(&self, _sender: Sender<TypingInputEvent>) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported)
    }

    fn stop_hook(&self) {}

    fn is_hook_active(&self) -> bool {
        false
    }
}

impl BatteryProvider for FallbackPlatform {
    fn get_battery_info(&self) -> Option<BatteryInfo> {
        None
    }
}

impl GpuProvider for FallbackPlatform {
    fn get_gpu_info(&self) -> Option<GpuInfo> {
        None
    }
}
