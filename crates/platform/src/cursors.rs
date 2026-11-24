use std::path::Path;
use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_SETCURSORS, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS};

pub fn restore(flag: &Path) {
    unsafe {
        let _ = SystemParametersInfoW(SPI_SETCURSORS, 0, None, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));
    }
    let _ = std::fs::remove_file(flag);
}

pub fn flagged(flag: &Path) -> bool {
    flag.exists()
}

pub fn recover_leftover(flag: &Path) -> bool {
    if flagged(flag) {
        restore(flag);
        true
    } else {
        false
    }
}
