use std::path::Path;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
use windows::Win32::UI::WindowsAndMessaging::*;

pub const SLIVER: i32 = 4;

fn rect_is_parked(r: &RECT) -> bool {
    r.right > 0 && r.right <= SLIVER + 2 && r.bottom > 0 && r.bottom <= SLIVER + 2 && r.left < -100 && r.top < -50
}

pub fn rect_on_monitors(r: (i32, i32, i32, i32), mons: &[(i32, i32, i32, i32)]) -> bool {
    let (l, t, right, bottom) = r;
    if right <= l || bottom <= t {
        return false;
    }
    let min_x = (right - l).min(40).max(1);
    let min_y = (bottom - t).min(28).max(1);
    mons.iter().any(|&(mx, my, mw, mh)| {
        let ox = right.min(mx + mw) - l.max(mx);
        let oy = bottom.min(my + mh) - t.max(my);
        ox >= min_x && oy >= min_y
    })
}

pub fn user_dragging() -> bool {
    unsafe {
        if GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 || GetAsyncKeyState(VK_RBUTTON.0 as i32) < 0 {
            return true;
        }
        let mut gti = GUITHREADINFO { cbSize: std::mem::size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        GetGUIThreadInfo(0, &mut gti).is_ok() && (gti.flags & (GUI_INMOVESIZE | GUI_INMENUMODE)).0 != 0
    }
}

pub fn is_on_screen(hwnd: isize) -> bool {
    unsafe {
        let w = h(hwnd);
        let mut r = RECT::default();
        if GetWindowRect(w, &mut r).is_err() || rect_is_parked(&r) {
            return false;
        }
        let mons: Vec<(i32, i32, i32, i32)> = crate::monitors::list().into_iter().map(|m| (m.area.x, m.area.y, m.area.w, m.area.h)).collect();
        rect_on_monitors((r.left, r.top, r.right, r.bottom), &mons)
    }
}

pub fn rescue_offscreen(hwnd: isize) -> bool {
    unsafe {
        let w = h(hwnd);
        if !IsWindow(Some(w)).as_bool() || IsIconic(w).as_bool() || is_on_screen(hwnd) {
            return false;
        }
        let mut r = RECT::default();
        if GetWindowRect(w, &mut r).is_err() {
            return false;
        }
        let mon = crate::monitors::under_cursor().work;
        let ww = (r.right - r.left).clamp(240, mon.w.max(240));
        let wh = (r.bottom - r.top).clamp(160, mon.h.max(160));
        let x = mon.x + ((mon.w - ww) / 2).max(0);
        let y = mon.y + ((mon.h - wh) / 2).max(0);
        let _ = SetWindowPos(w, None, x, y, ww, wh, SWP_NOACTIVATE | SWP_NOZORDER);
        true
    }
}

pub fn rescue_all_stranded(own_pid: u32, exclude: &[isize]) -> usize {
    let mut n = 0;
    for w in super::enumerate::list(own_pid) {
        if !exclude.contains(&w.hwnd) && !is_on_screen(w.hwnd) && rescue_offscreen(w.hwnd) {
            n += 1;
        }
    }
    n
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parked {
    pub hwnd: isize,
    pub rect: (i32, i32, i32, i32),
    pub maximized: bool,
}

fn h(v: isize) -> HWND {
    HWND(v as *mut _)
}

pub fn park(hwnd: isize) -> Option<Parked> {
    unsafe {
        let w = h(hwnd);
        if !IsWindow(Some(w)).as_bool() || IsIconic(w).as_bool() {
            return None;
        }
        let mut wp = WINDOWPLACEMENT { length: std::mem::size_of::<WINDOWPLACEMENT>() as u32, ..Default::default() };
        GetWindowPlacement(w, &mut wp).ok()?;
        let maximized = IsZoomed(w).as_bool();
        let r = wp.rcNormalPosition;
        let mut cur = RECT::default();
        GetWindowRect(w, &mut cur).ok()?;
        let rect = if maximized { (r.left, r.top, r.right, r.bottom) } else { (cur.left, cur.top, cur.right, cur.bottom) };
        if rect_is_parked(&cur) {
            return None;
        }
        let size = (cur.right - cur.left, cur.bottom - cur.top);
        if maximized {
            let _ = ShowWindow(w, SW_RESTORE);
        }
        let (pw, ph) = (size.0.max(200), size.1.max(120));
        let _ = SetWindowPos(w, None, SLIVER - pw, SLIVER - ph, pw, ph, SWP_NOACTIVATE | SWP_NOZORDER);
        Some(Parked { hwnd, rect, maximized })
    }
}

pub fn unpark(p: &Parked) {
    unsafe {
        let w = h(p.hwnd);
        if !IsWindow(Some(w)).as_bool() {
            return;
        }
        let (l, t, r, b) = p.rect;
        let _ = SetWindowPos(w, None, l, t, r - l, b - t, SWP_NOACTIVATE | SWP_NOZORDER);
        if p.maximized {
            let _ = ShowWindow(w, SW_MAXIMIZE);
        }
    }
}

pub fn is_parked(hwnd: isize) -> bool {
    unsafe {
        let mut r = RECT::default();
        GetWindowRect(h(hwnd), &mut r).is_ok() && rect_is_parked(&r)
    }
}

pub fn save(path: &Path, list: &[Parked]) {
    if list.is_empty() {
        let _ = std::fs::remove_file(path);
        return;
    }
    let mut s = String::new();
    for p in list {
        s.push_str(&format!("{} {} {} {} {} {}\n", p.hwnd, p.rect.0, p.rect.1, p.rect.2, p.rect.3, p.maximized as u8));
    }
    let _ = std::fs::write(path, s);
}

pub fn load(path: &Path) -> Vec<Parked> {
    let Ok(s) = std::fs::read_to_string(path) else { return vec![] };
    s.lines()
        .filter_map(|l| {
            let v: Vec<i64> = l.split_whitespace().filter_map(|x| x.parse().ok()).collect();
            (v.len() == 6).then(|| Parked { hwnd: v[0] as isize, rect: (v[1] as i32, v[2] as i32, v[3] as i32, v[4] as i32), maximized: v[5] != 0 })
        })
        .collect()
}

pub fn restore_file(path: &Path) -> usize {
    let list = load(path);
    for p in &list {
        if is_parked(p.hwnd) || !is_on_screen(p.hwnd) {
            unpark(p);
        }
    }
    let _ = std::fs::remove_file(path);
    list.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_offscreen_and_multimonitor_rects() {
        let mons = [(0, 0, 1920, 1080), (1920, 0, 1920, 1080)];
        // Normal window on monitor 1 or monitor 2
        assert!(rect_on_monitors((100, 100, 900, 700), &mons));
        assert!(rect_on_monitors((2476, 212, 3430, 1049), &mons));
        // Parked sliver at (-950, -833, 4, 4) or drifted while parked (-940, -820, 14, 17)
        assert!(!rect_on_monitors((-950, -833, 4, 4), &mons));
        assert!(!rect_on_monitors((-940, -820, 14, 17), &mons));
        assert!(!rect_on_monitors((-1200, -900, -246, -63), &mons));
    }
}
