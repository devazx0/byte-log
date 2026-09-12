use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveWindowInfo {
    pub process_id: u32,
    pub executable_name: String,
    pub window_title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserIdleStatus {
    Active,
    Idle(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypingInputEvent {
    StandardKey,
    Backspace,
    Enter,
    Modifier,
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percentage: f32,
    pub is_charging: bool,
    pub health_percent: Option<f32>,
    pub cycle_count: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub model_name: String,
    pub utilization_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub temperature_celsius: Option<f32>,
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("Permission denied by operating system: {0}")]
    PermissionDenied(String),
    #[error("API not supported on this platform: {0}")]
    Unsupported(String),
    #[error("OS system call failed with error code: {0}")]
    SystemCallFailed(i32),
    #[error("Platform error: {0}")]
    General(String),
}

pub trait ActiveApplicationProvider: Send + Sync {
    fn get_active_window(&self) -> Option<ActiveWindowInfo>;
}

pub trait UserActivityProvider: Send + Sync {
    fn get_idle_time(&self) -> Result<Duration, PlatformError>;
    fn is_session_locked(&self) -> bool;
}

pub trait RawInputHookProvider: Send + Sync {
    fn start_hook(
        &self,
        sender: crossbeam_channel::Sender<TypingInputEvent>,
    ) -> Result<(), PlatformError>;
    fn stop_hook(&self);
    fn is_hook_active(&self) -> bool;
}

pub trait BatteryProvider: Send + Sync {
    fn get_battery_info(&self) -> Option<BatteryInfo>;
}

pub trait GpuProvider: Send + Sync {
    fn get_gpu_info(&self) -> Option<GpuInfo>;
}
