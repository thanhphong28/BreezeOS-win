use nacre_core::marker::{Marker, Rect};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use windows::core::{w, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::Shell::{
    SHAppBarMessage, ABM_GETSTATE, ABM_GETTASKBARPOS, ABM_SETSTATE, APPBARDATA,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetClassNameW, GetWindowRect, GetWindowThreadProcessId, IsWindowVisible,
    SetWindowPos, ShowWindow, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_SHOW, SWP_NOACTIVATE, SWP_NOSIZE,
    SWP_NOZORDER, SW_HIDE, SW_SHOW, WINEVENT_OUTOFCONTEXT,
};

const PRIMARY_CLASS: &str = "Shell_TrayWnd";
const SECONDARY_CLASS: &str = "Shell_SecondaryTrayWnd";
const PARK_X: i32 = -10000;
const ABS_AUTOHIDE: i32 = 1;
const ABS_ALWAYSONTOP: i32 = 2;

static HIDDEN: AtomicBool = AtomicBool::new(false);

struct Active {
    marker: Marker,
    path: PathBuf,
}
static ACTIVE: Mutex<Option<Active>> = Mutex::new(None);
static HOOKS: Mutex<Vec<isize>> = Mutex::new(Vec::new());

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn is_hidden() -> bool {
    HIDDEN.load(Ordering::Relaxed)
}

fn class_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 64];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

fn primary() -> Option<HWND> {
    unsafe { FindWindowW(w!("Shell_TrayWnd"), None).ok() }
}

fn secondaries() -> Vec<HWND> {
    unsafe extern "system" fn each(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let out = &mut *(lparam.0 as *mut Vec<HWND>);
        if class_of(hwnd) == SECONDARY_CLASS {
            out.push(hwnd);
        }
        BOOL(1)
    }
    let mut out: Vec<HWND> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(each), LPARAM(&mut out as *mut Vec<HWND> as isize));
    }
    out
}

fn rect_of(h: HWND) -> Option<Rect> {
    let mut r = RECT::default();
    unsafe { GetWindowRect(h, &mut r).ok()? };
    Some(Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom })
}

fn abm_data(h: HWND) -> APPBARDATA {
    APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: h, ..Default::default() }
}

fn abm_get_state(h: HWND) -> i32 {
    let mut d = abm_data(h);
    unsafe { SHAppBarMessage(ABM_GETSTATE, &mut d) as i32 }
}

fn abm_set_state(h: HWND, state: i32) {
    let mut d = abm_data(h);
    d.lParam = LPARAM(state as isize);
    unsafe {
        SHAppBarMessage(ABM_SETSTATE, &mut d);
    }
}

fn abm_taskbar_pos(h: HWND) -> Option<Rect> {
    let mut d = abm_data(h);
    if unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut d) } == 0 {
        return None;
    }
    Some(Rect { left: d.rc.left, top: d.rc.top, right: d.rc.right, bottom: d.rc.bottom })
}

fn is_parked(h: HWND) -> bool {
    unsafe { !IsWindowVisible(h).as_bool() }
}

fn park(h: HWND) {
    unsafe {
        let _ = ShowWindow(h, SW_HIDE);
        let _ = SetWindowPos(h, None, PARK_X, PARK_X, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
    }
}

fn place_and_show(h: HWND, rect: Option<Rect>) {
    unsafe {
        if let Some(r) = rect {
            let _ = SetWindowPos(h, None, r.left, r.top, 0, 0, SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        }
        let _ = ShowWindow(h, SW_SHOW);
    }
}

pub fn read_marker(path: &Path) -> Option<Marker> {
    Marker::parse(&std::fs::read_to_string(path).ok()?)
}

fn write_marker(path: &Path, m: &Marker) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(m.serialize().as_bytes())?;
    f.sync_all()?;
    drop(f);
    std::fs::rename(&tmp, path)
}

pub fn marker_exists(path: &Path) -> bool {
    path.exists()
}

pub fn restore_marker(m: &Marker) -> bool {
    let Some(p) = primary() else { return false };
    HIDDEN.store(false, Ordering::Relaxed);
    abm_set_state(p, if m.abstate >= 0 { m.abstate } else { ABS_ALWAYSONTOP });
    place_and_show(p, m.primary);
    let mut ok = unsafe { IsWindowVisible(p).as_bool() } && rect_of(p).is_some_and(|r| r.is_placed());
    let mons = crate::monitors::list();
    let mut taken: Vec<nacre_core::display::Area> = Vec::new();
    for s in secondaries() {
        let size = rect_of(s).map(|r| (r.right - r.left, r.bottom - r.top)).unwrap_or((0, 0));
        let home = secondary_home(m.secondary_rect(s.0 as isize), size, &mons, &taken);
        if let Some((x, y)) = home {
            if let Some(mon) = nacre_core::display::at_point(&mons, x + size.0 / 2, y + size.1 / 2) {
                taken.push(mon.area);
            }
        }
        place_and_show(s, home.map(|(x, y)| Rect { left: x, top: y, right: x + size.0, bottom: y + size.1 }));
        ok &= unsafe { IsWindowVisible(s).as_bool() } && rect_of(s).is_some_and(|r| r.is_placed());
    }
    ok
}

fn secondary_home(saved: Option<Rect>, size: (i32, i32), mons: &[nacre_core::display::Monitor], taken: &[nacre_core::display::Area]) -> Option<(i32, i32)> {
    let others = || mons.iter().filter(|m| !m.primary && !taken.contains(&m.area));
    if let Some(r) = saved.filter(|r| r.is_placed()) {
        let (cx, cy) = ((r.left + r.right) / 2, (r.top + r.bottom) / 2);
        if others().any(|m| m.area.contains(cx, cy)) {
            return Some((r.left, r.top));
        }
    }
    others().next().map(|m| (m.area.x, m.area.bottom() - size.1.max(0)))
}

#[cfg(test)]
mod home_tests {
    use super::*;
    use nacre_core::display::{Area, Monitor};

    fn mon(x: i32, w: i32, primary: bool) -> Monitor {
        Monitor { area: Area::new(x, 0, w, 1080), work: Area::new(x, 0, w, 1080), dpi: 96, primary }
    }

    #[test]
    fn saved_position_on_a_live_screen_is_kept() {
        let mons = [mon(0, 1920, true), mon(1920, 1920, false)];
        let saved = Rect { left: 1920, top: 1032, right: 3840, bottom: 1080 };
        assert_eq!(secondary_home(Some(saved), (1920, 48), &mons, &[]), Some((1920, 1032)));
    }

    #[test]
    fn bar_without_saved_position_goes_to_bottom_of_a_free_screen() {
        let mons = [mon(0, 1920, true), mon(1920, 1280, false), mon(-1280, 1280, false)];
        assert_eq!(secondary_home(None, (1280, 48), &mons, &[]), Some((1920, 1032)));
        assert_eq!(secondary_home(None, (1280, 48), &mons, &[Area::new(1920, 0, 1280, 1080)]), Some((-1280, 1032)));
    }

    #[test]
    fn stale_position_from_an_unplugged_screen_is_moved() {
        let mons = [mon(0, 1920, true), mon(1920, 1920, false)];
        let saved = Rect { left: -1920, top: 1032, right: 0, bottom: 1080 };
        assert_eq!(secondary_home(Some(saved), (1920, 48), &mons, &[]), Some((1920, 1032)));
        assert_eq!(secondary_home(Some(saved), (1920, 48), &[mon(0, 1920, true)], &[]), None);
    }
}

pub fn recover_leftover(path: &Path) -> bool {
    if !path.exists() {
        return true;
    }
    let m = read_marker(path).unwrap_or_else(|| Marker::new(-1, 0));
    if m.pid != 0 && m.pid != std::process::id() && crate::process::is_same_program(m.pid) {
        crate::log::line("taskbar", "marker dimiliki proses BreezeOS lain yang masih hidup; tidak disentuh");
        return false;
    }
    let ok = restore_marker(&m);
    if ok {
        let _ = std::fs::remove_file(path);
    }
    crate::log::line("taskbar", &format!("recover_leftover ok={ok}"));
    ok
}

pub fn hide(marker_path: &Path, guard_exe: &Path) -> Result<(), String> {
    if is_hidden() {
        reassert();
        return Ok(());
    }
    if marker_path.exists() && !recover_leftover(marker_path) {
        let owner = read_marker(marker_path).map(|m| m.pid).unwrap_or(0);
        if owner != 0 && owner != std::process::id() && crate::process::is_same_program(owner) {
            return Err("taskbar sedang dipegang BreezeOS lain".into());
        }
        crate::log::line("taskbar", "sisa sesi lama tidak bisa dipulihkan; diganti sesi baru");
        let _ = std::fs::remove_file(marker_path);
    }
    let p = primary().ok_or("taskbar tidak ditemukan")?;
    let pid = std::process::id();

    let mut m = Marker::new(abm_get_state(p), pid);
    m.primary = rect_of(p).filter(|r| r.is_placed()).or_else(|| abm_taskbar_pos(p).filter(|r| r.is_placed()));
    for s in secondaries() {
        if let Some(r) = rect_of(s).filter(|r| r.is_placed()) {
            m.secondaries.push((s.0 as isize, r));
        }
    }

    write_marker(marker_path, &m).map_err(|e| format!("marker: {e}"))?;
    if let Err(e) = super::guard_spawn::spawn_once(guard_exe, pid, marker_path) {
        crate::log::line("taskbar", &format!("guard tidak bisa dijalankan, taskbar tetap disembunyikan: {e}"));
    }

    *lock(&ACTIVE) = Some(Active { marker: m, path: marker_path.to_path_buf() });
    abm_set_state(p, ABS_AUTOHIDE);
    park(p);
    for s in secondaries() {
        park(s);
    }
    HIDDEN.store(true, Ordering::Relaxed);
    install_enforcer();
    crate::log::line("taskbar", "hidden");
    Ok(())
}

pub fn restore() -> bool {
    let active = lock(&ACTIVE).take();
    remove_enforcer();
    let Some(a) = active else {
        HIDDEN.store(false, Ordering::Relaxed);
        return true;
    };
    let ok = restore_marker(&a.marker);
    if ok {
        let _ = std::fs::remove_file(&a.path);
    } else {
        *lock(&ACTIVE) = Some(a);
    }
    crate::log::line("taskbar", &format!("restore ok={ok}"));
    ok
}

pub fn reassert() {
    if !is_hidden() {
        return;
    }
    if let Some(p) = primary() {
        abm_set_state(p, ABS_AUTOHIDE);
        park(p);
    }
    for s in secondaries() {
        park(s);
    }
    install_enforcer();
}

unsafe extern "system" fn enforcer(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if !is_hidden() || id_object != 0 || hwnd.0.is_null() {
        return;
    }
    let c = class_of(hwnd);
    if (c == PRIMARY_CLASS || c == SECONDARY_CLASS) && !is_parked(hwnd) {
        park(hwnd);
    }
}

fn install_enforcer() {
    remove_enforcer();
    let Some(p) = primary() else { return };
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(p, Some(&mut pid)) };
    if pid == 0 {
        return;
    }
    let mut hooks = lock(&HOOKS);
    for ev in [EVENT_OBJECT_SHOW, EVENT_OBJECT_LOCATIONCHANGE] {
        let h = unsafe { SetWinEventHook(ev, ev, None, Some(enforcer), pid, 0, WINEVENT_OUTOFCONTEXT) };
        if !h.is_invalid() {
            hooks.push(h.0 as isize);
        }
    }
}

fn remove_enforcer() {
    for h in lock(&HOOKS).drain(..) {
        unsafe {
            let _ = UnhookWinEvent(HWINEVENTHOOK(h as *mut _));
        }
    }
}
