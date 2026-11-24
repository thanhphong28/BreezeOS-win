use nacre_core::notch::lrclib::{self, Track};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, SystemTime};

const NONE_TTL: Duration = Duration::from_secs(3 * 24 * 3600);
const TIMEOUT: Duration = Duration::from_secs(8);

pub struct Request {
    pub track: Track,
    pub online: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Local,
    Lrclib,
    None,
}

pub struct Answer {
    pub key: String,
    pub text: Option<String>,
    pub source: Source,
}

pub struct Lyrics {
    pub requests: Sender<Request>,
    pub answers: Receiver<Answer>,
}

fn file_safe(s: &str) -> String {
    s.chars().map(|c| if r#"\/:*?"<>|"#.contains(c) || c.is_control() { '_' } else { c }).collect::<String>().trim().to_string()
}

fn local_file(dir: &Path, t: &Track) -> Option<String> {
    let mut want = Vec::new();
    if !t.artist.is_empty() {
        want.push(format!("{} - {}.lrc", file_safe(&t.artist), file_safe(&t.title)).to_lowercase());
    }
    want.push(format!("{}.lrc", file_safe(&t.title)).to_lowercase());
    let names: Vec<PathBuf> = std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).collect();
    for w in &want {
        if let Some(p) = names.iter().find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.to_lowercase() == *w)) {
            if let Ok(s) = std::fs::read_to_string(p) {
                return Some(s);
            }
        }
    }
    None
}

fn http_get(url: &str) -> Option<(i32, String)> {
    crate::net::http_get(url, TIMEOUT)
}

fn ask_lrclib(t: &Track) -> Result<Option<lrclib::Found>, ()> {
    if let Some(url) = lrclib::get_url(t) {
        match http_get(&url) {
            Some((200, body)) => {
                if let Some(f) = lrclib::parse_get(&body) {
                    return Ok(Some(f));
                }
            }
            Some((404, _)) => {}
            Some(_) | None => return Err(()),
        }
    }
    let Some(url) = lrclib::search_url(t) else { return Ok(None) };
    match http_get(&url) {
        Some((200, body)) => Ok(lrclib::pick_search(&body, t.duration_s)),
        Some((404, _)) => Ok(None),
        _ => Err(()),
    }
}

fn lookup(dir: &Path, req: &Request) -> Answer {
    let t = &req.track;
    let key = lrclib::cache_key(t);
    if let Some(s) = local_file(dir, t) {
        return Answer { key, text: Some(s), source: Source::Local };
    }
    let cache = dir.join("cache");
    let hit = cache.join(format!("{key}.lrc"));
    if let Ok(s) = std::fs::read_to_string(&hit) {
        return Answer { key, text: Some(s), source: Source::Lrclib };
    }
    let none = cache.join(format!("{key}.none"));
    let fresh_none = std::fs::metadata(&none)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < NONE_TTL);
    if fresh_none || !req.online {
        return Answer { key, text: None, source: Source::None };
    }
    match ask_lrclib(t) {
        Ok(Some(f)) => {
            let text = f.text();
            let _ = std::fs::create_dir_all(&cache);
            let _ = std::fs::write(&hit, &text);
            crate::log::line("lirik", &format!("LRCLIB: ditemukan ({} byte)", text.len()));
            Answer { key, text: Some(text), source: Source::Lrclib }
        }
        Ok(None) => {
            let _ = std::fs::create_dir_all(&cache);
            let _ = std::fs::write(&none, b"");
            crate::log::line("lirik", "LRCLIB: tidak ada");
            Answer { key, text: None, source: Source::None }
        }
        Err(()) => {
            crate::log::line("lirik", "LRCLIB: gagal (jaringan/batas waktu)");
            Answer { key, text: None, source: Source::None }
        }
    }
}

pub fn spawn(dir: PathBuf, notify: impl Fn() + Send + 'static) -> Lyrics {
    let (req_tx, req_rx) = channel::<Request>();
    let (ans_tx, ans_rx) = channel::<Answer>();
    let _ = std::fs::create_dir_all(&dir);
    std::thread::spawn(move || {
        let com = unsafe { windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_MULTITHREADED).is_ok() };
        while let Ok(mut req) = req_rx.recv() {
            while let Ok(newer) = req_rx.try_recv() {
                req = newer;
            }
            let a = lookup(&dir, &req);
            if ans_tx.send(a).is_err() {
                break;
            }
            notify();
        }
        if com {
            unsafe { windows::Win32::System::Com::CoUninitialize() };
        }
    });
    Lyrics { requests: req_tx, answers: ans_rx }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_lrc_is_found_case_insensitively_and_names_are_file_safe() {
        let dir = std::env::temp_dir().join(format!("breezeos-lyrics-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("coldplay - clocks.LRC"), "[00:01.00]Lights go out").unwrap();
        let t = Track { title: "Clocks".into(), artist: "Coldplay".into(), album: String::new(), duration_s: None };
        assert_eq!(local_file(&dir, &t).as_deref(), Some("[00:01.00]Lights go out"));
        let other = Track { title: "AC/DC: T.N.T?".into(), ..t.clone() };
        assert_eq!(local_file(&dir, &other), None);
        assert_eq!(file_safe("AC/DC: T.N.T?"), "AC_DC_ T.N.T_");
        let a = lookup(&dir, &Request { track: t, online: false });
        assert_eq!(a.source, Source::Local);
        let none = lookup(&dir, &Request { track: other, online: false });
        assert_eq!((none.text, none.source), (None, Source::None), "luring: tidak menyentuh jaringan");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
