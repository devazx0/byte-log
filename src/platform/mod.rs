pub mod traits;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(windows)]
pub mod windows;

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub mod fallback;

pub use traits::*;

#[cfg(windows)]
pub type Platform = windows::windows_impl::WindowsPlatform;

#[cfg(target_os = "macos")]
pub type Platform = macos::MacOsPlatform;

#[cfg(target_os = "linux")]
pub type Platform = linux::LinuxPlatform;

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub type Platform = fallback::FallbackPlatform;

pub fn create_platform() -> Platform {
    Platform::new()
}
