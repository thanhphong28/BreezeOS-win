use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetDC, GetPixel, ReleaseDC};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{SetFocus, VK_ESCAPE};
use windows::Win32::UI::WindowsAndMessaging::*;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static PICKED: AtomicU32 = AtomicU32::new(u32::MAX);
const T_TIMEOUT: usize = 1;

pub fn hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

fn sample(pt: POINT) -> Option<[u8; 3]> {
    unsafe {
        let dc = GetDC(None);
        if dc.is_invalid() {
            return None;
        }
        let c: COLORREF = GetPixel(dc, pt.x, pt.y);
        ReleaseDC(None, dc);
        (c.0 != 0xFFFF_FFFF).then(|| [(c.0 & 0xFF) as u8, ((c.0 >> 8) & 0xFF) as u8, ((c.0 >> 16) & 0xFF) as u8])
    }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_SETCURSOR => {
            if let Ok(c) = LoadCursorW(None, IDC_CROSS) {
                SetCursor(Some(c));
            }
            LRESULT(1)
        }
        WM_LBUTTONDOWN => {
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            let _ = ShowWindow(hwnd, SW_HIDE);
            std::thread::sleep(std::time::Duration::from_millis(60));
            if let Some(rgb) = sample(pt) {
                PICKED.store(((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | rgb[2] as u32, Ordering::Relaxed);
            }
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_KEYDOWN if wp.0 as u16 == VK_ESCAPE.0 => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_TIMER if wp.0 == T_TIMEOUT => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

pub fn pick(done: impl FnOnce(Option<[u8; 3]>) + Send + 'static) -> bool {
    if ACTIVE.swap(true, Ordering::AcqRel) {
        return false;
    }
    PICKED.store(u32::MAX, Ordering::Relaxed);
    std::thread::spawn(move || unsafe {
        let made = (|| -> windows_core::Result<()> {
            let hi = GetModuleHandleW(None)?.into();
            let class = w!("BreezeOSColorPick");
            RegisterClassW(&WNDCLASSW { lpfnWndProc: Some(proc), hInstance: hi, lpszClassName: class, ..Default::default() });
            let (x, y, cw, ch) = (GetSystemMetrics(SM_XVIRTUALSCREEN), GetSystemMetrics(SM_YVIRTUALSCREEN), GetSystemMetrics(SM_CXVIRTUALSCREEN), GetSystemMetrics(SM_CYVIRTUALSCREEN));
            let hwnd = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED, class, w!("breezeos-colorpick"), WS_POPUP, x, y, cw, ch, None, None, Some(hi), None)?;
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 1, LWA_ALPHA)?;
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
            let _ = SetFocus(Some(hwnd));
            SetTimer(Some(hwnd), T_TIMEOUT, 20_000, None);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            Ok(())
        })();
        if let Err(e) = made {
            crate::log::line("warna", &format!("pengambil warna gagal: {e}"));
        }
        let v = PICKED.load(Ordering::Relaxed);
        ACTIVE.store(false, Ordering::Release);
        done((v != u32::MAX).then(|| [(v >> 16) as u8, (v >> 8) as u8, v as u8]));
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_is_uppercase_six_digits() {
        assert_eq!(hex([127, 119, 221]), "#7F77DD");
        assert_eq!(hex([0, 5, 255]), "#0005FF");
    }
}
