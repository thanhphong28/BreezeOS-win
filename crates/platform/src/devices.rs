use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use windows::Devices::Bluetooth::{BluetoothConnectionStatus, BluetoothDevice, BluetoothLEDevice};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher};
use windows::Foundation::TypedEventHandler;
use windows_core::{IInspectable, HSTRING};

pub struct Watchers {
    list: Vec<DeviceWatcher>,
}

impl Drop for Watchers {
    fn drop(&mut self) {
        for w in &self.list {
            let _ = w.Stop();
        }
    }
}

fn watch(selector: HSTRING, report: Arc<dyn Fn(String, bool) + Send + Sync>) -> Option<DeviceWatcher> {
    let w = DeviceInformation::CreateWatcherAqsFilter(&selector).ok()?;
    let names: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let ready = Arc::new(std::sync::atomic::AtomicBool::new(false));
    {
        let (names, ready, report) = (names.clone(), ready.clone(), report.clone());
        w.Added(&TypedEventHandler::<DeviceWatcher, DeviceInformation>::new(move |_, info| {
            if let Some(info) = info.as_ref() {
                let id = info.Id().map(|h| h.to_string()).unwrap_or_default();
                let name = info.Name().map(|h| h.to_string()).unwrap_or_default();
                if name.trim().is_empty() {
                    return Ok(());
                }
                let fresh = names.lock().unwrap_or_else(|e| e.into_inner()).insert(id, name.clone()).is_none();
                if fresh && ready.load(std::sync::atomic::Ordering::Relaxed) {
                    report(name, true);
                }
            }
            Ok(())
        }))
        .ok()?;
    }
    {
        let (names, ready, report) = (names.clone(), ready.clone(), report.clone());
        w.Removed(&TypedEventHandler::<DeviceWatcher, DeviceInformationUpdate>::new(move |_, upd| {
            if let Some(u) = upd.as_ref() {
                let id = u.Id().map(|h| h.to_string()).unwrap_or_default();
                let gone = names.lock().unwrap_or_else(|e| e.into_inner()).remove(&id);
                if let (Some(name), true) = (gone, ready.load(std::sync::atomic::Ordering::Relaxed)) {
                    report(name, false);
                }
            }
            Ok(())
        }))
        .ok()?;
    }
    {
        let ready = ready.clone();
        w.EnumerationCompleted(&TypedEventHandler::<DeviceWatcher, IInspectable>::new(move |_, _| {
            ready.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        }))
        .ok()?;
    }
    w.Start().ok()?;
    Some(w)
}

pub fn watch_bluetooth(report: impl Fn(String, bool) + Send + Sync + 'static) -> Watchers {
    let report: Arc<dyn Fn(String, bool) + Send + Sync> = Arc::new(report);
    let mut list = Vec::new();
    if let Ok(sel) = BluetoothDevice::GetDeviceSelectorFromConnectionStatus(BluetoothConnectionStatus::Connected) {
        if let Some(w) = watch(sel, report.clone()) {
            list.push(w);
        }
    }
    if let Ok(sel) = BluetoothLEDevice::GetDeviceSelectorFromConnectionStatus(BluetoothConnectionStatus::Connected) {
        if let Some(w) = watch(sel, report) {
            list.push(w);
        }
    }
    crate::log::line("perangkat", &format!("pemantau Bluetooth aktif: {}", list.len()));
    Watchers { list }
}

pub fn volume_letters(lparam: isize) -> Vec<char> {
    use windows::Win32::UI::WindowsAndMessaging::{DBT_DEVTYP_VOLUME, DEV_BROADCAST_HDR, DEV_BROADCAST_VOLUME};
    if lparam == 0 {
        return vec![];
    }
    unsafe {
        let hdr = &*(lparam as *const DEV_BROADCAST_HDR);
        if hdr.dbch_devicetype != DBT_DEVTYP_VOLUME {
            return vec![];
        }
        let vol = &*(lparam as *const DEV_BROADCAST_VOLUME);
        (0..26u32).filter(|i| vol.dbcv_unitmask & (1 << i) != 0).map(|i| (b'A' + i as u8) as char).collect()
    }
}

pub fn drive_label(letter: char) -> String {
    use windows::Win32::Storage::FileSystem::GetVolumeInformationW;
    let root = HSTRING::from(format!("{letter}:\\"));
    let mut name = [0u16; 64];
    unsafe {
        if GetVolumeInformationW(&root, Some(&mut name), None, None, None, None).is_ok() {
            let n = name.iter().position(|c| *c == 0).unwrap_or(name.len());
            let s = String::from_utf16_lossy(&name[..n]).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    String::new()
}
