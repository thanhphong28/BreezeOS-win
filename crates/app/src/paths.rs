use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("NACRE_CONFIG_DIR") {
        return PathBuf::from(d);
    }
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let dir = base.join("BreezeOS");
    migrate_legacy(&[base.join("ABRA"), base.join("Nacre")], &dir);
    dir
}

fn migrate_legacy(olds: &[PathBuf], new: &std::path::Path) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if new.exists() {
            return;
        }
        for old in olds {
            if old.exists() {
                let _ = std::fs::create_dir_all(new);
                for f in ["pins.txt", "settings.cfg"] {
                    let _ = std::fs::copy(old.join(f), new.join(f));
                }
                break;
            }
        }
    });
}

pub fn marker() -> PathBuf {
    config_dir().join("taskbar_hidden.flag")
}

pub fn log() -> PathBuf {
    config_dir().join("breezeos.log")
}

pub fn guard_exe() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("breezeos-guard.exe")))
        .unwrap_or_else(|| PathBuf::from("breezeos-guard.exe"))
}
