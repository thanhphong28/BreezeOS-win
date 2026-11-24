use std::path::Path;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ};

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const NAME: PCWSTR = w!("BreezeOS");

pub const TASK: &str = "BreezeOS Autostart";

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn task_xml(exe: &Path, user: &str) -> String {
    let (cmd, user) = (escape(&exe.display().to_string()), escape(user));
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-16\"?>\n<Task version=\"1.2\" xmlns=\"http://schemas.microsoft.com/windows/2004/02/mit/task\">\n  <Triggers>\n    <LogonTrigger>\n      <Enabled>true</Enabled>\n      <UserId>{user}</UserId>\n    </LogonTrigger>\n  </Triggers>\n  <Principals>\n    <Principal id=\"Author\">\n      <UserId>{user}</UserId>\n      <LogonType>InteractiveToken</LogonType>\n      <RunLevel>LeastPrivilege</RunLevel>\n    </Principal>\n  </Principals>\n  <Settings>\n    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>\n    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>\n    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>\n    <StartWhenAvailable>true</StartWhenAvailable>\n    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>\n    <Priority>5</Priority>\n  </Settings>\n  <Actions Context=\"Author\">\n    <Exec>\n      <Command>{cmd}</Command>\n    </Exec>\n  </Actions>\n</Task>\n"
    )
}

fn schtasks(args: &[&str]) -> Option<std::process::Output> {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("schtasks.exe").args(args).creation_flags(0x0800_0000).output().ok()
}

fn current_user() -> Option<String> {
    let name = std::env::var("USERNAME").ok()?;
    Some(match std::env::var("USERDOMAIN") {
        Ok(d) if !d.is_empty() => format!("{d}\\{name}"),
        _ => name,
    })
}

pub fn create_task(name: &str, exe: &Path) -> Result<(), String> {
    let user = current_user().ok_or("nama pengguna tidak diketahui")?;
    let xml = task_xml(exe, &user);
    let mut bytes = vec![0xFF, 0xFE];
    for u in xml.encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    let file = std::env::temp_dir().join(format!("breezeos-task-{}.xml", std::process::id()));
    std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
    let out = schtasks(&["/Create", "/TN", name, "/XML", &file.to_string_lossy(), "/F"]);
    let _ = std::fs::remove_file(&file);
    match out {
        Some(o) if o.status.success() => Ok(()),
        Some(o) => Err(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        None => Err("schtasks tidak bisa dijalankan".into()),
    }
}

pub fn delete_task(name: &str) {
    let _ = schtasks(&["/Delete", "/TN", name, "/F"]);
}

pub fn task_command(name: &str) -> Option<String> {
    let o = schtasks(&["/Query", "/TN", name, "/XML"])?;
    if !o.status.success() {
        return None;
    }
    let text = if o.stdout.len() >= 2 && o.stdout[0] == 0xFF && o.stdout[1] == 0xFE {
        let u: Vec<u16> = o.stdout[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&u)
    } else {
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let a = text.find("<Command>")? + "<Command>".len();
    let b = text[a..].find("</Command>")? + a;
    let raw = text[a..b].replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"");
    Some(format!("\"{}\"", raw.trim_matches('"')))
}

pub fn current() -> Option<String> {
    task_command(TASK).or_else(run_value)
}

fn run_value() -> Option<String> {
    unsafe {
        let mut size = 0u32;
        if RegGetValueW(HKEY_CURRENT_USER, RUN_KEY, NAME, RRF_RT_REG_SZ, None, None, Some(&mut size)).is_err() || size == 0 {
            return None;
        }
        let mut buf = vec![0u16; (size as usize).div_ceil(2)];
        if RegGetValueW(HKEY_CURRENT_USER, RUN_KEY, NAME, RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr() as *mut _), Some(&mut size)).is_err() {
            return None;
        }
        let n = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..n]))
    }
}

pub fn is_enabled() -> bool {
    current().is_some()
}

pub fn command_for(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

pub fn set(enabled: bool, exe: &Path) -> Result<(), String> {
    if enabled {
        match create_task(TASK, exe) {
            Ok(()) => {
                remove_run_value();
                return Ok(());
            }
            Err(e) => crate::log::line("startup", &format!("tugas login gagal ({e}); memakai kunci Run"))
        }
    } else {
        delete_task(TASK);
    }
    set_run_value(enabled, exe)
}

pub fn run_value_absent() -> bool {
    run_value().is_none()
}

fn remove_run_value() {
    unsafe {
        let _ = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, NAME);
    }
}

fn set_run_value(enabled: bool, exe: &Path) -> Result<(), String> {
    unsafe {
        if enabled {
            let cmd = command_for(exe);
            let wide: Vec<u16> = cmd.encode_utf16().chain(std::iter::once(0)).collect();
            let err = RegSetKeyValueW(HKEY_CURRENT_USER, RUN_KEY, NAME, REG_SZ.0, Some(wide.as_ptr() as *const _), (wide.len() * 2) as u32);
            if err.is_err() {
                return Err(format!("RegSetKeyValue: {err:?}"));
            }
        } else {
            let err = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, NAME);
            if err.is_err() && err != ERROR_FILE_NOT_FOUND {
                return Err(format!("RegDeleteKeyValue: {err:?}"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_xml_is_well_formed_and_escapes() {
        let x = task_xml(Path::new("C:\\Apps\\A&B\\breezeos.exe"), "PC\\Budi <x>");
        assert!(x.contains("<LogonTrigger>") && x.contains("<UserId>PC\\Budi &lt;x&gt;</UserId>"));
        assert!(x.contains("<Command>C:\\Apps\\A&amp;B\\breezeos.exe</Command>"));
        assert!(x.contains("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"));
        assert!(x.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"));
        assert!(x.contains("<RunLevel>LeastPrivilege</RunLevel>"));
    }

    #[test]
    fn logon_task_round_trip_on_this_machine() {
        if std::env::var_os("NACRE_TEST_TASK").is_none() {
            return;
        }
        let name = format!("BreezeOS-uji-{}", std::process::id());
        let exe = std::path::PathBuf::from(r"C:\Windows\System32\hostname.exe");
        create_task(&name, &exe).expect("buat tugas");
        let got = task_command(&name);
        delete_task(&name);
        assert_eq!(got.as_deref(), Some(command_for(&exe).as_str()));
        assert!(task_command(&name).is_none(), "tugas harus terhapus");
    }
}
