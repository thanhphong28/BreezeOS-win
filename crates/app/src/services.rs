use nacre_platform::{audio, brightness};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::OnceLock;
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP};

pub const WM_VOLUME: u32 = WM_APP + 20;
pub const WM_BRIGHT: u32 = WM_APP + 21;
pub const WM_RADIO: u32 = WM_APP + 22;

static HUD: AtomicIsize = AtomicIsize::new(0);
static ISLAND: AtomicIsize = AtomicIsize::new(0);
static AUDIO: OnceLock<Sender<audio::Cmd>> = OnceLock::new();
static BRIGHT: OnceLock<Sender<brightness::Cmd>> = OnceLock::new();

pub fn register_hud(h: HWND) {
    HUD.store(h.0 as isize, Ordering::Relaxed);
}

pub fn register_island(h: HWND) {
    ISLAND.store(h.0 as isize, Ordering::Relaxed);
}

fn post_all(msg: u32) {
    for h in [HUD.load(Ordering::Relaxed), ISLAND.load(Ordering::Relaxed)] {
        if h != 0 {
            unsafe {
                let _ = PostMessageW(Some(HWND(h as *mut _)), msg, WPARAM(0), LPARAM(0));
            }
        }
    }
}

pub fn init() {
    if AUDIO.get().is_some() {
        return;
    }
    let a = audio::spawn(|| post_all(WM_VOLUME));
    let _ = AUDIO.set(a.commands);
    let b = brightness::spawn(|| post_all(WM_BRIGHT));
    let _ = BRIGHT.set(b.commands);
}

pub fn set_volume(v: f32) {
    if let Some(tx) = AUDIO.get() {
        let _ = tx.send(audio::Cmd::SetLevel(v));
    }
}

pub fn set_muted(on: bool) {
    if let Some(tx) = AUDIO.get() {
        let _ = tx.send(audio::Cmd::SetMuted(on));
    }
}

pub fn toggle_mute() {
    if let Some(tx) = AUDIO.get() {
        let _ = tx.send(audio::Cmd::ToggleMute);
    }
}

pub fn set_brightness(frac: f32) {
    if let Some(tx) = BRIGHT.get() {
        let _ = tx.send(brightness::Cmd::Set((frac.clamp(0.0, 1.0) * 100.0).round() as u8));
    }
}

static NOTCHES: std::sync::Mutex<Vec<(isize, (i32, i32, i32, i32))>> = std::sync::Mutex::new(Vec::new());

// One entry per Breeze Notch window (id = its hwnd); l == r == 0 removes the entry.
pub fn set_notch(id: isize, l: i32, r: i32, h: i32, top: i32) {
    if let Ok(mut g) = NOTCHES.lock() {
        g.retain(|e| e.0 != id);
        if r > l && h > 0 {
            g.push((id, (l, r, h, top)));
        }
    }
    let bars: Vec<isize> = BARS.lock().map(|g| g.clone()).unwrap_or_default();
    for bar in bars {
        unsafe {
            let _ = PostMessageW(Some(HWND(bar as *mut _)), WM_APP + 71, WPARAM(0), LPARAM(0));
        }
    }
}

pub fn notch_on(area: nacre_core::display::Area) -> Option<(i32, i32, i32)> {
    let g = NOTCHES.lock().ok()?;
    g.iter().map(|e| e.1).find(|n| n.3 == area.y && n.0 >= area.x && n.1 <= area.right()).map(|n| (n.0 - area.x, n.1 - area.x, n.2))
}

static RAIL_AREA: std::sync::Mutex<Option<nacre_core::display::Area>> = std::sync::Mutex::new(None);

pub fn set_rail_area(a: Option<nacre_core::display::Area>) {
    if let Ok(mut g) = RAIL_AREA.lock() {
        *g = a;
    }
}

pub fn rail_area() -> Option<nacre_core::display::Area> {
    RAIL_AREA.lock().ok().and_then(|g| *g)
}

static BARS: std::sync::Mutex<Vec<isize>> = std::sync::Mutex::new(Vec::new());

pub fn register_bar(h: HWND) {
    if let Ok(mut g) = BARS.lock() {
        if !g.contains(&(h.0 as isize)) {
            g.push(h.0 as isize);
        }
    }
}

pub fn unregister_bar(h: HWND) {
    if let Ok(mut g) = BARS.lock() {
        g.retain(|b| *b != h.0 as isize);
    }
}
pub fn post_island(msg: u32) {
    let h = ISLAND.load(Ordering::Relaxed);
    if h != 0 {
        unsafe {
            let _ = PostMessageW(Some(HWND(h as *mut _)), msg, WPARAM(0), LPARAM(0));
        }
    }
}
