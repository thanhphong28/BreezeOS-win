use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CLOAKED, EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE,
    EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
    EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND, GetAncestor, GetClassNameW, GetForegroundWindow,
    GetWindowLongW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, GA_ROOT, GA_ROOTOWNER, GWL_EXSTYLE,
    WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
};

static FOCUS: Mutex<Option<HashMap<isize, i64>>> = Mutex::new(None);
static CALLBACKS: Mutex<Vec<fn()>> = Mutex::new(Vec::new());
static FX_CALLBACKS: Mutex<Vec<fn(u32, isize)>> = Mutex::new(Vec::new());
static HOOKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static EXTERNAL_FG: AtomicIsize = AtomicIsize::new(0);

fn class_of(h: HWND) -> String {
    let mut b = [0u16; 128];
    let n = unsafe { GetClassNameW(h, &mut b) };
    String::from_utf16_lossy(&b[..n.max(0) as usize])
}

pub fn is_transient_shell_class(c: &str) -> bool {
    matches!(
        c,
        "ForegroundStaging"
            | "XamlExplorerHostIslandWindow"
            | "MultitaskingViewFrame"
            | "TaskSwitcherWnd"
            | "TaskSwitcherOverlayWnd"
            | "Shell_TrayWnd"
            | "Shell_SecondaryTrayWnd"
            | "NotifyIconOverflowWindow"
            | "TopLevelWindowForOverflowXamlIsland"
    )
}

fn is_cloaked(h: HWND) -> bool {
    let mut cloaked = 0u32;
    unsafe { DwmGetWindowAttribute(h, DWMWA_CLOAKED, &mut cloaked as *mut u32 as *mut c_void, 4).is_ok() && cloaked != 0 }
}

unsafe fn check_candidate(h: HWND, own_pid: u32) -> Option<HWND> {
    if h.0.is_null() || !IsWindow(Some(h)).as_bool() || !IsWindowVisible(h).as_bool() || IsIconic(h).as_bool() || is_cloaked(h) {
        return None;
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(h, Some(&mut pid));
    if pid == 0 || pid == own_pid {
        return None;
    }
    let cls = class_of(h);
    if cls == "Progman" || cls == "WorkerW" {
        return Some(h);
    }
    if is_transient_shell_class(&cls) {
        return None;
    }
    let ex = GetWindowLongW(h, GWL_EXSTYLE) as u32;
    if ex & WS_EX_TOOLWINDOW.0 != 0 && ex & WS_EX_APPWINDOW.0 == 0 {
        return None;
    }
    Some(h)
}

unsafe fn normalize_external(hwnd: HWND, own_pid: u32) -> Option<HWND> {
    if hwnd.0.is_null() {
        return None;
    }
    let root = GetAncestor(hwnd, GA_ROOT);
    let top = if root.0.is_null() { hwnd } else { root };
    if let Some(ok) = check_candidate(top, own_pid) {
        return Some(ok);
    }
    let owner = GetAncestor(top, GA_ROOTOWNER);
    if !owner.0.is_null() && owner != top {
        return check_candidate(owner, own_pid);
    }
    None
}

pub fn external_foreground() -> isize {
    let own = std::process::id();
    unsafe {
        if let Some(live) = normalize_external(GetForegroundWindow(), own) {
            let h = live.0 as isize;
            EXTERNAL_FG.store(h, Ordering::Relaxed);
            return h;
        }
        let cached = EXTERNAL_FG.load(Ordering::Relaxed);
        if cached != 0 {
            if let Some(valid) = normalize_external(HWND(cached as *mut _), own) {
                return valid.0 as isize;
            }
        }
    }
    0
}

fn focus_map() -> MutexGuard<'static, Option<HashMap<isize, i64>>> {
    FOCUS.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn recent(hwnd: isize) -> i64 {
    focus_map().as_ref().and_then(|m| m.get(&hwnd).copied()).unwrap_or(0)
}

pub fn last_focused_except(exclude: isize, alive: impl Fn(isize) -> bool) -> Option<isize> {
    let map = focus_map();
    map.as_ref()?
        .iter()
        .filter(|(h, _)| **h != exclude && alive(**h))
        .max_by_key(|(_, t)| **t)
        .map(|(h, _)| *h)
}

pub fn prune(alive: impl Fn(isize) -> bool) {
    if let Some(m) = focus_map().as_mut() {
        m.retain(|h, _| alive(*h));
    }
}

unsafe extern "system" fn callback(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if hwnd.0.is_null() || id_object != 0 || id_child != 0 {
        return;
    }
    if matches!(
        event,
        EVENT_SYSTEM_FOREGROUND
            | EVENT_SYSTEM_MINIMIZESTART
            | EVENT_SYSTEM_MINIMIZEEND
            | EVENT_OBJECT_DESTROY
            | EVENT_SYSTEM_MOVESIZEEND
            | EVENT_OBJECT_SHOW
            | EVENT_OBJECT_CLOAKED
            | EVENT_OBJECT_UNCLOAKED
    ) || (event == EVENT_OBJECT_LOCATIONCHANGE && hwnd == GetForegroundWindow())
    {
        let fx: Vec<fn(u32, isize)> = FX_CALLBACKS.lock().unwrap_or_else(|e| e.into_inner()).clone();
        for f in fx {
            f(event, hwnd.0 as isize);
        }
    }
    if event == EVENT_OBJECT_UNCLOAKED {
        return;
    }
    if event == EVENT_OBJECT_LOCATIONCHANGE && hwnd != GetForegroundWindow() {
        return;
    }
    let own = std::process::id();
    if event == EVENT_SYSTEM_FOREGROUND {
        if let Some(valid) = normalize_external(hwnd, own).or_else(|| normalize_external(GetForegroundWindow(), own)) {
            let h = valid.0 as isize;
            focus_map().get_or_insert_with(HashMap::new).insert(h, now_ms());
            EXTERNAL_FG.store(h, Ordering::Relaxed);
        }
    } else if matches!(event, EVENT_OBJECT_CLOAKED | EVENT_OBJECT_HIDE | EVENT_SYSTEM_MINIMIZESTART | EVENT_OBJECT_DESTROY) {
        if let Some(live) = normalize_external(GetForegroundWindow(), own) {
            EXTERNAL_FG.store(live.0 as isize, Ordering::Relaxed);
        }
    }
    let list: Vec<fn()> = CALLBACKS.lock().unwrap_or_else(|e| e.into_inner()).clone();
    for f in list {
        f();
    }
}

pub fn install_fx(cb: fn(u32, isize)) {
    FX_CALLBACKS.lock().unwrap_or_else(|e| e.into_inner()).push(cb);
}

pub const EV_FOREGROUND: u32 = EVENT_SYSTEM_FOREGROUND;
pub const EV_MINIMIZE_START: u32 = EVENT_SYSTEM_MINIMIZESTART;
pub const EV_MINIMIZE_END: u32 = EVENT_SYSTEM_MINIMIZEEND;
pub const EV_DESTROY: u32 = EVENT_OBJECT_DESTROY;
pub const EV_MOVESIZE_END: u32 = EVENT_SYSTEM_MOVESIZEEND;
pub const EV_LOCATION: u32 = EVENT_OBJECT_LOCATIONCHANGE;
pub const EV_SHOW: u32 = EVENT_OBJECT_SHOW;
pub const EV_CLOAKED: u32 = EVENT_OBJECT_CLOAKED;
pub const EV_UNCLOAKED: u32 = EVENT_OBJECT_UNCLOAKED;

pub fn install(on_change: fn()) {
    CALLBACKS.lock().unwrap_or_else(|e| e.into_inner()).push(on_change);
    unsafe {
        if EXTERNAL_FG.load(Ordering::Relaxed) == 0 {
            if let Some(live) = normalize_external(GetForegroundWindow(), std::process::id()) {
                EXTERNAL_FG.store(live.0 as isize, Ordering::Relaxed);
            }
        }
    }
    if HOOKED.swap(true, Ordering::Relaxed) {
        return;
    }
    for ev in [
        EVENT_OBJECT_CREATE,
        EVENT_OBJECT_DESTROY,
        EVENT_OBJECT_SHOW,
        EVENT_OBJECT_HIDE,
        EVENT_OBJECT_LOCATIONCHANGE,
        EVENT_SYSTEM_MOVESIZEEND,
        EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_MINIMIZESTART,
        EVENT_SYSTEM_MINIMIZEEND,
        EVENT_OBJECT_CLOAKED,
        EVENT_OBJECT_UNCLOAKED,
    ] {
        unsafe {
            let _ = SetWinEventHook(ev, ev, None, Some(callback), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_alt_tab_and_shell_transient_classes() {
        assert!(is_transient_shell_class("ForegroundStaging"));
        assert!(is_transient_shell_class("XamlExplorerHostIslandWindow"));
        assert!(is_transient_shell_class("MultitaskingViewFrame"));
        assert!(is_transient_shell_class("Shell_TrayWnd"));
        assert!(!is_transient_shell_class("Chrome_WidgetWin_1"));
        assert!(!is_transient_shell_class("ApplicationFrameWindow"));
        assert!(!is_transient_shell_class("Progman"));
    }
}
