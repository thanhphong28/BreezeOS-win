use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Half,
    Jpg,
    Png,
    Zip,
}

impl Op {
    pub const ALL: [Op; 4] = [Op::Half, Op::Jpg, Op::Png, Op::Zip];

    pub fn title(self) -> &'static str {
        match self {
            Op::Half => "Kecilkan gambar 50%",
            Op::Jpg => "Jadikan JPG",
            Op::Png => "Jadikan PNG",
            Op::Zip => "Kompres jadi ZIP",
        }
    }

    pub fn needs_image(self) -> bool {
        self != Op::Zip
    }
}

const IMAGE_EXT: [&str; 9] = ["png", "jpg", "jpeg", "bmp", "gif", "webp", "tif", "tiff", "ico"];

fn ext_of(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

pub fn is_image(p: &str) -> bool {
    IMAGE_EXT.contains(&ext_of(Path::new(p)).as_str())
}

pub fn unique(base: PathBuf, exists: &dyn Fn(&Path) -> bool) -> PathBuf {
    if !exists(&base) {
        return base;
    }
    let dir = base.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = base.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = base.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    for n in 2..1000 {
        let c = dir.join(format!("{stem} ({n}){ext}"));
        if !exists(&c) {
            return c;
        }
    }
    dir.join(format!("{stem} (baru){ext}"))
}

pub fn output_for(src: &str, op: Op, small: &str, exists: &dyn Fn(&Path) -> bool) -> PathBuf {
    let p = Path::new(src);
    let dir = p.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = ext_of(p);
    let keep = match ext.as_str() {
        "jpg" | "jpeg" | "png" | "bmp" | "tif" | "tiff" => ext.clone(),
        _ => "png".to_string(),
    };
    let base = match op {
        Op::Half => dir.join(format!("{stem} ({small}).{keep}")),
        Op::Jpg => dir.join(format!("{stem}.jpg")),
        Op::Png => dir.join(format!("{stem}.png")),
        Op::Zip => dir.join(format!("{stem}.zip")),
    };
    unique(base, exists)
}

pub fn zip_target(paths: &[String], archive: &str, exists: &dyn Fn(&Path) -> bool) -> Option<PathBuf> {
    let first = Path::new(paths.first()?);
    if paths.len() == 1 {
        return Some(output_for(&paths[0], Op::Zip, "", exists));
    }
    let dir = first.parent()?.to_path_buf();
    Some(unique(dir.join(format!("{archive}.zip")), exists))
}

pub fn find_link(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let start = ["https://", "http://"].iter().filter_map(|s| lower.find(s)).min()?;
    let rest = &text[start..];
    let end = rest.find(|c: char| c.is_whitespace() || matches!(c, '"' | '<' | '>' | '\'' | ')' | ']' | '}')).unwrap_or(rest.len());
    let link = rest[..end].trim_end_matches(['.', ',', ';', ':', '!', '?']);
    (link.len() > 10).then(|| link.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub title: String,
    pub start: u32,
    pub minutes: u32,
    pub link: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    Soon(u32),
    Live(u32),
}

pub const LEAD_MIN: u32 = 10;

pub fn current(slots: &[Slot], now: u32) -> Option<(usize, When)> {
    let live = slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.start <= now && now < s.start + s.minutes.max(1))
        .max_by_key(|(_, s)| s.start)
        .map(|(i, s)| (i, When::Live(now - s.start)));
    let soon = slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.start > now && s.start - now <= LEAD_MIN)
        .min_by_key(|(_, s)| s.start)
        .map(|(i, s)| (i, When::Soon(s.start - now)));
    match (live, soon) {
        (Some(_), Some(s)) if matches!(s.1, When::Soon(m) if m <= 2) => Some(s),
        (Some(l), _) => Some(l),
        (None, s) => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: &Path) -> bool {
        false
    }

    #[test]
    fn images_are_detected_by_extension() {
        assert!(is_image("C:\\a\\Foto.JPG"));
        assert!(is_image("x.webp"));
        assert!(!is_image("laporan.pdf"));
        assert!(!is_image("noext"));
    }

    #[test]
    fn outputs_sit_next_to_the_source() {
        let src = "C:\\Foto\\liburan.png";
        assert_eq!(output_for(src, Op::Half, "kecil", &none), PathBuf::from("C:\\Foto\\liburan (kecil).png"));
        assert_eq!(output_for(src, Op::Jpg, "kecil", &none), PathBuf::from("C:\\Foto\\liburan.jpg"));
        assert_eq!(output_for("C:\\Foto\\a.webp", Op::Half, "kecil", &none), PathBuf::from("C:\\Foto\\a (kecil).png"));
    }

    #[test]
    fn existing_names_get_a_number() {
        let taken = |p: &Path| p.to_string_lossy().ends_with("liburan.png") || p.to_string_lossy().ends_with("liburan (2).png");
        assert_eq!(output_for("C:\\Foto\\liburan.jpg", Op::Png, "kecil", &taken), PathBuf::from("C:\\Foto\\liburan (3).png"));
    }

    #[test]
    fn zip_names() {
        let one = vec!["C:\\D\\laporan.pdf".to_string()];
        assert_eq!(zip_target(&one, "Arsip", &none), Some(PathBuf::from("C:\\D\\laporan.zip")));
        let many = vec!["C:\\D\\a.txt".to_string(), "C:\\D\\b.txt".to_string()];
        assert_eq!(zip_target(&many, "Arsip", &none), Some(PathBuf::from("C:\\D\\Arsip.zip")));
        assert_eq!(zip_target(&[], "Arsip", &none), None);
    }

    #[test]
    fn links_are_found_in_messy_text() {
        assert_eq!(find_link("Join: https://meet.google.com/abc-defg-hij."), Some("https://meet.google.com/abc-defg-hij".into()));
        assert_eq!(find_link("<https://teams.microsoft.com/l/meetup-join/xyz>"), Some("https://teams.microsoft.com/l/meetup-join/xyz".into()));
        assert_eq!(find_link("Ruang 3, lantai 2"), None);
        assert_eq!(find_link("HTTPS://ZOOM.US/j/123456789 pwd"), Some("HTTPS://ZOOM.US/j/123456789".into()));
    }

    fn slot(start: u32, minutes: u32) -> Slot {
        Slot { title: format!("r{start}"), start, minutes, link: None }
    }

    #[test]
    fn meeting_states() {
        let s = vec![slot(600, 30), slot(660, 60)];
        assert_eq!(current(&s, 580), None);
        assert_eq!(current(&s, 592), Some((0, When::Soon(8))));
        assert_eq!(current(&s, 615), Some((0, When::Live(15))));
        assert_eq!(current(&s, 630), None);
        assert_eq!(current(&s, 655), Some((1, When::Soon(5))));
    }

    #[test]
    fn next_meeting_wins_in_the_last_two_minutes() {
        let s = vec![slot(600, 60), slot(630, 30)];
        assert_eq!(current(&s, 620), Some((0, When::Live(20))));
        assert_eq!(current(&s, 628), Some((1, When::Soon(2))));
        assert_eq!(current(&s, 640), Some((1, When::Live(10))));
    }
}
