use windows::core::PCSTR;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowA, IsWindowVisible, ShowWindow, SW_HIDE};

pub fn hide_native() {
    unsafe {
        if let Ok(h) = FindWindowA(PCSTR(c"NativeHWNDHost".as_ptr() as *const u8), PCSTR::null()) {
            if IsWindowVisible(h).as_bool() {
                let _ = ShowWindow(h, SW_HIDE);
            }
        }
    }
}
