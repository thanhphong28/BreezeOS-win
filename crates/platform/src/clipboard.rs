use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW,
    RemoveClipboardFormatListener, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::WindowsAndMessaging::*;

const CF_UNICODETEXT: u32 = 13;
const CF_DIB: u32 = 8;
const MAX_IMAGES: usize = 6;
const THUMB_PX: u32 = 160;
const MAX_DIB: usize = 64 * 1024 * 1024;
const KEEP_SECS: i64 = 8 * 3600;
const MAX_ITEMS: usize = 100;
const MAX_CHARS: usize = 4000;

#[derive(Clone, Debug)]
pub struct Entry {
    pub at: i64,
    pub text: String,
    pub image: Option<(u64, u32, u32)>,
}

pub use nacre_core::clipstack::Pin;

static HISTORY: Mutex<Vec<Entry>> = Mutex::new(Vec::new());
static IMAGES: Mutex<Vec<(u64, Vec<u8>)>> = Mutex::new(Vec::new());
static THUMBS: Mutex<Vec<(u64, crate::icons::Bitmap)>> = Mutex::new(Vec::new());
static PINS: Mutex<Vec<Pin>> = Mutex::new(Vec::new());
static STORE: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

enum Clip {
    Text(String),
    Dib(Vec<u8>),
}
static ENABLED: AtomicBool = AtomicBool::new(true);
static HWND_LISTENER: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

fn now_secs() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

pub fn push(history: &mut Vec<Entry>, text: &str, now: i64) {
    let t = text.trim_end_matches(['\r', '\n', '\0']);
    if t.trim().is_empty() {
        return;
    }
    let t: String = t.chars().take(MAX_CHARS).collect();
    history.retain(|e| (e.image.is_some() || e.text != t) && now - e.at <= KEEP_SECS);
    history.insert(0, Entry { at: now, text: t, image: None });
    history.truncate(MAX_ITEMS);
}

fn push_image(history: &mut Vec<Entry>, id: u64, w: u32, h: u32, now: i64) {
    history.retain(|e| e.image.map(|i| i.0) != Some(id) && now - e.at <= KEEP_SECS);
    history.insert(0, Entry { at: now, text: String::new(), image: Some((id, w, h)) });
    history.truncate(MAX_ITEMS);
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn clip_dir() -> Option<std::path::PathBuf> {
    lock(&STORE).as_ref().map(|d| d.join("clip"))
}

fn pins_file() -> Option<std::path::PathBuf> {
    lock(&STORE).as_ref().map(|d| d.join("clip_pins.txt"))
}

fn save_pins() {
    let Some(f) = pins_file() else { return };
    let text = nacre_core::clipstack::pins_text(&lock(&PINS));
    if let Some(dir) = f.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(f, text);
}

pub fn set_store(dir: std::path::PathBuf) {
    *lock(&STORE) = Some(dir);
    let pins = pins_file().and_then(|f| std::fs::read_to_string(f).ok()).map(|t| nacre_core::clipstack::parse_pins(&t)).unwrap_or_default();
    *lock(&PINS) = pins;
}

pub fn pins() -> Vec<Pin> {
    lock(&PINS).clone()
}

pub fn is_pinned(e: &Entry) -> bool {
    let p = lock(&PINS);
    match e.image {
        Some((id, _, _)) => p.iter().any(|x| matches!(x, Pin::Image { id: i, .. } if *i == id)),
        None => p.iter().any(|x| matches!(x, Pin::Text(t) if *t == e.text)),
    }
}

fn dib_bytes(id: u64) -> Option<Vec<u8>> {
    if let Some(b) = lock(&IMAGES).iter().find(|(i, _)| *i == id).map(|(_, b)| b.clone()) {
        return Some(b);
    }
    std::fs::read(clip_dir()?.join(format!("{id}.dib"))).ok()
}

pub fn toggle_pin(e: &Entry) -> bool {
    let pinned = is_pinned(e);
    {
        let mut p = lock(&PINS);
        match e.image {
            Some((id, w, h)) => {
                if pinned {
                    p.retain(|x| !matches!(x, Pin::Image { id: i, .. } if *i == id));
                    if let Some(d) = clip_dir() {
                        let _ = std::fs::remove_file(d.join(format!("{id}.dib")));
                    }
                } else {
                    let Some(bytes) = lock(&IMAGES).iter().find(|(i, _)| *i == id).map(|(_, b)| b.clone()) else { return false };
                    let Some(d) = clip_dir() else { return false };
                    let _ = std::fs::create_dir_all(&d);
                    if std::fs::write(d.join(format!("{id}.dib")), bytes).is_err() {
                        return false;
                    }
                    p.insert(0, Pin::Image { id, w, h });
                }
            }
            None => {
                if pinned {
                    p.retain(|x| !matches!(x, Pin::Text(t) if *t == e.text));
                } else {
                    p.insert(0, Pin::Text(e.text.clone()));
                }
            }
        }
        p.truncate(nacre_core::clipstack::MAX_PINS);
    }
    save_pins();
    !pinned
}

pub fn thumb(id: u64) -> Option<crate::icons::Bitmap> {
    if let Some(b) = lock(&THUMBS).iter().find(|(i, _)| *i == id).map(|(_, b)| b.clone()) {
        return Some(b);
    }
    let bytes = dib_bytes(id)?;
    let (w, h, px) = nacre_core::clipstack::dib_to_bgra(&bytes)?;
    let (tw, th, t) = nacre_core::clipstack::thumbnail(&px, w, h, THUMB_PX);
    let b = crate::icons::Bitmap { w: tw, h: th, bgra: t };
    remember_thumb(id, b.clone());
    Some(b)
}

fn remember_thumb(id: u64, b: crate::icons::Bitmap) {
    let mut t = lock(&THUMBS);
    t.retain(|(i, _)| *i != id);
    t.insert(0, (id, b));
    t.truncate(40);
}

fn store_image(bytes: Vec<u8>, now: i64) {
    let id = nacre_core::clipstack::fingerprint(&bytes);
    let Some((w, h, px)) = nacre_core::clipstack::dib_to_bgra(&bytes) else { return };
    let (tw, th, t) = nacre_core::clipstack::thumbnail(&px, w, h, THUMB_PX);
    remember_thumb(id, crate::icons::Bitmap { w: tw, h: th, bgra: t });
    let dropped: Vec<u64> = {
        let mut imgs = lock(&IMAGES);
        imgs.retain(|(i, _)| *i != id);
        imgs.insert(0, (id, bytes));
        let keep = MAX_IMAGES.min(imgs.len());
        imgs.split_off(keep).into_iter().map(|(i, _)| i).collect()
    };
    let mut hist = lock(&HISTORY);
    hist.retain(|e| e.image.is_none_or(|(i, _, _)| !dropped.contains(&i)));
    push_image(&mut hist, id, w, h, now);
}

pub fn copy_image(id: u64) -> bool {
    let Some(bytes) = dib_bytes(id) else { return false };
    unsafe {
        let hwnd = HWND(HWND_LISTENER.load(Ordering::Relaxed) as *mut _);
        for _ in 0..6 {
            if OpenClipboard(Some(hwnd)).is_ok() {
                let ok = (|| {
                    let _ = EmptyClipboard();
                    let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len()).ok()?;
                    let p = GlobalLock(h) as *mut u8;
                    if p.is_null() {
                        return None;
                    }
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
                    let _ = GlobalUnlock(h);
                    SetClipboardData(CF_DIB, Some(windows::Win32::Foundation::HANDLE(h.0))).ok()?;
                    Some(())
                })()
                .is_some();
                let _ = CloseClipboard();
                return ok;
            }
            std::thread::sleep(std::time::Duration::from_millis(12));
        }
    }
    false
}

pub fn history() -> Vec<Entry> {
    let now = now_secs();
    let mut g = HISTORY.lock().unwrap();
    g.retain(|e| now - e.at <= KEEP_SECS);
    g.clone()
}

pub fn clear() {
    HISTORY.lock().unwrap().clear();
    lock(&IMAGES).clear();
    let pinned: Vec<u64> = lock(&PINS).iter().filter_map(|p| if let Pin::Image { id, .. } = p { Some(*id) } else { None }).collect();
    lock(&THUMBS).retain(|(i, _)| pinned.contains(i));
}

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
    if !on {
        clear();
    }
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

fn opted_out(exclude: bool, can_include: Option<&[u8]>) -> bool {
    exclude || can_include.is_some_and(|b| b.len() < 4 || b[..4] == [0; 4])
}

#[cfg(test)]
unsafe fn read_text(hwnd: HWND) -> Option<String> {
    match read_clip(hwnd) {
        Some(Clip::Text(t)) => Some(t),
        _ => None,
    }
}

unsafe fn read_dib() -> Option<Vec<u8>> {
    if IsClipboardFormatAvailable(CF_DIB).is_err() {
        return None;
    }
    let h = GetClipboardData(CF_DIB).ok()?;
    let g = windows::Win32::Foundation::HGLOBAL(h.0);
    let n = GlobalSize(g);
    if n == 0 || n > MAX_DIB {
        return None;
    }
    let p = GlobalLock(g) as *const u8;
    if p.is_null() {
        return None;
    }
    let v = std::slice::from_raw_parts(p, n).to_vec();
    let _ = GlobalUnlock(g);
    Some(v)
}

unsafe fn read_clip(hwnd: HWND) -> Option<Clip> {
    for _ in 0..6 {
        if OpenClipboard(Some(hwnd)).is_ok() {
            let r = (|| {
                let has = |name| {
                    let f = RegisterClipboardFormatW(name);
                    (f != 0 && IsClipboardFormatAvailable(f).is_ok()).then_some(f)
                };
                let can = has(w!("CanIncludeInClipboardHistory")).map(|f| {
                    let Ok(h) = GetClipboardData(f) else { return Vec::new() };
                    let g = windows::Win32::Foundation::HGLOBAL(h.0);
                    let p = GlobalLock(g) as *const u8;
                    if p.is_null() {
                        return Vec::new();
                    }
                    let v = std::slice::from_raw_parts(p, GlobalSize(g).min(4)).to_vec();
                    let _ = GlobalUnlock(g);
                    v
                });
                if opted_out(has(w!("ExcludeClipboardContentFromMonitorProcessing")).is_some(), can.as_deref()) {
                    return None;
                }
                if IsClipboardFormatAvailable(CF_UNICODETEXT).is_err() {
                    return read_dib().map(Clip::Dib);
                }
                let h = GetClipboardData(CF_UNICODETEXT).ok()?;
                let p = GlobalLock(windows::Win32::Foundation::HGLOBAL(h.0)) as *const u16;
                if p.is_null() {
                    return None;
                }
                let mut n = 0usize;
                while *p.add(n) != 0 && n < 200_000 {
                    n += 1;
                }
                let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, n));
                let _ = GlobalUnlock(windows::Win32::Foundation::HGLOBAL(h.0));
                Some(Clip::Text(s))
            })();
            let _ = CloseClipboard();
            return r;
        }
        std::thread::sleep(std::time::Duration::from_millis(12));
    }
    None
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        if ENABLED.load(Ordering::Relaxed) {
            match read_clip(hwnd) {
                Some(Clip::Text(t)) => push(&mut HISTORY.lock().unwrap(), &t, now_secs()),
                Some(Clip::Dib(b)) => store_image(b, now_secs()),
                None => {}
            }
        }
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

pub fn init() -> windows_core::Result<()> {
    unsafe {
        let hi = GetModuleHandleW(None)?.into();
        let class = w!("BreezeOSClipboard");
        RegisterClassW(&WNDCLASSW { lpfnWndProc: Some(proc), hInstance: hi, lpszClassName: class, ..Default::default() });
        let hwnd = CreateWindowExW(WINDOW_EX_STYLE(0), class, w!("breezeos-clip"), WS_POPUP, 0, 0, 0, 0, Some(HWND_MESSAGE), None, Some(hi), None)?;
        AddClipboardFormatListener(hwnd)?;
        HWND_LISTENER.store(hwnd.0 as isize, Ordering::Relaxed);
    }
    Ok(())
}

pub fn shutdown() {
    let h = HWND_LISTENER.swap(0, Ordering::Relaxed);
    if h != 0 {
        unsafe {
            let _ = RemoveClipboardFormatListener(HWND(h as *mut _));
        }
    }
}

pub fn copy_text(text: &str) -> bool {
    unsafe {
        let hwnd = HWND(HWND_LISTENER.load(Ordering::Relaxed) as *mut _);
        for _ in 0..6 {
            if OpenClipboard(Some(hwnd)).is_ok() {
                let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                let ok = (|| {
                    let _ = EmptyClipboard();
                    let h = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2).ok()?;
                    let p = GlobalLock(h) as *mut u16;
                    if p.is_null() {
                        return None;
                    }
                    std::ptr::copy_nonoverlapping(wide.as_ptr(), p, wide.len());
                    let _ = GlobalUnlock(h);
                    SetClipboardData(CF_UNICODETEXT, Some(windows::Win32::Foundation::HANDLE(h.0))).ok()?;
                    Some(())
                })()
                .is_some();
                let _ = CloseClipboard();
                return ok;
            }
            std::thread::sleep(std::time::Duration::from_millis(12));
        }
        false
    }
}

#[allow(dead_code)]
fn _unused(_: PCWSTR) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn penanda_sensitif_diabaikan() {
        assert!(!opted_out(false, None), "tanpa penanda: simpan");
        assert!(opted_out(true, None));
        assert!(opted_out(false, Some(&0u32.to_le_bytes())), "CanIncludeInClipboardHistory=0");
        assert!(!opted_out(false, Some(&1u32.to_le_bytes())), "CanIncludeInClipboardHistory=1");
        assert!(opted_out(false, Some(&[1, 0])), "data rusak dianggap jangan");
        assert!(opted_out(true, Some(&1u32.to_le_bytes())), "Exclude menang");
    }

    unsafe fn put(text: &str, can_include: Option<u32>) {
        assert!(OpenClipboard(None).is_ok());
        let _ = EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let h = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2).unwrap();
        std::ptr::copy_nonoverlapping(wide.as_ptr(), GlobalLock(h) as *mut u16, wide.len());
        let _ = GlobalUnlock(h);
        SetClipboardData(CF_UNICODETEXT, Some(windows::Win32::Foundation::HANDLE(h.0))).unwrap();
        if let Some(v) = can_include {
            let h = GlobalAlloc(GMEM_MOVEABLE, 4).unwrap();
            std::ptr::copy_nonoverlapping(v.to_le_bytes().as_ptr(), GlobalLock(h) as *mut u8, 4);
            let _ = GlobalUnlock(h);
            SetClipboardData(RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory")), Some(windows::Win32::Foundation::HANDLE(h.0))).unwrap();
        }
        let _ = CloseClipboard();
    }

    #[test]
    #[ignore = "menimpa papan klip sistem; jalankan manual: cargo test -p nacre-platform clipboard -- --ignored"]
    fn papan_klip_asli_menghormati_can_include() {
        unsafe {
            let lama = read_text(HWND::default());
            put("rahasia-uji", Some(0));
            assert_eq!(read_text(HWND::default()), None);
            put("boleh-uji", Some(1));
            assert_eq!(read_text(HWND::default()).as_deref(), Some("boleh-uji"));
            put(lama.as_deref().unwrap_or(""), None);
        }
    }

    #[test]
    fn push_dedups_trims_old_and_caps() {
        let mut h = Vec::new();
        push(&mut h, "a", 1000);
        push(&mut h, "b\r\n", 1001);
        push(&mut h, "a", 1002);
        assert_eq!(h.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        push(&mut h, "   ", 1003);
        assert_eq!(h.len(), 2, "kosong diabaikan");
        push(&mut h, "baru", 1000 + KEEP_SECS + 10);
        assert_eq!(h.len(), 1, "yang lebih tua dari 8 jam dibuang");
        for i in 0..150 {
            push(&mut h, &format!("x{i}"), 5_000_000 + i);
        }
        assert_eq!(h.len(), MAX_ITEMS);
        let long = "z".repeat(10_000);
        push(&mut h, &long, 6_000_000);
        assert_eq!(h[0].text.chars().count(), MAX_CHARS);
    }
}
