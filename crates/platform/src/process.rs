use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, WaitForSingleObject, INFINITE, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SYNCHRONIZE,
};

#[derive(Debug, PartialEq, Eq)]
pub enum WaitOutcome {
    Exited,
    Unknown,
}

pub fn wait_for_exit(pid: u32) -> WaitOutcome {
    unsafe {
        match OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            Ok(h) => {
                let r = WaitForSingleObject(h, INFINITE);
                let _ = CloseHandle(h);
                if r == WAIT_OBJECT_0 {
                    WaitOutcome::Exited
                } else {
                    WaitOutcome::Unknown
                }
            }
            Err(_) if GetLastError() == ERROR_INVALID_PARAMETER => WaitOutcome::Exited,
            Err(_) => WaitOutcome::Unknown,
        }
    }
}

pub fn is_alive(pid: u32) -> bool {
    const STILL_ACTIVE: u32 = 259;
    unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => {
                let mut code = 0u32;
                let ok = GetExitCodeProcess(h, &mut code).is_ok();
                let _ = CloseHandle(h);
                ok && code == STILL_ACTIVE
            }
            Err(_) => GetLastError() != ERROR_INVALID_PARAMETER,
        }
    }
}

pub fn image_name(pid: u32) -> Option<String> {
    use windows::core::PWSTR;
    use windows::Win32::System::Threading::{QueryFullProcessImageNameW, PROCESS_NAME_WIN32};
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(h);
        if !ok {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().to_lowercase())
    }
}

pub fn is_same_program(pid: u32) -> bool {
    let Some(mine) = std::env::current_exe().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase())) else { return false };
    is_alive(pid) && image_name(pid).is_some_and(|n| n == mine)
}
