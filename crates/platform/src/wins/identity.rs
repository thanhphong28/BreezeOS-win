use std::collections::HashMap;
use std::sync::Mutex;
use windows::core::HSTRING;
use windows::Win32::Foundation::HWND;
use windows::Win32::Storage::EnhancedStorage::PKEY_AppUserModel_ID;
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow};
use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName};

static WEBAPP_CACHE: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);

pub fn aumid_from_store(store: &IPropertyStore) -> Option<String> {
    unsafe {
        let mut pv = store.GetValue(&PKEY_AppUserModel_ID).ok()?;
        let s = PropVariantToStringAlloc(&pv).ok().and_then(|p| {
            let text = p.to_string().ok();
            CoTaskMemFree(Some(p.0 as *const _));
            text
        });
        let _ = PropVariantClear(&mut pv);
        s.filter(|t| !t.trim().is_empty())
    }
}

pub fn aumid(hwnd: isize) -> Option<String> {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(HWND(hwnd as *mut _)).ok()?;
        aumid_from_store(&store)
    }
}

pub fn shell_path(aumid: &str) -> String {
    format!("shell:AppsFolder\\{aumid}")
}

pub fn is_browser_host(exe_file: &str) -> bool {
    matches!(
        exe_file.to_ascii_lowercase().as_str(),
        "msedge.exe"
            | "msedge_proxy.exe"
            | "msedgewebview2.exe"
            | "chrome.exe"
            | "chrome_proxy.exe"
            | "brave.exe"
            | "vivaldi.exe"
            | "opera.exe"
            | "arc.exe"
            | "thorium.exe"
            | "coccoc.exe"
            | "chromium.exe"
            | "firefox.exe"
            | "zen.exe"
            | "librewolf.exe"
    )
}

pub fn is_browser_webapp_aumid(exe_file: &str, aumid: &str) -> bool {
    if !is_browser_host(exe_file) {
        return false;
    }
    let lower = aumid.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return false;
    }
    if lower.contains("_crx_") || lower.contains("._app_") || lower.contains(".webapp.") || lower.starts_with("firefox.webapp") {
        return true;
    }
    if let Some((pkg, _)) = lower.split_once('!') {
        let self_pkg = pkg.starts_with("microsoft.microsoftedge")
            || pkg.starts_with("google.chrome")
            || pkg.starts_with("bravesoftware.brave")
            || pkg.starts_with("thebrowsercompany.arc")
            || pkg.starts_with("mozilla.firefox")
            || pkg.starts_with("operasoftware.")
            || pkg.starts_with("vivaldi.");
        return !self_pkg && !pkg.is_empty();
    }
    false
}

fn shell_item_exists(path: &str) -> bool {
    unsafe { SHCreateItemFromParsingName::<_, _, IShellItem>(&HSTRING::from(path), None).is_ok() }
}

// shortcut: uninstalled ad-hoc --app= windows without an AppsFolder entry stay grouped with the host browser until installed as an app
pub fn resolve_webapp_path(aumid: &str) -> Option<String> {
    let key = aumid.trim().to_ascii_lowercase();
    if let Some(cached) = WEBAPP_CACHE.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).get(&key).cloned() {
        return cached;
    }
    let direct = shell_path(aumid.trim());
    let resolved = if shell_item_exists(&direct) {
        Some(direct)
    } else if let Some(pos) = key.find(".userdata.") {
        let base = shell_path(&aumid.trim()[..pos]);
        shell_item_exists(&base).then_some(base)
    } else {
        None
    };
    let mut g = WEBAPP_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    if map.len() > 256 {
        map.clear();
    }
    map.insert(key, resolved.clone());
    resolved
}

pub fn browser_webapp_path(exe_file: &str, hwnd: isize) -> Option<String> {
    if !is_browser_host(exe_file) {
        return None;
    }
    let id = aumid(hwnd)?;
    if !is_browser_webapp_aumid(exe_file, &id) {
        return None;
    }
    resolve_webapp_path(&id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_browser_webapps_from_normal_browser_windows() {
        assert!(is_browser_webapp_aumid("msedge.exe", "FACEBOOK.FACEBOOK_8xx8rvfyw5nnt!App"));
        assert!(is_browser_webapp_aumid("msedge.exe", "www.youtube.com-54E21B02_pd8mbgmqs65xy!App"));
        assert!(is_browser_webapp_aumid("msedge_proxy.exe", "MSEdge._crx_kippjfofjhjlffjecoapiogbkgbpmgej"));
        assert!(is_browser_webapp_aumid("chrome.exe", "Chrome._crx_cinhiknlkffjgod.UserData.Profile1"));
        assert!(is_browser_webapp_aumid("chrome_proxy.exe", "Chrome._crx_faolnafnngnbpnkhgohbobgegn"));
        assert!(is_browser_webapp_aumid("firefox.exe", "Mozilla.Firefox.WebApp.12345"));

        assert!(!is_browser_webapp_aumid("msedge.exe", "MSEdge"));
        assert!(!is_browser_webapp_aumid("msedge.exe", "MSEdge.UserData.Default"));
        assert!(!is_browser_webapp_aumid("msedge.exe", "Microsoft.MicrosoftEdge.Stable_8wekyb3d8bbwe!App"));
        assert!(!is_browser_webapp_aumid("chrome.exe", "Chrome"));
        assert!(!is_browser_webapp_aumid("chrome.exe", "Chrome.UserData.Profile1"));
        assert!(!is_browser_webapp_aumid("arc.exe", "TheBrowserCompany.Arc_ttt1ap7aakyb4!Arc"));
        assert!(!is_browser_webapp_aumid("firefox.exe", "308046B0AF4A39CB"));
        assert!(!is_browser_webapp_aumid("SourceTree.exe", "com.squirrel.SourceTree.SourceTree"));
    }
}
