use crate::notch::glance::{Capsule, Decor, Item, Layout};
use crate::settings::{hex, parse_hex, Settings, Theme, ACCENTS, ACCENT_MULTI};
use crate::theme::{self, FontChoice, Preset, VisStyle};

pub const EXT: &str = "breezetheme";
pub const MAX_BYTES: usize = 16 * 1024;
pub const MAX_LINES: usize = 200;
pub const FORMAT: u32 = 1;
pub const HEADER: &str = "breezeos_theme";
const TEXT_MAX: usize = 40;
const IGNORED_MAX: usize = 20;
const PALETTE: [&str; 8] = ["blue", "purple", "pink", "red", "orange", "yellow", "green", "graphite"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Accent {
    System,
    Palette(u8),
    Rgb([u8; 3]),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Spec {
    pub name: String,
    pub author: String,
    pub preset: Option<Preset>,
    pub theme: Option<Theme>,
    pub accent: Option<Accent>,
    pub font: Option<FontChoice>,
    pub visualizer: Option<VisStyle>,
    pub transparency: Option<u8>,
    pub round: Option<u8>,
    pub glow: Option<bool>,
    pub reduce_motion: Option<bool>,
    pub decor: Option<Decor>,
    pub capsule: Option<Capsule>,
    pub layout: Option<Layout>,
}

impl Spec {
    pub fn fields(&self) -> usize {
        [
            self.preset.is_some(),
            self.theme.is_some(),
            self.accent.is_some(),
            self.font.is_some(),
            self.visualizer.is_some(),
            self.transparency.is_some(),
            self.round.is_some(),
            self.glow.is_some(),
            self.reduce_motion.is_some(),
            self.decor.is_some(),
            self.capsule.is_some(),
            self.layout.is_some(),
        ]
        .iter()
        .filter(|b| **b)
        .count()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    TooBig,
    NotText,
    NoHeader,
    Newer,
    Empty,
}

impl Reject {
    pub fn reason(self) -> &'static str {
        match self {
            Reject::TooBig => "Berkas terlalu besar",
            Reject::NotText => "Bukan berkas teks",
            Reject::NoHeader => "Bukan berkas tema BreezeOS",
            Reject::Newer => "Butuh BreezeOS versi lebih baru",
            Reject::Empty => "Tidak ada pengaturan yang bisa dipakai",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    pub spec: Spec,
    pub ignored: Vec<String>,
}

fn clean(v: &str) -> String {
    v.chars().filter(|c| !c.is_control()).collect::<String>().trim().chars().take(TEXT_MAX).collect()
}

fn flag(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "on" | "true" | "yes" | "1" => Some(true),
        "off" | "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

fn percent(v: &str) -> Option<u8> {
    v.trim_end_matches('%').trim().parse::<i64>().ok().map(|n| n.clamp(0, 100) as u8)
}

fn accent(v: &str) -> Option<Accent> {
    let low = v.to_ascii_lowercase();
    if matches!(low.as_str(), "system" | "multi" | "windows") {
        return Some(Accent::System);
    }
    if let Some(i) = PALETTE.iter().position(|p| *p == low) {
        return Some(Accent::Palette(i as u8));
    }
    parse_hex(v).map(Accent::Rgb)
}

fn capsule(v: &str) -> Option<(Capsule, bool)> {
    let parts: Vec<&str> = v.split(',').map(str::trim).filter(|p| !p.is_empty()).collect();
    let known = parts.iter().filter(|p| Item::parse(p).is_some()).count();
    if known == 0 && !parts.is_empty() {
        return None;
    }
    Some((Capsule::parse(v), known < parts.len()))
}

fn motion(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "normal" | "full" => Some(false),
        "reduce" | "reduced" | "minim" | "minimal" => Some(true),
        _ => None,
    }
}

pub fn parse(bytes: &[u8]) -> Result<Parsed, Reject> {
    if bytes.len() > MAX_BYTES {
        return Err(Reject::TooBig);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Reject::NotText)?;
    if text.contains('\0') {
        return Err(Reject::NotText);
    }
    let text = text.trim_start_matches('\u{feff}');
    if text.lines().count() > MAX_LINES {
        return Err(Reject::TooBig);
    }
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#'));
    let head = lines.next().ok_or(Reject::NoHeader)?;
    let version = match head.split_once('=') {
        Some((k, v)) if k.trim() == HEADER || k.trim() == "abra_theme" => v.trim().parse::<u32>().map_err(|_| Reject::NoHeader)?,
        _ => return Err(Reject::NoHeader),
    };
    if version == 0 {
        return Err(Reject::NoHeader);
    }
    if version > FORMAT {
        return Err(Reject::Newer);
    }
    let mut spec = Spec::default();
    let mut ignored: Vec<String> = Vec::new();
    let skip = |what: &str, ignored: &mut Vec<String>| {
        if ignored.len() < IGNORED_MAX && !ignored.iter().any(|i| i == what) {
            ignored.push(clean(what));
        }
    };
    for line in lines {
        let Some((k, v)) = line.split_once('=') else {
            skip(line, &mut ignored);
            continue;
        };
        let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
        let ok = match k.as_str() {
            "name" => {
                spec.name = clean(v);
                !spec.name.is_empty()
            }
            "author" => {
                spec.author = clean(v);
                !spec.author.is_empty()
            }
            "preset" => Preset::parse(v).map(|p| spec.preset = Some(p)).is_some(),
            "theme" => Theme::parse(v).map(|t| spec.theme = Some(t)).is_some(),
            "accent" => accent(v).map(|a| spec.accent = Some(a)).is_some(),
            "font" => FontChoice::parse(v).map(|f| spec.font = Some(f)).is_some(),
            "visualizer" => VisStyle::parse(v).map(|x| spec.visualizer = Some(x)).is_some(),
            "transparency" => percent(v).map(|n| spec.transparency = Some(n)).is_some(),
            "round" => percent(v).map(|n| spec.round = Some(n)).is_some(),
            "glow" => flag(v).map(|b| spec.glow = Some(b)).is_some(),
            "motion" => motion(v).map(|b| spec.reduce_motion = Some(b)).is_some(),
            "decor" => Decor::parse(v).map(|d| spec.decor = Some(d)).is_some(),
            "capsule_layout" => Layout::parse(v).map(|l| spec.layout = Some(l)).is_some(),
            "capsule" => match capsule(v) {
                Some((c, partial)) => {
                    spec.capsule = Some(c);
                    if partial {
                        skip("capsule", &mut ignored);
                    }
                    true
                }
                None => false,
            },
            _ => false,
        };
        if !ok {
            skip(&k, &mut ignored);
        }
    }
    if spec.fields() == 0 {
        return Err(Reject::Empty);
    }
    Ok(Parsed { spec, ignored })
}

pub fn apply(spec: &Spec, s: &mut Settings) {
    if let Some(p) = spec.preset {
        theme::apply(s, p);
    }
    if let Some(t) = spec.theme {
        s.theme = t;
    }
    match spec.accent {
        Some(Accent::System) => {
            s.accent = ACCENT_MULTI;
            s.accent_rgb = None;
        }
        Some(Accent::Palette(i)) => {
            s.accent = i.min(ACCENTS.len() as u8 - 1);
            s.accent_rgb = None;
        }
        Some(Accent::Rgb(c)) => s.accent_rgb = Some(c),
        None => {}
    }
    if let Some(f) = spec.font {
        s.font = f;
    }
    if let Some(v) = spec.visualizer {
        s.visualizer = v;
    }
    if let Some(t) = spec.transparency {
        s.transparency = t;
    }
    if let Some(r) = spec.round {
        s.dock_round = r;
        s.notch_round = r;
    }
    if let Some(g) = spec.glow {
        s.notch_glow = g;
    }
    if let Some(m) = spec.reduce_motion {
        s.reduce_motion = m;
    }
    if let Some(d) = spec.decor {
        s.capsule_decor = d;
    }
    if let Some(c) = spec.capsule {
        s.capsule = c;
    }
    if let Some(l) = spec.layout {
        s.capsule_layout = l;
    }
}

pub fn export(s: &Settings, name: &str, author: &str) -> String {
    let accent = match s.accent_rgb {
        Some(c) => hex(c),
        None if s.accent == ACCENT_MULTI => "system".to_string(),
        None => PALETTE[(s.accent as usize).min(PALETTE.len() - 1)].to_string(),
    };
    let mut out = format!("{HEADER}={FORMAT}\nname={}\n", clean(name));
    let author = clean(author);
    if !author.is_empty() {
        out.push_str(&format!("author={author}\n"));
    }
    out.push_str(&format!(
        "preset={}\ntheme={}\naccent={accent}\nfont={}\nvisualizer={}\ntransparency={}\nround={}\nglow={}\nmotion={}\ndecor={}\ncapsule={}\ncapsule_layout={}\n",
        s.preset.key(),
        s.theme.key(),
        s.font.key(),
        s.visualizer.key(),
        s.transparency,
        s.notch_round,
        if s.notch_glow { "on" } else { "off" },
        if s.reduce_motion { "reduce" } else { "normal" },
        s.capsule_decor.key(),
        s.capsule.text(),
        s.capsule_layout.key(),
    ));
    out
}

pub fn file_name(name: &str) -> String {
    let base: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(TEXT_MAX)
        .collect();
    let base = if base.is_empty() { "Tema".to_string() } else { base };
    format!("{base}.{EXT}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# komentar boleh\nbreezeos_theme=1\nname=Midnight Glass\nauthor=BreezeOS\npreset=stealth\naccent=#7F77DD\nfont=bahnschrift\nvisualizer=wave\ntransparency=30\nround=60\nglow=on\nmotion=normal\ndecor=dot\ncapsule=clock,battery,system\ncapsule_layout=split\n";

    #[test]
    fn valid_theme_parses_and_applies_on_top_of_its_preset() {
        let p = parse(SAMPLE.as_bytes()).unwrap();
        assert!(p.ignored.is_empty());
        assert_eq!((p.spec.name.as_str(), p.spec.author.as_str(), p.spec.fields()), ("Midnight Glass", "BreezeOS", 11));
        let mut s = Settings::default();
        s.dock_size = 77;
        apply(&p.spec, &mut s);
        assert_eq!(s.preset, Preset::Stealth);
        assert_eq!(s.theme, Theme::Dark);
        assert_eq!(s.accent_rgb, Some([0x7F, 0x77, 0xDD]));
        assert_eq!((s.font, s.visualizer, s.transparency, s.dock_round, s.notch_round), (FontChoice::Bahnschrift, VisStyle::Wave, 30, 60, 60));
        assert!(s.notch_glow && !s.reduce_motion);
        assert_eq!((s.capsule_decor, s.capsule_layout, s.capsule.text()), (Decor::Dot, Layout::Split, "clock,battery,system".to_string()));
        assert_eq!(s.dock_size, 77, "tema tidak menyentuh setelan di luar tampilan");
    }

    #[test]
    fn export_round_trips_through_the_validator() {
        let mut s = Settings::default();
        theme::apply(&mut s, Preset::Neon);
        s.accent_rgb = Some([1, 2, 3]);
        s.capsule = Capsule::parse("agenda,clock");
        s.capsule_decor = Decor::Edge;
        s.reduce_motion = true;
        let text = export(&s, "Neon\u{7}  Saya", "BreezeOS\nx=1");
        let p = parse(text.as_bytes()).unwrap();
        assert!(p.ignored.is_empty(), "{:?}", p.ignored);
        assert_eq!(p.spec.name, "Neon  Saya");
        assert_eq!(p.spec.author, "BreezeOSx=1");
        let mut back = Settings::default();
        apply(&p.spec, &mut back);
        assert_eq!((back.preset, back.accent_rgb, back.capsule, back.capsule_decor, back.reduce_motion), (s.preset, s.accent_rgb, s.capsule, s.capsule_decor, true));
        assert_eq!((back.transparency, back.notch_round, back.notch_glow, back.visualizer), (s.transparency, s.notch_round, s.notch_glow, s.visualizer));
    }

    #[test]
    fn unknown_or_invalid_keys_are_ignored_and_named() {
        let p = parse(b"breezeos_theme=1\nname=Sakura\naccent=pink\nexec=calc.exe\nicon=C:\\\\Windows\\\\x.dll\ntransparency=banyak\nround=500\ncapsule=clock,cuaca\njust text\n").unwrap();
        assert_eq!(p.spec.accent, Some(Accent::Palette(2)));
        assert_eq!(p.spec.round, Some(100));
        assert_eq!(p.spec.capsule.unwrap().text(), "clock");
        assert_eq!(p.ignored, vec!["exec", "icon", "transparency", "capsule", "just text"]);
        let mut many = String::from("breezeos_theme=1\nfont=mono\n");
        for i in 0..60 {
            many.push_str(&format!("k{i}=1\n"));
        }
        assert_eq!(parse(many.as_bytes()).unwrap().ignored.len(), 20);
    }

    #[test]
    fn bad_files_are_rejected() {
        assert_eq!(parse(b"name=x\npreset=neon\n"), Err(Reject::NoHeader));
        assert_eq!(parse(b""), Err(Reject::NoHeader));
        assert_eq!(parse(b"breezeos_theme=abc\npreset=neon\n"), Err(Reject::NoHeader));
        assert_eq!(parse(b"breezeos_theme=0\npreset=neon\n"), Err(Reject::NoHeader));
        assert_eq!(parse(b"breezeos_theme=2\npreset=neon\n"), Err(Reject::Newer));
        assert_eq!(parse(b"breezeos_theme=1\nname=Kosong\nfoo=bar\n"), Err(Reject::Empty));
        assert_eq!(parse(&[0xFF, 0xFE, 0x00, 0x41]), Err(Reject::NotText));
        assert_eq!(parse(b"breezeos_theme=1\0\npreset=neon\n"), Err(Reject::NotText));
        assert_eq!(parse(&vec![b'a'; MAX_BYTES + 1]), Err(Reject::TooBig));
        assert_eq!(parse("breezeos_theme=1\n".repeat(MAX_LINES + 1).as_bytes()), Err(Reject::TooBig));
        assert!(parse("\u{feff}breezeos_theme=1\npreset=pixel\n".as_bytes()).is_ok());
    }

    #[test]
    fn file_names_are_safe() {
        assert_eq!(file_name("Midnight Glass"), "Midnight Glass.breezetheme");
        assert_eq!(file_name("..\\..\\Windows/sys32:x"), "Windows sys32 x.breezetheme");
        assert_eq!(file_name("   "), "Tema.breezetheme");
        assert_eq!(file_name(&"a".repeat(90)).len(), 40 + 1 + EXT.len());
    }
}
