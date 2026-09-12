#[cfg(windows)]
pub mod windows_impl {
    use super::super::traits::*;
    use crossbeam_channel::Sender;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use std::ptr::null_mut;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    use windows_sys::Win32::System::SystemInformation::GetTickCount64;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextW,
        GetWindowThreadProcessId, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
        HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
    };

    #[repr(C)]
    #[allow(non_snake_case, clippy::upper_case_acronyms)]
    struct LASTINPUTINFO {
        cbSize: u32,
        dwTime: u32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetLastInputInfo(plii: *mut LASTINPUTINFO) -> BOOL;
    }

    const VK_BACK: u16 = 0x08;
    const VK_RETURN: u16 = 0x0D;
    const VK_SHIFT: u16 = 0x10;
    const VK_CONTROL: u16 = 0x11;
    const VK_MENU: u16 = 0x12;
    const VK_LWIN: u16 = 0x5B;
    const VK_RWIN: u16 = 0x5C;
    const VK_LSHIFT: u16 = 0xA0;
    const VK_RSHIFT: u16 = 0xA1;
    const VK_LCONTROL: u16 = 0xA2;
    const VK_RCONTROL: u16 = 0xA3;
    const VK_LMENU: u16 = 0xA4;
    const VK_RMENU: u16 = 0xA5;

    pub struct WindowsPlatform {
        hook_active: AtomicBool,
        hook_thread_id: Mutex<Option<u32>>,
    }

    impl WindowsPlatform {
        pub fn new() -> Self {
            Self {
                hook_active: AtomicBool::new(false),
                hook_thread_id: Mutex::new(None),
            }
        }
    }

    impl Default for WindowsPlatform {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ActiveApplicationProvider for WindowsPlatform {
        fn get_active_window(&self) -> Option<ActiveWindowInfo> {
            unsafe {
                let hwnd: HWND = GetForegroundWindow();
                if hwnd.is_null() {
                    return None;
                }

                let mut process_id: u32 = 0;
                GetWindowThreadProcessId(hwnd, &mut process_id);
                if process_id == 0 {
                    return None;
                }

                let h_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
                let executable_name = if !h_process.is_null() {
                    let mut path_buf = [0u16; 1024];
                    let mut size = path_buf.len() as u32;
                    let success =
                        QueryFullProcessImageNameW(h_process, 0, path_buf.as_mut_ptr(), &mut size);
                    CloseHandle(h_process);

                    if success != 0 && size > 0 {
                        let path_os = OsString::from_wide(&path_buf[..size as usize]);
                        let path = Path::new(&path_os);
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .map(|s| s.to_lowercase())
                            .unwrap_or_else(|| "unknown".to_string())
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    "unknown".to_string()
                };

                let mut title_buf = [0u16; 512];
                let title_len =
                    GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
                let window_title = if title_len > 0 {
                    Some(
                        OsString::from_wide(&title_buf[..title_len as usize])
                            .to_string_lossy()
                            .into_owned(),
                    )
                } else {
                    None
                };

                Some(ActiveWindowInfo {
                    process_id,
                    executable_name,
                    window_title,
                })
            }
        }
    }

    impl UserActivityProvider for WindowsPlatform {
        fn get_idle_time(&self) -> Result<Duration, PlatformError> {
            unsafe {
                let mut lii = LASTINPUTINFO {
                    cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                    dwTime: 0,
                };

                if GetLastInputInfo(&mut lii) == 0 {
                    return Err(PlatformError::SystemCallFailed(0));
                }

                let now_ticks = GetTickCount64();
                let last_input_ticks = lii.dwTime as u64;

                let idle_ms = now_ticks.saturating_sub(last_input_ticks);

                Ok(Duration::from_millis(idle_ms))
            }
        }

        fn is_session_locked(&self) -> bool {
            // If GetForegroundWindow returns NULL and idle is high, or screen lock is active
            unsafe { GetForegroundWindow().is_null() }
        }
    }

    static HOOK_SENDER: Mutex<Option<Sender<TypingInputEvent>>> = Mutex::new(None);

    unsafe extern "system" fn low_level_keyboard_proc(
        code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if code >= 0 && (w_param == WM_KEYDOWN as usize || w_param == WM_SYSKEYDOWN as usize) {
            let kbd_struct = *(l_param as *const KBDLLHOOKSTRUCT);
            let vk = kbd_struct.vkCode as u16;

            let event = match vk {
                VK_BACK => TypingInputEvent::Backspace,
                VK_RETURN => TypingInputEvent::Enter,
                VK_SHIFT | VK_LSHIFT | VK_RSHIFT | VK_CONTROL | VK_LCONTROL | VK_RCONTROL
                | VK_MENU | VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN => TypingInputEvent::Modifier,
                _ => TypingInputEvent::StandardKey,
            };

            if let Ok(guard) = HOOK_SENDER.lock() {
                if let Some(ref sender) = *guard {
                    let _ = sender.try_send(event);
                }
            }
        }

        CallNextHookEx(null_mut(), code, w_param, l_param)
    }

    impl RawInputHookProvider for WindowsPlatform {
        fn start_hook(&self, sender: Sender<TypingInputEvent>) -> Result<(), PlatformError> {
            if self.hook_active.load(Ordering::SeqCst) {
                return Ok(());
            }

            {
                let mut guard = HOOK_SENDER.lock().unwrap();
                *guard = Some(sender);
            }

            let (init_tx, init_rx) = crossbeam_channel::bounded::<Result<u32, PlatformError>>(1);

            thread::spawn(move || unsafe {
                let hook: HHOOK =
                    SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), null_mut(), 0);

                if hook.is_null() {
                    let _ = init_tx.send(Err(PlatformError::General(
                        "Failed to install low-level keyboard hook".to_string(),
                    )));
                    return;
                }

                let thread_id = windows_sys::Win32::System::Threading::GetCurrentThreadId();
                let _ = init_tx.send(Ok(thread_id));

                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                    if msg.message == WM_QUIT {
                        break;
                    }
                    DispatchMessageW(&msg);
                }

                UnhookWindowsHookEx(hook);
            });

            match init_rx.recv() {
                Ok(Ok(tid)) => {
                    *self.hook_thread_id.lock().unwrap() = Some(tid);
                    self.hook_active.store(true, Ordering::SeqCst);
                    Ok(())
                }
                Ok(Err(e)) => Err(e),
                Err(_) => Err(PlatformError::General(
                    "Hook thread initialization failed".to_string(),
                )),
            }
        }

        fn stop_hook(&self) {
            if self.hook_active.swap(false, Ordering::SeqCst) {
                if let Ok(mut guard) = self.hook_thread_id.lock() {
                    if let Some(tid) = guard.take() {
                        unsafe {
                            PostThreadMessageW(tid, WM_QUIT, 0, 0);
                        }
                    }
                }
                let mut guard = HOOK_SENDER.lock().unwrap();
                *guard = None;
            }
        }

        fn is_hook_active(&self) -> bool {
            self.hook_active.load(Ordering::SeqCst)
        }
    }

    impl BatteryProvider for WindowsPlatform {
        fn get_battery_info(&self) -> Option<BatteryInfo> {
            unsafe {
                let mut sps: SYSTEM_POWER_STATUS = std::mem::zeroed();
                if GetSystemPowerStatus(&mut sps) != 0 && sps.BatteryLifePercent != 255 {
                    return Some(BatteryInfo {
                        percentage: sps.BatteryLifePercent as f32,
                        is_charging: (sps.ACLineStatus == 1),
                        health_percent: None,
                        cycle_count: None,
                    });
                }
                None
            }
        }
    }

    impl GpuProvider for WindowsPlatform {
        fn get_gpu_info(&self) -> Option<GpuInfo> {
            // Null-safe graceful degradation per FR-08 / Sec 9
            None
        }
    }
}
