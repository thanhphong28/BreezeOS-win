use crate::settings::{BarStyle, IconStyle, Look, Material, Settings, Theme, ACCENT_MULTI};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preset {
    #[default]
    Default,
    Prism,
    Neon,
    Aurora,
    Stealth,
    Pixel,
}

impl Preset {
    pub const ALL: [Preset; 6] = [Preset::Default, Preset::Prism, Preset::Neon, Preset::Aurora, Preset::Stealth, Preset::Pixel];

    pub fn parse(s: &str) -> Option<Preset> {
        match s.trim().to_ascii_lowercase().as_str() {
            "default" | "bawaan" => Some(Preset::Default),
            "prism" => Some(Preset::Prism),
            "neon" => Some(Preset::Neon),
            "aurora" => Some(Preset::Aurora),
            "stealth" => Some(Preset::Stealth),
            "pixel" => Some(Preset::Pixel),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Preset::Default => "default",
            Preset::Prism => "prism",
            Preset::Neon => "neon",
            Preset::Aurora => "aurora",
            Preset::Stealth => "stealth",
            Preset::Pixel => "pixel",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Preset::Default => "Bawaan",
            Preset::Prism => "Prism",
            Preset::Neon => "Neon",
            Preset::Aurora => "Aurora",
            Preset::Stealth => "Stealth",
            Preset::Pixel => "Pixel",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VisStyle {
    #[default]
    Bars,
    Spectrum,
    Wave,
    Dots,
}

impl VisStyle {
    pub const ALL: [VisStyle; 4] = [VisStyle::Bars, VisStyle::Spectrum, VisStyle::Wave, VisStyle::Dots];

    pub fn parse(s: &str) -> Option<VisStyle> {
        match s.trim().to_ascii_lowercase().as_str() {
            "bars" | "batang" => Some(VisStyle::Bars),
            "spectrum" | "spektrum" => Some(VisStyle::Spectrum),
            "wave" | "gelombang" => Some(VisStyle::Wave),
            "dots" | "titik" => Some(VisStyle::Dots),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            VisStyle::Bars => "bars",
            VisStyle::Spectrum => "spectrum",
            VisStyle::Wave => "wave",
            VisStyle::Dots => "dots",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            VisStyle::Bars => "Batang",
            VisStyle::Spectrum => "Spektrum",
            VisStyle::Wave => "Gelombang",
            VisStyle::Dots => "Titik",
        }
    }

    pub fn count(self) -> usize {
        match self {
            VisStyle::Bars => 5,
            VisStyle::Spectrum => 16,
            VisStyle::Wave => 23,
            VisStyle::Dots => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontChoice {
    #[default]
    Inter,
    Segoe,
    Bahnschrift,
    Mono,
}

impl FontChoice {
    pub const ALL: [FontChoice; 4] = [FontChoice::Inter, FontChoice::Segoe, FontChoice::Bahnschrift, FontChoice::Mono];

    pub fn parse(s: &str) -> Option<FontChoice> {
        match s.trim().to_ascii_lowercase().as_str() {
            "inter" => Some(FontChoice::Inter),
            "segoe" => Some(FontChoice::Segoe),
            "bahnschrift" => Some(FontChoice::Bahnschrift),
            "mono" => Some(FontChoice::Mono),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            FontChoice::Inter => "inter",
            FontChoice::Segoe => "segoe",
            FontChoice::Bahnschrift => "bahnschrift",
            FontChoice::Mono => "mono",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            FontChoice::Inter => "Inter",
            FontChoice::Segoe => "Segoe UI",
            FontChoice::Bahnschrift => "Bahnschrift",
            FontChoice::Mono => "Mono",
        }
    }

    pub fn faces(self) -> &'static [&'static str] {
        match self {
            FontChoice::Inter => &["Inter"],
            FontChoice::Segoe => &["Segoe UI Variable Text", "Segoe UI"],
            FontChoice::Bahnschrift => &["Bahnschrift"],
            FontChoice::Mono => &["Cascadia Mono", "Consolas"],
        }
    }
}

pub fn apply(s: &mut Settings, p: Preset) {
    let d = Settings::default();
    s.preset = p;
    s.theme = d.theme;
    s.dock_material = d.dock_material;
    s.notch_material = d.notch_material;
    s.accent = d.accent;
    s.accent_rgb = None;
    s.icon_style = d.icon_style;
    s.menubar_glass = d.menubar_glass;
    s.menubar_material = d.menubar_material;
    s.menubar_style = d.menubar_style;
    s.dock_round = d.dock_round;
    s.notch_round = d.notch_round;
    s.transparency = d.transparency;
    s.notch_glow = d.notch_glow;
    s.visualizer = d.visualizer;
    if s.font == FontChoice::Mono {
        s.font = d.font;
    }
    match p {
        Preset::Default => {}
        Preset::Prism => {
            s.theme = Theme::Light;
            s.accent = ACCENT_MULTI;
            s.menubar_material = false;
            s.menubar_glass = true;
            s.menubar_style = BarStyle::Light;
            s.transparency = 55;
            s.visualizer = VisStyle::Spectrum;
        }
        Preset::Neon => {
            s.theme = Theme::Dark;
            s.dock_material = Material::Black;
            s.accent = 1;
            s.icon_style = IconStyle::Clear;
            s.menubar_material = false;
            s.menubar_glass = true;
            s.menubar_style = BarStyle::Dark;
            s.dock_round = 50;
            s.notch_round = 50;
            s.transparency = 20;
            s.visualizer = VisStyle::Spectrum;
        }
        Preset::Aurora => {
            s.theme = Theme::Dark;
            s.dock_material = Material::Blur;
            s.accent = 6;
            s.transparency = 45;
            s.visualizer = VisStyle::Wave;
        }
        Preset::Stealth => {
            s.theme = Theme::Dark;
            s.dock_material = Material::Black;
            s.accent = 7;
            s.icon_style = IconStyle::Dark;
            s.menubar_material = false;
            s.menubar_glass = true;
            s.menubar_style = BarStyle::Dark;
            s.transparency = 0;
            s.notch_glow = false;
            s.visualizer = VisStyle::Dots;
        }
        Preset::Pixel => {
            s.theme = Theme::Dark;
            s.dock_material = Material::Black;
            s.accent = 5;
            s.menubar_material = false;
            s.menubar_glass = true;
            s.menubar_style = BarStyle::Dark;
            s.dock_round = 8;
            s.notch_round = 8;
            s.transparency = 10;
            s.visualizer = VisStyle::Bars;
            s.font = FontChoice::Mono;
        }
    }
}

pub fn tune(l: &mut Look, p: Preset) {
    match p {
        Preset::Default => {}
        Preset::Prism => {
            l.rim = [210, 255, 255, 255];
            if !l.dark {
                l.tint = [l.tint[0], 250, 250, 253];
            }
            l.saturation = l.saturation.max(1.6);
        }
        Preset::Neon => {
            l.rim = [255, 0, 229, 255];
            if l.dark {
                l.tint = [l.tint[0].max(200), 8, 8, 14];
                l.solid = [255, 8, 8, 14];
            }
            l.highlight = 0.0;
        }
        Preset::Aurora => {
            if l.dark {
                l.tint = [l.tint[0], 44, 26, 86];
                l.solid = [240, 30, 20, 60];
            }
            l.rim = [170, 140, 255, 220];
            l.saturation = l.saturation.max(1.5);
        }
        Preset::Stealth => {
            l.tint = [l.tint[0].max(245), 0, 0, 0];
            l.solid = [255, 0, 0, 0];
            l.rim = [0, 255, 255, 255];
            l.highlight = 0.0;
            l.liquify = false;
            if l.dark {
                l.fg = [196, 196, 200];
            }
        }
        Preset::Pixel => {
            if l.dark {
                l.tint = [l.tint[0].max(230), 26, 28, 44];
                l.solid = [255, 26, 28, 44];
            }
            l.rim = [220, 255, 205, 117];
            l.highlight = 0.0;
            l.liquify = false;
        }
    }
}

pub fn quantize(v: f32, p: Preset) -> f32 {
    if p == Preset::Pixel {
        (v.clamp(0.0, 1.0) * 4.0).round() / 4.0
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::look;

    #[test]
    fn keys_round_trip() {
        for p in Preset::ALL {
            assert_eq!(Preset::parse(p.key()), Some(p));
        }
        for v in VisStyle::ALL {
            assert_eq!(VisStyle::parse(v.key()), Some(v));
        }
        for f in FontChoice::ALL {
            assert_eq!(FontChoice::parse(f.key()), Some(f));
            assert!(!f.faces().is_empty());
        }
        assert_eq!(Preset::parse("nope"), None);
    }

    #[test]
    fn default_preset_restores_defaults() {
        let mut s = Settings::default();
        apply(&mut s, Preset::Neon);
        assert_eq!(s.dock_material, Material::Black);
        apply(&mut s, Preset::Default);
        let d = Settings::default();
        assert_eq!((s.theme, s.dock_material, s.accent, s.dock_round, s.transparency, s.visualizer), (d.theme, d.dock_material, d.accent, d.dock_round, d.transparency, d.visualizer));
    }

    #[test]
    fn pixel_brings_mono_and_leaving_it_drops_mono_only() {
        let mut s = Settings::default();
        s.font = FontChoice::Bahnschrift;
        apply(&mut s, Preset::Prism);
        assert_eq!(s.font, FontChoice::Bahnschrift);
        apply(&mut s, Preset::Pixel);
        assert_eq!(s.font, FontChoice::Mono);
        apply(&mut s, Preset::Aurora);
        assert_eq!(s.font, FontChoice::Inter);
    }

    #[test]
    fn presets_leave_unrelated_settings_alone() {
        let mut s = Settings::default();
        s.dock_size = 61;
        s.lang = crate::i18n::Lang::Vi;
        apply(&mut s, Preset::Stealth);
        assert_eq!(s.dock_size, 61);
        assert_eq!(s.lang, crate::i18n::Lang::Vi);
        assert!(!s.notch_glow);
    }

    #[test]
    fn looks_follow_the_preset() {
        for p in Preset::ALL {
            let mut s = Settings::default();
            apply(&mut s, p);
            let l = look(&s, true, s.dock_material);
            match p {
                Preset::Neon => assert_eq!(&l.rim[1..], &[0, 229, 255]),
                Preset::Stealth => assert_eq!(&l.tint[1..], &[0, 0, 0]),
                Preset::Pixel => assert!(!l.liquify && l.highlight == 0.0),
                Preset::Prism => assert!(!l.dark),
                _ => {}
            }
        }
    }

    #[test]
    fn pixel_steps_levels() {
        assert_eq!(quantize(0.6, Preset::Pixel), 0.5);
        assert_eq!(quantize(0.9, Preset::Pixel), 1.0);
        assert_eq!(quantize(0.6, Preset::Neon), 0.6);
    }
}
