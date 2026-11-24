use windows::core::w;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_REMOTESESSION};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conditions {
    pub transparency: bool,
    pub battery_saver: bool,
    pub remote_session: bool,
}

pub fn conditions() -> Conditions {
    let mut v = 1u32;
    let mut size = 4u32;
    let transparency = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("EnableTransparency"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut v as *mut u32 as *mut _),
            Some(&mut size),
        )
        .is_err()
            || v != 0
    };
    let mut ps = SYSTEM_POWER_STATUS::default();
    let battery_saver = unsafe { GetSystemPowerStatus(&mut ps).is_ok() && ps.SystemStatusFlag & 1 != 0 };
    let remote_session = unsafe { GetSystemMetrics(SM_REMOTESESSION) != 0 };
    Conditions { transparency, battery_saver, remote_session }
}

fn fake_battery() -> Option<(u8, bool)> {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = *START.get_or_init(std::time::Instant::now);
    let base = std::env::var("NACRE_FAKE_BATTERY").ok()?.parse::<u8>().ok()?.min(100);
    let drop = (start.elapsed().as_secs() / 60) as u8;
    Some((base.saturating_sub(drop), false))
}

pub fn battery() -> Option<(u8, bool)> {
    if let Some(f) = fake_battery() {
        return Some(f);
    }
    let mut ps = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut ps).ok()? };
    if ps.BatteryFlag & 128 != 0 || ps.BatteryLifePercent > 100 {
        return None;
    }
    Some((ps.BatteryLifePercent, ps.BatteryFlag & 8 != 0 || ps.ACLineStatus == 1))
}

pub fn accent_color() -> Option<[u8; 3]> {
    let mut v: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;
    unsafe {
        RegGetValueW(HKEY_CURRENT_USER, w!("Software\\Microsoft\\Windows\\DWM"), w!("AccentColor"), RRF_RT_REG_DWORD, None, Some(&mut v as *mut u32 as *mut _), Some(&mut size)).ok().ok()?;
    }
    Some([(v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, ((v >> 16) & 0xFF) as u8])
}

pub fn screen_size() -> (i32, i32) {
    if let Some(p) = std::env::var_os("NACRE_SCREEN_FILE") {
        if let Ok(text) = std::fs::read_to_string(p) {
            if let Some((w, h)) = text.trim().split_once('x') {
                if let (Ok(w), Ok(h)) = (w.parse(), h.parse()) {
                    return (w, h);
                }
            }
        }
    }
    unsafe { (GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN), GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN)) }
}

pub fn apps_dark() -> bool {
    let mut v = 1u32;
    let mut size = 4u32;
    let ok = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut v as *mut u32 as *mut _),
            Some(&mut size),
        )
        .is_ok()
    };
    ok && v == 0
}

pub fn screen_luma(points: &[(i32, i32)]) -> Option<f32> {
    use windows::Win32::Graphics::Gdi::{GetDC, GetPixel, ReleaseDC};
    unsafe {
        let dc = GetDC(None);
        if dc.is_invalid() {
            return None;
        }
        let (mut sum, mut n) = (0.0f32, 0u32);
        for (x, y) in points {
            let c = GetPixel(dc, *x, *y).0;
            if c == 0xFFFF_FFFF {
                continue;
            }
            let (r, g, b) = ((c & 0xFF) as f32, ((c >> 8) & 0xFF) as f32, ((c >> 16) & 0xFF) as f32);
            sum += (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0;
            n += 1;
        }
        ReleaseDC(None, dc);
        (n > 0).then(|| sum / n as f32)
    }
}

pub fn cpu_times() -> Option<(u64, u64)> {
    use windows::Win32::Foundation::FILETIME;
    let (mut idle, mut kernel, mut user) = (FILETIME::default(), FILETIME::default(), FILETIME::default());
    unsafe { windows::Win32::System::Threading::GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).ok()? };
    let v = |f: FILETIME| ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64;
    Some((v(idle), v(kernel) + v(user)))
}

pub fn memory_load() -> Option<u8> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut m = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
    unsafe { GlobalMemoryStatusEx(&mut m).ok()? };
    Some(m.dwMemoryLoad.min(100) as u8)
}

pub fn utc_offset_min() -> i32 {
    use windows::Win32::System::Time::{GetTimeZoneInformation, TIME_ZONE_INFORMATION};
    let mut tz = TIME_ZONE_INFORMATION::default();
    let r = unsafe { GetTimeZoneInformation(&mut tz) };
    let dst = if r == 2 { tz.DaylightBias } else { tz.StandardBias };
    -(tz.Bias + dst)
}

static TRIM_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn trim_memory() {
    use windows::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
    unsafe {
        let _ = SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

pub fn trim_later(ms: u64) {
    use std::sync::atomic::Ordering;
    let gen = TRIM_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    let _ = std::thread::Builder::new().name("breezeos-trim".into()).spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(ms));
        if TRIM_GEN.load(Ordering::Relaxed) == gen {
            trim_memory();
        }
    });
}
