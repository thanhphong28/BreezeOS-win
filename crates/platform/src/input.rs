use crate::hookthread::{HookThread, Kind};
use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, PostMessageW, MSLLHOOKSTRUCT, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_XBUTTONDOWN,
};

static HOOK: Mutex<Option<HookThread>> = Mutex::new(None);
static TARGET: AtomicIsize = AtomicIsize::new(0);
static MSG: AtomicU32 = AtomicU32::new(0);
static RECT: [std::sync::atomic::AtomicI32; 4] = [const { std::sync::atomic::AtomicI32::new(0) }; 4];

unsafe extern "system" fn proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code >= 0 {
        let m = wp.0 as u32;
        if matches!(m, WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN) {
            let pt = (*(lp.0 as *const MSLLHOOKSTRUCT)).pt;
            let (l, t, r, b) = (
                RECT[0].load(Ordering::Relaxed),
                RECT[1].load(Ordering::Relaxed),
                RECT[2].load(Ordering::Relaxed),
                RECT[3].load(Ordering::Relaxed),
            );
            let inside = pt.x >= l && pt.x < r && pt.y >= t && pt.y < b;
            if !inside {
                let packed = ((pt.y as i16 as u16 as u32) << 16) | (pt.x as i16 as u16 as u32);
                let _ = PostMessageW(Some(HWND(TARGET.load(Ordering::Relaxed) as *mut _)), MSG.load(Ordering::Relaxed), WPARAM(0), LPARAM(packed as isize));
                return LRESULT(1);
            }
        }
    }
    CallNextHookEx(None, code, wp, lp)
}

pub fn begin_dismiss(rect: (i32, i32, i32, i32), target: isize, msg: u32) {
    end_dismiss();
    RECT[0].store(rect.0, Ordering::Relaxed);
    RECT[1].store(rect.1, Ordering::Relaxed);
    RECT[2].store(rect.2, Ordering::Relaxed);
    RECT[3].store(rect.3, Ordering::Relaxed);
    TARGET.store(target, Ordering::Relaxed);
    MSG.store(msg, Ordering::Relaxed);
    let h = HookThread::start(Kind::Mouse, Some(proc));
    *HOOK.lock().unwrap_or_else(|e| e.into_inner()) = h;
}

pub fn end_dismiss() {
    let h = HOOK.lock().unwrap_or_else(|e| e.into_inner()).take();
    drop(h);
}
