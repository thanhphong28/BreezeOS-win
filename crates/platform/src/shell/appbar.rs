use std::sync::Mutex;
use nacre_core::display::Area;
use windows::core::w;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Shell::{SHAppBarMessage, ABE_BOTTOM, ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBARDATA};
use windows::Win32::UI::WindowsAndMessaging::RegisterWindowMessageW;

static BARS: Mutex<Vec<(isize, bool)>> = Mutex::new(Vec::new());

fn data(h: HWND) -> APPBARDATA {
    APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: h, ..Default::default() }
}

fn reserve(hwnd: HWND, top_edge: bool, area: Area, strip_h: i32) -> Option<i32> {
    unsafe {
        let mut abd = data(hwnd);
        abd.uCallbackMessage = RegisterWindowMessageW(w!("NacreAppBar"));
        let mut bars = BARS.lock().ok()?;
        let key = hwnd.0 as isize;
        match bars.iter().position(|b| b.0 == key) {
            Some(i) if bars[i].1 == top_edge => {}
            found => {
                if let Some(i) = found {
                    bars.remove(i);
                }
                let _ = SHAppBarMessage(ABM_REMOVE, &mut abd);
                if SHAppBarMessage(ABM_NEW, &mut abd) == 0 {
                    return None;
                }
                bars.push((key, top_edge));
            }
        }
        drop(bars);
        if top_edge {
            abd.uEdge = ABE_TOP;
            abd.rc = RECT { left: area.x, top: area.y, right: area.right(), bottom: area.y + strip_h };
            SHAppBarMessage(ABM_QUERYPOS, &mut abd);
            abd.rc.bottom = abd.rc.top + strip_h;
            SHAppBarMessage(ABM_SETPOS, &mut abd);
            Some(abd.rc.bottom)
        } else {
            abd.uEdge = ABE_BOTTOM;
            abd.rc = RECT { left: area.x, top: area.bottom() - strip_h, right: area.right(), bottom: area.bottom() };
            SHAppBarMessage(ABM_QUERYPOS, &mut abd);
            abd.rc.top = abd.rc.bottom - strip_h;
            SHAppBarMessage(ABM_SETPOS, &mut abd);
            Some(abd.rc.top)
        }
    }
}

pub fn reserve_bottom(hwnd: HWND, area: Area, strip_h: i32) -> Option<i32> {
    reserve(hwnd, false, area, strip_h)
}

pub fn reserve_top(hwnd: HWND, area: Area, strip_h: i32) -> Option<i32> {
    reserve(hwnd, true, area, strip_h)
}

fn remove_where(f: impl Fn(&(isize, bool)) -> bool) {
    let gone: Vec<isize> = match BARS.lock() {
        Ok(mut bars) => {
            let g = bars.iter().filter(|b| f(b)).map(|b| b.0).collect();
            bars.retain(|b| !f(b));
            g
        }
        Err(_) => return,
    };
    for h in gone {
        unsafe {
            let mut abd = data(HWND(h as *mut _));
            SHAppBarMessage(ABM_REMOVE, &mut abd);
        }
    }
}

pub fn release_top() {
    remove_where(|b| b.1);
}

pub fn release_top_if(hwnd: HWND) {
    let key = hwnd.0 as isize;
    remove_where(|b| b.0 == key && b.1);
}

pub fn release_bottom() {
    remove_where(|b| !b.1);
}

pub fn release_bottom_if(hwnd: HWND) {
    let key = hwnd.0 as isize;
    remove_where(|b| b.0 == key && !b.1);
}

pub fn release() {
    remove_where(|_| true);
}
