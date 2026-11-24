use nacre_core::community::{self, Reject, Spec};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_FILES: usize = 30;
const FRESH: Duration = Duration::from_secs(3);

#[derive(Clone, Debug)]
pub enum Status {
    Valid,
    Partial(usize),
    Rejected(Reject),
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub author: String,
    pub status: Status,
    spec: Option<Spec>,
}

static CACHE: Mutex<Option<(Instant, Vec<Entry>)>> = Mutex::new(None);
static NOTICE: Mutex<Option<(String, Reject)>> = Mutex::new(None);

pub fn dir() -> PathBuf {
    let d = crate::paths::config_dir().join("themes");
    let _ = std::fs::create_dir_all(&d);
    d
}

fn read(path: &Path) -> Result<community::Parsed, Reject> {
    let len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if len > community::MAX_BYTES as u64 {
        return Err(Reject::TooBig);
    }
    let bytes = std::fs::read(path).map_err(|_| Reject::NotText)?;
    community::parse(&bytes)
}

fn entry(path: &Path) -> Entry {
    let stem = path.file_stem().map(|s| s.to_string_lossy().chars().take(40).collect::<String>()).unwrap_or_default();
    match read(path) {
        Ok(p) => Entry {
            name: if p.spec.name.is_empty() { stem } else { p.spec.name.clone() },
            author: p.spec.author.clone(),
            status: if p.ignored.is_empty() { Status::Valid } else { Status::Partial(p.ignored.len()) },
            spec: Some(p.spec),
        },
        Err(r) => Entry { name: stem, author: String::new(), status: Status::Rejected(r), spec: None },
    }
}

fn is_theme(p: &Path) -> bool {
    p.is_file() && p.extension().is_some_and(|e| {
        let ext = e.to_string_lossy();
        ext.eq_ignore_ascii_case(community::EXT) || ext.eq_ignore_ascii_case("abratheme")
    })
}

fn scan() -> Vec<Entry> {
    let Ok(rd) = std::fs::read_dir(dir()) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| is_theme(p)).collect();
    paths.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
    paths.truncate(MAX_FILES);
    paths.iter().map(|p| entry(p)).collect()
}

fn invalidate() {
    if let Ok(mut c) = CACHE.lock() {
        *c = None;
    }
}

pub fn list() -> Vec<Entry> {
    let Ok(mut c) = CACHE.lock() else { return Vec::new() };
    if let Some((at, v)) = c.as_ref() {
        if at.elapsed() < FRESH {
            return v.clone();
        }
    }
    let v = scan();
    *c = Some((Instant::now(), v.clone()));
    v
}

pub fn notice() -> Option<(String, Reject)> {
    NOTICE.lock().ok().and_then(|n| n.clone())
}

pub fn apply(i: usize) {
    if let Some(spec) = list().get(i).and_then(|e| e.spec.clone()) {
        crate::prefs::update(|s| community::apply(&spec, s));
    }
}

fn free_name(name: &str) -> PathBuf {
    let d = dir();
    let first = d.join(community::file_name(name));
    if !first.exists() {
        return first;
    }
    (2..100).map(|n| d.join(community::file_name(&format!("{name} {n}")))).find(|p| !p.exists()).unwrap_or(first)
}

pub fn import(path: &Path) {
    let stem = path.file_stem().map(|s| s.to_string_lossy().chars().take(40).collect::<String>()).unwrap_or_default();
    let parsed = read(path);
    let mut notice = NOTICE.lock().ok();
    match parsed {
        Ok(p) => {
            let name = if p.spec.name.is_empty() { stem } else { p.spec.name.clone() };
            let to = dir().join(community::file_name(&name));
            let same = std::fs::canonicalize(path).ok() == std::fs::canonicalize(&to).ok();
            if !same {
                let _ = std::fs::copy(path, &to);
            }
            if let Some(n) = notice.as_mut() {
                **n = None;
            }
        }
        Err(r) => {
            if let Some(n) = notice.as_mut() {
                **n = Some((stem, r));
            }
        }
    }
    drop(notice);
    invalidate();
}

pub fn export() -> Option<PathBuf> {
    let s = crate::prefs::get();
    let name = crate::prefs::tr("Tampilanku");
    let to = free_name(name);
    let shown = to.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| name.to_string());
    std::fs::write(&to, community::export(&s, &shown, "")).ok()?;
    invalidate();
    Some(to)
}
