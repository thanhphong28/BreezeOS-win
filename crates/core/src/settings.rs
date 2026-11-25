use crate::display::Screen;
use crate::theme::{FontChoice, Preset, VisStyle};
use crate::notch::glance::{Capsule, Decor, Layout};
use crate::i18n::Lang;
use crate::visibility::policy::Mode;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reveal {
    #[default]
    Quick,
    Calm,
}

impl Reveal {
    pub const ALL: [Reveal; 2] = [Reveal::Quick, Reveal::Calm];

    pub fn parse(s: &str) -> Option<Reveal> {
        match s.trim().to_ascii_lowercase().as_str() {
            "quick" | "cepat" | "fast" | "sensitive" => Some(Reveal::Quick),
            "calm" | "santai" | "relaxed" | "delay" => Some(Reveal::Calm),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Reveal::Quick => "quick",
            Reveal::Calm => "calm",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Reveal::Quick => "Cepat",
            Reveal::Calm => "Santai",
        }
    }

    pub fn deck_ms(self) -> u32 {
        match self {
            Reveal::Quick => 200,
            Reveal::Calm => 550,
        }
    }

    pub fn rail_ms(self) -> u32 {
        match self {
            Reveal::Quick => 150,
            Reveal::Calm => 500,
        }
    }

    pub fn visor_edge_ms(self) -> u32 {
        match self {
            Reveal::Quick => 200,
            Reveal::Calm => 500,
        }
    }

    pub fn visor_open_ms(self) -> u32 {
        match self {
            Reveal::Quick => 180,
            Reveal::Calm => 450,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Behavior {
    Fixed,
    Overlay,
    Smart,
}

impl Behavior {
    pub fn parse(s: &str) -> Option<Behavior> {
        match s.trim().to_ascii_lowercase().as_str() {
            "fixed" | "tetap" | "permanent" => Some(Behavior::Fixed),
            "overlay" | "menimpa" => Some(Behavior::Overlay),
            "smart" | "autohide" | "auto-hide" | "auto" | "peek" | "hidden" => Some(Behavior::Smart),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Behavior::Fixed => "fixed",
            Behavior::Overlay => "overlay",
            Behavior::Smart => "smart",
        }
    }

    pub fn mode(self) -> Mode {
        match self {
            Behavior::Fixed | Behavior::Overlay => Mode::Always,
            Behavior::Smart => Mode::AutoHide,
        }
    }

    pub fn reserves(self) -> bool {
        self == Behavior::Fixed
    }

    pub const ALL: [Behavior; 3] = [Behavior::Fixed, Behavior::Overlay, Behavior::Smart];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Auto,
    Light,
    Dark,
}

impl Theme {
    pub fn parse(s: &str) -> Option<Theme> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" | "system" => Some(Theme::Auto),
            "light" | "terang" => Some(Theme::Light),
            "dark" | "gelap" => Some(Theme::Dark),
            _ => None,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Theme::Auto => "auto",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
    pub const ALL: [Theme; 3] = [Theme::Auto, Theme::Light, Theme::Dark];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlurStyle {
    Frosted,
    Clear,
    Vivid,
}

impl BlurStyle {
    pub fn parse(s: &str) -> Option<BlurStyle> {
        match s.trim().to_ascii_lowercase().as_str() {
            "frosted" | "buram" => Some(BlurStyle::Frosted),
            "clear" | "bening" => Some(BlurStyle::Clear),
            "vivid" | "hidup" => Some(BlurStyle::Vivid),
            _ => None,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            BlurStyle::Frosted => "frosted",
            BlurStyle::Clear => "clear",
            BlurStyle::Vivid => "vivid",
        }
    }
    pub const ALL: [BlurStyle; 3] = [BlurStyle::Frosted, BlurStyle::Clear, BlurStyle::Vivid];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    Blur,
    Glass,
    Black,
}

impl Material {
    pub fn parse(s: &str) -> Option<Material> {
        match s.trim().to_ascii_lowercase().as_str() {
            "blur" | "buram" | "frosted" => Some(Material::Blur),
            "glass" | "liquid" | "liquidglass" => Some(Material::Glass),
            "black" | "hitam" => Some(Material::Black),
            "clear" | "transparan" | "transparent" => Some(Material::Glass),
            _ => None,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Material::Blur => "blur",
            Material::Glass => "glass",
            Material::Black => "black",
        }
    }
    pub const ALL: [Material; 3] = [Material::Blur, Material::Glass, Material::Black];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarStyle {
    Light,
    Dark,
}

impl BarStyle {
    pub fn parse(s: &str) -> Option<BarStyle> {
        match s.trim().to_ascii_lowercase().as_str() {
            "light" | "terang" => Some(BarStyle::Light),
            "dark" | "gelap" => Some(BarStyle::Dark),
            _ => None,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            BarStyle::Light => "light",
            BarStyle::Dark => "dark",
        }
    }
}

pub const ACCENT_MULTI: u8 = 255;

pub const ACCENTS: [[u8; 3]; 8] = [[10, 132, 255], [175, 82, 222], [255, 55, 95], [255, 69, 58], [255, 149, 10], [255, 204, 0], [48, 209, 88], [142, 142, 147]];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconStyle {
    Default,
    Dark,
    Clear,
    Tinted,
}

impl IconStyle {
    pub fn parse(s: &str) -> Option<IconStyle> {
        match s.trim().to_ascii_lowercase().as_str() {
            "default" | "bawaan" => Some(IconStyle::Default),
            "dark" | "gelap" => Some(IconStyle::Dark),
            "clear" | "bening" => Some(IconStyle::Clear),
            "tinted" | "berwarna" => Some(IconStyle::Tinted),
            _ => None,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            IconStyle::Default => "default",
            IconStyle::Dark => "dark",
            IconStyle::Clear => "clear",
            IconStyle::Tinted => "tinted",
        }
    }
}

pub fn round_factor(round: u8) -> f32 {
    0.2 + 1.6 * (round.min(100) as f32 / 100.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HotAction {
    None,
    Mission,
    Launchpad,
    Desktop,
    Stage,
    Notifications,
    Lock,
}

impl HotAction {
    pub const ALL: [HotAction; 7] =
        [HotAction::None, HotAction::Mission, HotAction::Launchpad, HotAction::Desktop, HotAction::Stage, HotAction::Notifications, HotAction::Lock];

    pub fn parse(s: &str) -> Option<HotAction> {
        Self::ALL.into_iter().find(|a| a.key() == s.trim().to_ascii_lowercase())
    }
    pub fn key(self) -> &'static str {
        match self {
            HotAction::None => "none",
            HotAction::Mission => "mission",
            HotAction::Launchpad => "launchpad",
            HotAction::Desktop => "desktop",
            HotAction::Stage => "stage",
            HotAction::Notifications => "notifications",
            HotAction::Lock => "lock",
        }
    }
    pub fn next(self) -> HotAction {
        let i = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShortText {
    len: u8,
    buf: [u8; 96],
}

impl ShortText {
    pub const EMPTY: ShortText = ShortText { len: 0, buf: [0; 96] };

    pub fn new(s: &str) -> ShortText {
        let clean: String = s.chars().filter(|c| !c.is_control()).collect();
        let clean = clean.trim();
        let mut end = clean.len().min(96);
        while !clean.is_char_boundary(end) {
            end -= 1;
        }
        let mut buf = [0u8; 96];
        buf[..end].copy_from_slice(&clean.as_bytes()[..end]);
        ShortText { len: end as u8, buf }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("")
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for ShortText {
    fn default() -> Self {
        ShortText::EMPTY
    }
}

impl std::fmt::Debug for ShortText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

pub const HK_CTRL: u8 = 1;
pub const HK_ALT: u8 = 2;
pub const HK_SHIFT: u8 = 4;
pub const HK_WIN: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hotkey {
    pub mods: u8,
    pub vk: u16,
}

impl Default for Hotkey {
    fn default() -> Self {
        Self::WIN_F
    }
}

impl Hotkey {
    pub const WIN_F: Hotkey = Hotkey { mods: HK_WIN, vk: 0x46 };

    pub fn has_required_mod(mods: u8) -> bool {
        mods & (HK_CTRL | HK_ALT | HK_WIN) != 0
    }

    pub fn is_modifier_vk(vk: u16) -> bool {
        matches!(vk, 0x10..=0x12 | 0x5B | 0x5C | 0xA0..=0xA5)
    }

    pub fn key_name(vk: u16) -> Option<String> {
        match vk {
            0x41..=0x5A | 0x30..=0x39 => Some(String::from(vk as u8 as char)),
            0x70..=0x87 => Some(format!("F{}", vk - 0x6F)),
            0x20 => Some("Space".into()),
            0x09 => Some("Tab".into()),
            0x0D => Some("Enter".into()),
            0xC0 => Some("`".into()),
            0xBD => Some("-".into()),
            0xBB => Some("=".into()),
            0xDB => Some("[".into()),
            0xDD => Some("]".into()),
            0xDC => Some("\\".into()),
            0xBA => Some(";".into()),
            0xDE => Some("'".into()),
            0xBC => Some(",".into()),
            0xBE => Some(".".into()),
            0xBF => Some("/".into()),
            0x25 => Some("Left".into()),
            0x26 => Some("Up".into()),
            0x27 => Some("Right".into()),
            0x28 => Some("Down".into()),
            0x2D => Some("Insert".into()),
            0x2E => Some("Delete".into()),
            0x24 => Some("Home".into()),
            0x23 => Some("End".into()),
            0x21 => Some("PgUp".into()),
            0x22 => Some("PgDn".into()),
            _ => None,
        }
    }

    fn vk_from_token(t: &str) -> Option<u16> {
        let u = t.trim().to_ascii_uppercase();
        if u.len() == 1 {
            let b = u.as_bytes()[0];
            if b.is_ascii_uppercase() || b.is_ascii_digit() {
                return Some(b as u16);
            }
        }
        if let Some(rest) = u.strip_prefix('F') {
            if let Ok(n) = rest.parse::<u16>() {
                if (1..=24).contains(&n) {
                    return Some(0x6F + n);
                }
            }
        }
        match u.as_str() {
            "SPACE" => Some(0x20),
            "TAB" => Some(0x09),
            "ENTER" | "RETURN" => Some(0x0D),
            "`" | "BACKTICK" | "TILDE" => Some(0xC0),
            "-" | "MINUS" => Some(0xBD),
            "=" | "PLUS" | "EQUAL" => Some(0xBB),
            "[" => Some(0xDB),
            "]" => Some(0xDD),
            "\\" => Some(0xDC),
            ";" => Some(0xBA),
            "'" => Some(0xDE),
            "," => Some(0xBC),
            "." => Some(0xBE),
            "/" => Some(0xBF),
            "LEFT" => Some(0x25),
            "UP" => Some(0x26),
            "RIGHT" => Some(0x27),
            "DOWN" => Some(0x28),
            "INSERT" | "INS" => Some(0x2D),
            "DELETE" | "DEL" => Some(0x2E),
            "HOME" => Some(0x24),
            "END" => Some(0x23),
            "PGUP" | "PAGEUP" => Some(0x21),
            "PGDN" | "PAGEDOWN" => Some(0x22),
            _ => None,
        }
    }

    pub fn from_parts(mods: u8, vk: u16) -> Option<Hotkey> {
        let mods = mods & (HK_CTRL | HK_ALT | HK_SHIFT | HK_WIN);
        (Self::has_required_mod(mods) && !Self::is_modifier_vk(vk) && Self::key_name(vk).is_some()).then_some(Hotkey { mods, vk })
    }

    pub fn parse(s: &str) -> Option<Hotkey> {
        let mut mods = 0u8;
        let mut vk = None;
        for part in s.split('+') {
            let p = part.trim();
            if p.is_empty() {
                continue;
            }
            match p.to_ascii_lowercase().as_str() {
                "win" | "super" | "meta" => mods |= HK_WIN,
                "ctrl" | "control" => mods |= HK_CTRL,
                "alt" | "opt" | "option" => mods |= HK_ALT,
                "shift" => mods |= HK_SHIFT,
                _ => {
                    if vk.is_some() {
                        return None;
                    }
                    vk = Some(Self::vk_from_token(p)?);
                }
            }
        }
        Self::from_parts(mods, vk?)
    }

    fn parts(self) -> Vec<String> {
        let mut out = Vec::new();
        if self.mods & HK_WIN != 0 {
            out.push("Win".into());
        }
        if self.mods & HK_CTRL != 0 {
            out.push("Ctrl".into());
        }
        if self.mods & HK_ALT != 0 {
            out.push("Alt".into());
        }
        if self.mods & HK_SHIFT != 0 {
            out.push("Shift".into());
        }
        out.push(Self::key_name(self.vk).unwrap_or_else(|| "F".into()));
        out
    }

    pub fn label(self) -> String {
        self.parts().join(" + ")
    }

    pub fn key(self) -> String {
        self.parts().join("+")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub takeover: bool,
    pub dock: Behavior,
    pub notch: Behavior,
    pub blur: u8,
    pub style: BlurStyle,
    pub transparency: u8,
    pub theme: Theme,
    pub lang: Lang,
    pub dock_material: Material,
    pub dock_round: u8,
    pub notch_material: Material,
    pub notch_round: u8,
    pub autostart: bool,
    pub dock_size: u8,
    pub dock_gap: u8,
    pub dock_zoom: u8,
    pub dock_margin: u8,
    pub anim: bool,
    pub refraction: bool,
    pub dock_enabled: bool,
    pub notch_enabled: bool,
    pub notch_hover_open: bool,
    pub win_shadow: bool,
    pub menubar: bool,
    pub clock_24h: bool,
    pub menubar_glass: bool,
    pub menubar_material: bool,
    pub menubar_style: BarStyle,
    pub hot: [HotAction; 4],
    pub stage: bool,
    pub switcher: bool,
    pub stage_right: bool,
    pub stage_size: u8,
    pub stage_gap: u8,
    pub stage_count: u8,
    pub stage_speed: u8,
    pub stage_desktop_click: bool,
    pub stage_media: bool,
    pub stage_icons: u8,
    pub stage_titles: bool,
    pub hide_desktop_icons: bool,
    pub dock_plates: bool,
    pub above_fullscreen: bool,
    pub clipboard_history: bool,
    pub icon_style: IconStyle,
    pub icon_tint: u8,
    pub accent: u8,
    pub icon_variant: u8,
    pub min_effect: u8,
    pub notch_idle: u8,
    pub greet_name: ShortText,
    pub greet_text: ShortText,
    pub lyrics_compact: bool,
    pub lyrics_online: bool,
    pub notch_downloads: bool,
    pub notch_send: bool,
    pub mac_shortcuts: bool,
    pub downloads_stack: bool,
    pub recent_apps: bool,
    pub dock_preview: bool,
    pub search_hotkey: Hotkey,
    pub icon_pack: bool,
    pub ui_scale: u8,
    pub notch_island: bool,
    pub widget_style: u8,
    pub battery_pct: u8,
    pub rail: Behavior,
    pub dock_reveal: Reveal,
    pub rail_reveal: Reveal,
    pub notch_reveal: Reveal,
    pub notch_scale: u8,
    pub notch_height: u8,
    pub rail_scale: u8,
    pub island_gap: u8,
    pub update_check: bool,
    pub notch_glow: bool,
    pub pill_devices: bool,
    pub pill_notices: bool,
    pub pill_reminders: bool,
    pub todo_hotkey: bool,
    pub deck_screen: Screen,
    pub rail_screen: Screen,
    pub notch_screen: Screen,
    pub widget_screen: Screen,
    pub preset: Preset,
    pub visualizer: VisStyle,
    pub font: FontChoice,
    pub reduce_motion: bool,
    pub focus_cat: u8,
    pub focus_mins: [u8; crate::focus::CATS],
    pub break_mins: [u8; crate::focus::CATS],
    pub focus_name: ShortText,
    pub capsule: Capsule,
    pub capsule_layout: Layout,
    pub capsule_decor: Decor,
    pub accent_rgb: Option<[u8; 3]>,
    pub meeting_companion: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            takeover: true,
            dock: Behavior::Smart,
            notch: Behavior::Fixed,
            blur: 100,
            style: BlurStyle::Clear,
            transparency: 50,
            theme: Theme::Auto,
            lang: Lang::En,
            dock_material: Material::Glass,
            dock_round: 62,
            notch_material: Material::Black,
            notch_round: 60,
            autostart: false,
            dock_size: 48,
            dock_gap: 8,
            dock_zoom: 184,
            dock_margin: 8,
            anim: true,
            refraction: true,
            dock_enabled: true,
            notch_enabled: true,
            notch_hover_open: true,
            win_shadow: true,
            menubar: true,
            clock_24h: false,
            menubar_glass: false,
            menubar_material: true,
            menubar_style: BarStyle::Dark,
            hot: [HotAction::None; 4],
            stage: false,
            switcher: true,
            stage_right: false,
            stage_size: 152,
            stage_gap: 20,
            stage_count: 6,
            stage_speed: 100,
            stage_desktop_click: true,
            stage_media: false,
            stage_icons: 2,
            stage_titles: true,
            hide_desktop_icons: true,
            dock_plates: true,
            above_fullscreen: false,
            clipboard_history: true,
            icon_style: IconStyle::Default,
            icon_tint: 0,
            accent: 0,
            icon_variant: 1,
            min_effect: 0,
            notch_idle: 0,
            greet_name: ShortText::EMPTY,
            greet_text: ShortText::EMPTY,
            lyrics_compact: true,
            lyrics_online: true,
            notch_downloads: true,
            notch_send: true,
            mac_shortcuts: true,
            downloads_stack: true,
            recent_apps: true,
            dock_preview: true,
            search_hotkey: Hotkey::WIN_F,
            icon_pack: true,
            ui_scale: 100,
            notch_island: false,
            widget_style: 0,
            battery_pct: 2,
            rail: Behavior::Fixed,
            dock_reveal: Reveal::Quick,
            rail_reveal: Reveal::Quick,
            notch_reveal: Reveal::Quick,
            notch_scale: 100,
            notch_height: 100,
            rail_scale: 100,
            island_gap: 7,
            update_check: true,
            notch_glow: true,
            pill_devices: true,
            pill_notices: false,
            pill_reminders: true,
            todo_hotkey: true,
            deck_screen: Screen::Primary,
            rail_screen: Screen::Primary,
            notch_screen: Screen::Primary,
            widget_screen: Screen::Primary,
            preset: Preset::Default,
            visualizer: VisStyle::Bars,
            font: FontChoice::Inter,
            reduce_motion: false,
            focus_cat: 0,
            focus_mins: crate::focus::DEFAULT_FOCUS,
            break_mins: crate::focus::DEFAULT_BREAK,
            focus_name: ShortText::EMPTY,
            meeting_companion: true,
            capsule: Capsule::default(),
            capsule_layout: Layout::Rotate,
            capsule_decor: Decor::None,
            accent_rgb: None,
        }
    }
}

impl Settings {
    pub fn parse(text: &str) -> Settings {
        let mut s = Settings::default();
        let mut legacy_glass = false;
        let mut saw_notch_scale = false;
        let mut saw_notch_height = false;
        for line in text.trim_start_matches('\u{feff}').lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            match k {
                "takeover" => {
                    if let Some(b) = parse_bool(v) {
                        s.takeover = b;
                    }
                }
                "dock" => s.dock = Behavior::parse(v).unwrap_or(s.dock),
                "notch" => s.notch = Behavior::parse(v).unwrap_or(s.notch),
                "blur" => s.blur = v.parse::<i32>().map(|n| n.clamp(0, 100) as u8).unwrap_or(s.blur),
                "style" => {}
                "transparency" => s.transparency = v.parse::<i32>().map(|n| n.clamp(0, 100) as u8).unwrap_or(s.transparency),
                "theme" => s.theme = Theme::parse(v).unwrap_or(s.theme),
                "lang" => s.lang = Lang::parse(v).unwrap_or(s.lang),
                "dock_material" => s.dock_material = Material::parse(v).unwrap_or(s.dock_material),
                "notch_material" => s.notch_material = Material::parse(v).unwrap_or(s.notch_material),
                "dock_round" => s.dock_round = v.parse::<i32>().map(|n| n.clamp(0, 100) as u8).unwrap_or(s.dock_round),
                "notch_round" => s.notch_round = v.parse::<i32>().map(|n| n.clamp(0, 100) as u8).unwrap_or(s.notch_round),
                "dock_size" => s.dock_size = v.parse::<i32>().map(|n| n.clamp(32, 72) as u8).unwrap_or(s.dock_size),
                "dock_gap" => s.dock_gap = v.parse::<i32>().map(|n| n.clamp(0, 24) as u8).unwrap_or(s.dock_gap),
                "dock_zoom" => s.dock_zoom = v.parse::<i32>().map(|n| n.clamp(100, 250) as u8).unwrap_or(s.dock_zoom),
                "dock_margin" => s.dock_margin = v.parse::<i32>().map(|n| n.clamp(0, 60) as u8).unwrap_or(s.dock_margin),
                "dock_enabled" => {
                    if let Some(b) = parse_bool(v) {
                        s.dock_enabled = b;
                    }
                }
                "menubar" => {
                    if let Some(b) = parse_bool(v) {
                        s.menubar = b;
                    }
                }
                "hot_tl" => s.hot[0] = HotAction::parse(v).unwrap_or(s.hot[0]),
                "hot_tr" => s.hot[1] = HotAction::parse(v).unwrap_or(s.hot[1]),
                "hot_bl" => s.hot[2] = HotAction::parse(v).unwrap_or(s.hot[2]),
                "hot_br" => s.hot[3] = HotAction::parse(v).unwrap_or(s.hot[3]),
                "switcher" => {
                    if let Some(b) = parse_bool(v) {
                        s.switcher = b;
                    }
                }
                "stage" => {
                    if let Some(b) = parse_bool(v) {
                        s.stage = b;
                    }
                }
                "stage_right" => {
                    if let Some(b) = parse_bool(v) {
                        s.stage_right = b;
                    }
                }
                "stage_desktop_click" => {
                    if let Some(b) = parse_bool(v) {
                        s.stage_desktop_click = b;
                    }
                }
                "icon_style" => s.icon_style = IconStyle::parse(v).unwrap_or(s.icon_style),
                "downloads_stack" => {
                    if let Some(b) = parse_bool(v) {
                        s.downloads_stack = b;
                    }
                }
                "icon_pack" => {
                    if let Some(b) = parse_bool(v) {
                        s.icon_pack = b;
                    }
                }
                "recent_apps" => {
                    if let Some(b) = parse_bool(v) {
                        s.recent_apps = b;
                    }
                }
                "mac_shortcuts" => {
                    if let Some(b) = parse_bool(v) {
                        s.mac_shortcuts = b;
                    }
                }
                "accent" => s.accent = v.parse::<i32>().map(|n| if n == ACCENT_MULTI as i32 { ACCENT_MULTI } else { n.clamp(0, 7) as u8 }).unwrap_or(s.accent),
                "min_effect" => s.min_effect = v.parse::<i32>().map(|n| n.clamp(0, 1) as u8).unwrap_or(s.min_effect),
                "notch_idle" => s.notch_idle = v.parse::<i32>().map(|n| n.clamp(0, 2) as u8).unwrap_or(s.notch_idle),
                "greet_name" => s.greet_name = ShortText::new(v),
                "greet_text" => s.greet_text = ShortText::new(v),
                "lyrics_compact" => {
                    if let Some(b) = parse_bool(v) {
                        s.lyrics_compact = b;
                    }
                }
                "lyrics_online" => {
                    if let Some(b) = parse_bool(v) {
                        s.lyrics_online = b;
                    }
                }
                "notch_downloads" => {
                    if let Some(b) = parse_bool(v) {
                        s.notch_downloads = b;
                    }
                }
                "notch_send" => {
                    if let Some(b) = parse_bool(v) {
                        s.notch_send = b;
                    }
                }
                "icon_variant" => s.icon_variant = v.parse::<i32>().map(|n| n.clamp(0, 2) as u8).unwrap_or(s.icon_variant),
                "icon_tint" => s.icon_tint = v.parse::<i32>().map(|n| n.clamp(0, 4) as u8).unwrap_or(s.icon_tint),
                "clipboard_history" => {
                    if let Some(b) = parse_bool(v) {
                        s.clipboard_history = b;
                    }
                }
                "over_fullscreen" => {
                    if let Some(b) = parse_bool(v) {
                        s.above_fullscreen = b;
                    }
                }
                "dock_plates" => {
                    if let Some(b) = parse_bool(v) {
                        s.dock_plates = b;
                    }
                }
                "hide_desktop_icons" => {
                    if let Some(b) = parse_bool(v) {
                        s.hide_desktop_icons = b;
                    }
                }
                "stage_media" => {
                    if let Some(b) = parse_bool(v) {
                        s.stage_media = b;
                    }
                }
                "stage_titles" => {
                    if let Some(b) = parse_bool(v) {
                        s.stage_titles = b;
                    }
                }
                "stage_size" => s.stage_size = v.parse::<i32>().map(|n| n.clamp(70, 260) as u8).unwrap_or(s.stage_size),
                "stage_gap" => s.stage_gap = v.parse::<i32>().map(|n| n.clamp(0, 60) as u8).unwrap_or(s.stage_gap),
                "stage_count" => s.stage_count = v.parse::<i32>().map(|n| n.clamp(2, 10) as u8).unwrap_or(s.stage_count),
                "stage_speed" => s.stage_speed = v.parse::<i32>().map(|n| n.clamp(50, 200) as u8).unwrap_or(s.stage_speed),
                "stage_icons" => s.stage_icons = v.parse::<i32>().map(|n| n.clamp(0, 2) as u8).unwrap_or(s.stage_icons),
                "menubar_style" => {
                    if matches!(v.to_ascii_lowercase().as_str(), "glass" | "kaca") {
                        legacy_glass = true;
                    } else {
                        s.menubar_style = BarStyle::parse(v).unwrap_or(s.menubar_style);
                    }
                }
                "menubar_material" => {
                    if let Some(b) = parse_bool(v) {
                        s.menubar_material = b;
                    }
                }
                "menubar_glass" => {
                    if let Some(b) = parse_bool(v) {
                        s.menubar_glass = b;
                    }
                }
                "clock_24h" => {
                    if let Some(b) = parse_bool(v) {
                        s.clock_24h = b;
                    }
                }
                "win_shadow" => {
                    if let Some(b) = parse_bool(v) {
                        s.win_shadow = b;
                    }
                }
                "notch_hover_open" => {
                    if let Some(b) = parse_bool(v) {
                        s.notch_hover_open = b;
                    }
                }
                "notch_enabled" => {
                    if let Some(b) = parse_bool(v) {
                        s.notch_enabled = b;
                    }
                }
                "refraction" => {
                    if let Some(b) = parse_bool(v) {
                        s.refraction = b;
                    }
                }
                "anim" => {
                    if let Some(b) = parse_bool(v) {
                        s.anim = b;
                    }
                }
                "ui_scale" => s.ui_scale = v.trim_end_matches('%').parse::<i32>().map(|n| n.clamp(75, 200) as u8).unwrap_or(s.ui_scale),
                "notch_island" => {
                    if let Some(b) = parse_bool(v) {
                        s.notch_island = b;
                    }
                }
                "widget_style" => s.widget_style = v.parse::<u8>().map(|n| n.min(1)).unwrap_or(s.widget_style),
                "battery_pct" => s.battery_pct = v.parse::<u8>().map(|n| n.min(2)).unwrap_or(s.battery_pct),
                "rail" => s.rail = Behavior::parse(v).unwrap_or(s.rail),
                "dock_reveal" => s.dock_reveal = Reveal::parse(v).unwrap_or(s.dock_reveal),
                "rail_reveal" => s.rail_reveal = Reveal::parse(v).unwrap_or(s.rail_reveal),
                "notch_reveal" => s.notch_reveal = Reveal::parse(v).unwrap_or(s.notch_reveal),
                "notch_scale" | "notch_width" => {
                    if let Ok(n) = v.trim_end_matches('%').parse::<u16>() {
                        s.notch_scale = n.clamp(60, 180) as u8;
                        saw_notch_scale = true;
                    }
                }
                "notch_height" => {
                    if let Ok(n) = v.trim_end_matches('%').parse::<u16>() {
                        s.notch_height = n.clamp(70, 180) as u8;
                        saw_notch_height = true;
                    }
                }
                "rail_scale" => s.rail_scale = v.trim_end_matches('%').parse::<u16>().map(|n| n.clamp(70, 160) as u8).unwrap_or(s.rail_scale),
                "island_gap" => s.island_gap = v.parse::<u16>().map(|n| n.min(40) as u8).unwrap_or(s.island_gap),
                "update_check" => s.update_check = parse_bool(v).unwrap_or(s.update_check),
                "notch_glow" => s.notch_glow = parse_bool(v).unwrap_or(s.notch_glow),
                "pill_devices" => s.pill_devices = parse_bool(v).unwrap_or(s.pill_devices),
                "pill_notices" => s.pill_notices = parse_bool(v).unwrap_or(s.pill_notices),
                "pill_reminders" => s.pill_reminders = parse_bool(v).unwrap_or(s.pill_reminders),
                "todo_hotkey" => s.todo_hotkey = parse_bool(v).unwrap_or(s.todo_hotkey),
                "dock_preview" => s.dock_preview = parse_bool(v).unwrap_or(s.dock_preview),
                "search_hotkey" => s.search_hotkey = Hotkey::parse(v).unwrap_or(s.search_hotkey),
                "deck_screen" => s.deck_screen = Screen::parse(v).unwrap_or(s.deck_screen),
                "rail_screen" => s.rail_screen = Screen::parse(v).unwrap_or(s.rail_screen),
                "notch_screen" => s.notch_screen = Screen::parse(v).unwrap_or(s.notch_screen),
                "widget_screen" => s.widget_screen = Screen::parse(v).unwrap_or(s.widget_screen),
                "preset" => s.preset = Preset::parse(v).unwrap_or(s.preset),
                "visualizer" => s.visualizer = VisStyle::parse(v).unwrap_or(s.visualizer),
                "font" => s.font = FontChoice::parse(v).unwrap_or(s.font),
                "reduce_motion" => s.reduce_motion = parse_bool(v).unwrap_or(s.reduce_motion),
                "focus_cat" => s.focus_cat = v.parse::<u8>().ok().filter(|c| (*c as usize) < crate::focus::CATS).unwrap_or(s.focus_cat),
                "focus_mins" => s.focus_mins = crate::focus::parse_list(v, s.focus_mins, false),
                "break_mins" => s.break_mins = crate::focus::parse_list(v, s.break_mins, true),
                "focus_name" => s.focus_name = ShortText::new(v),
                "meeting_companion" => s.meeting_companion = parse_bool(v).unwrap_or(s.meeting_companion),
                "capsule" => s.capsule = Capsule::parse(v),
                "capsule_layout" => s.capsule_layout = Layout::parse(v).unwrap_or(s.capsule_layout),
                "capsule_decor" => s.capsule_decor = Decor::parse(v).unwrap_or(s.capsule_decor),
                "accent_rgb" => s.accent_rgb = parse_hex(v),
                "autostart" => {
                    if let Some(b) = parse_bool(v) {
                        s.autostart = b;
                    }
                }
                _ => {}
            }
        }
        if legacy_glass {
            s.menubar_glass = false;
        }
        if saw_notch_scale && !saw_notch_height {
            s.notch_height = s.notch_scale.clamp(70, 180);
        }
        s
    }

    pub fn serialize(&self) -> String {
        format!(
            "# BreezeOS settings\ntakeover={}\ndock={}\nnotch={}\nblur={}\nstyle={}\ntransparency={}\ntheme={}\nlang={}\ndock_material={}\ndock_round={}\nnotch_material={}\nnotch_round={}\nautostart={}\ndock_size={}\ndock_gap={}\ndock_zoom={}\ndock_margin={}\nanim={}\nrefraction={}\ndock_enabled={}\nnotch_enabled={}\nnotch_hover_open={}\nwin_shadow={}\nmenubar={}\nclock_24h={}\nmenubar_glass={}\nhot_tl={}\nhot_tr={}\nhot_bl={}\nhot_br={}\nstage={}\nswitcher={}\nstage_right={}\nstage_size={}\nstage_gap={}\nstage_count={}\nstage_speed={}\nstage_desktop_click={}\nstage_media={}\nstage_icons={}\nstage_titles={}\nhide_desktop_icons={}\nmenubar_style={}\ndock_plates={}\nover_fullscreen={}\nclipboard_history={}\nicon_style={}\nicon_tint={}\naccent={}\nmac_shortcuts={}\ndownloads_stack={}\nrecent_apps={}\ndock_preview={}\nsearch_hotkey={}\nicon_pack={}\nmenubar_material={}\nicon_variant={}\nmin_effect={}\nnotch_idle={}\ngreet_name={}\ngreet_text={}\nlyrics_compact={}\nlyrics_online={}\nnotch_downloads={}\nnotch_send={}\nui_scale={}\nnotch_island={}\nwidget_style={}\nbattery_pct={}\nrail={}\nnotch_scale={}\nnotch_height={}\nrail_scale={}\nisland_gap={}\nupdate_check={}\nnotch_glow={}\npill_devices={}\npill_notices={}\npill_reminders={}\ntodo_hotkey={}\ndeck_screen={}\nrail_screen={}\nnotch_screen={}\nwidget_screen={}\npreset={}\nvisualizer={}\nfont={}\nreduce_motion={}\nfocus_cat={}\nfocus_mins={}\nbreak_mins={}\nfocus_name={}\nmeeting_companion={}\ncapsule={}\ncapsule_layout={}\ncapsule_decor={}\naccent_rgb={}\ndock_reveal={}\nrail_reveal={}\nnotch_reveal={}\n",
            self.takeover,
            self.dock.key(),
            self.notch.key(),
            self.blur,
            self.style.key(),
            self.transparency,
            self.theme.key(),
            self.lang.key(),
            self.dock_material.key(),
            self.dock_round,
            self.notch_material.key(),
            self.notch_round,
            self.autostart,
            self.dock_size,
            self.dock_gap,
            self.dock_zoom,
            self.dock_margin,
            self.anim,
            self.refraction,
            self.dock_enabled,
            self.notch_enabled,
            self.notch_hover_open,
            self.win_shadow,
            self.menubar,
            self.clock_24h,
            self.menubar_glass,
            self.hot[0].key(),
            self.hot[1].key(),
            self.hot[2].key(),
            self.hot[3].key(),
            self.stage,
            self.switcher,
            self.stage_right,
            self.stage_size,
            self.stage_gap,
            self.stage_count,
            self.stage_speed,
            self.stage_desktop_click,
            self.stage_media,
            self.stage_icons,
            self.stage_titles,
            self.hide_desktop_icons,
            self.menubar_style.key(),
            self.dock_plates,
            self.above_fullscreen,
            self.clipboard_history,
            self.icon_style.key(),
            self.icon_tint,
            self.accent,
            self.mac_shortcuts,
            self.downloads_stack,
            self.recent_apps,
            self.dock_preview,
            self.search_hotkey.key(),
            self.icon_pack,
            self.menubar_material,
            self.icon_variant,
            self.min_effect,
            self.notch_idle,
            self.greet_name.as_str(),
            self.greet_text.as_str(),
            self.lyrics_compact,
            self.lyrics_online,
            self.notch_downloads,
            self.notch_send,
            self.ui_scale,
            self.notch_island,
            self.widget_style,
            self.battery_pct,
            self.rail.key(),
            self.notch_scale,
            self.notch_height,
            self.rail_scale,
            self.island_gap,
            self.update_check,
            self.notch_glow,
            self.pill_devices,
            self.pill_notices,
            self.pill_reminders,
            self.todo_hotkey,
            self.deck_screen.key(),
            self.rail_screen.key(),
            self.notch_screen.key(),
            self.widget_screen.key(),
            self.preset.key(),
            self.visualizer.key(),
            self.font.key(),
            self.reduce_motion,
            self.focus_cat,
            crate::focus::list_text(self.focus_mins),
            crate::focus::list_text(self.break_mins),
            self.focus_name.as_str(),
            self.meeting_companion,
            self.capsule.text(),
            self.capsule_layout.key(),
            self.capsule_decor.key(),
            self.accent_rgb.map(hex).unwrap_or_default(),
            self.dock_reveal.key(),
            self.rail_reveal.key(),
            self.notch_reveal.key()
        )
    }
}

pub fn parse_hex(v: &str) -> Option<[u8; 3]> {
    let h = v.trim().strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let b = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some([b(0)?, b(2)?, b(4)?])
}

pub fn hex(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub dark: bool,
    pub blur_sigma: f32,
    pub saturation: f32,
    pub tint: [u8; 4],
    pub rim: [u8; 4],
    pub solid: [u8; 4],
    pub highlight: f32,
    pub fg: [u8; 3],
    pub liquify: bool,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn look(s: &Settings, system_dark: bool, material: Material) -> Look {
    let dark = match (material, s.theme) {
        (Material::Black, _) => true,
        (_, Theme::Auto) => system_dark,
        (_, Theme::Dark) => true,
        (_, Theme::Light) => false,
    };
    let b = s.blur.min(100) as f32 / 100.0;
    let t = s.transparency.min(100) as f32 / 100.0;
    let (sigma_k, sat) = match s.style {
        BlurStyle::Frosted => (1.0, 1.25),
        BlurStyle::Clear => (0.45, 1.05),
        BlurStyle::Vivid => (0.85, 1.9),
    };
    let mut blur_sigma = lerp(1.0, 40.0, b) * sigma_k;
    let mut saturation: f32 = sat;
    let (tr, tg, tb, a_max, a_min) = if dark { (20u8, 20u8, 24u8, 232.0, 40.0) } else { (255, 255, 255, 215.0, 24.0) };
    let mut a = lerp(a_max, a_min, t);
    let highlight: f32;
    let (mut rim, mut solid, fg) = if dark {
        ([70u8, 255, 255, 255], [235u8, 28, 28, 32], [255u8, 255, 255])
    } else {
        ([150, 255, 255, 255], [235, 238, 240, 246], [24, 24, 28])
    };
    let (mut tr, mut tg, mut tb) = (tr, tg, tb);
    match material {
        Material::Glass => {
            a = if dark { lerp(120.0, 22.0, t) } else { lerp(80.0, 18.0, t) };
            if !dark {
                (tr, tg, tb) = (226, 231, 240);
            }
            blur_sigma = lerp(1.5, 7.0, b) * sigma_k;
            saturation = saturation.max(1.4);
            rim = [140, 255, 255, 255];
            highlight = 0.35;
        }
        Material::Blur => {
            a = if dark { lerp(100.0, 30.0, t) } else { lerp(120.0, 40.0, t) };
            (tr, tg, tb) = if dark { (26, 28, 30) } else { (240, 242, 246) };
            blur_sigma = lerp(8.0, 36.0, b);
            saturation = 1.1;
            rim = if dark { [34, 255, 255, 255] } else { [60, 255, 255, 255] };
            highlight = 0.2;
        }
        Material::Black => {
            (tr, tg, tb) = (10, 10, 12);
            a = lerp(255.0, 225.0, t);
            saturation = 1.0;
            blur_sigma *= 0.8;
            rim = [0, 255, 255, 255];
            solid = [255, 10, 10, 12];
            highlight = 0.0;
        }
    }
    let liquify = material == Material::Glass && s.refraction;
    let mut l = Look { dark, blur_sigma, saturation, tint: [a.round() as u8, tr, tg, tb], rim, solid, highlight, fg, liquify };
    crate::theme::tune(&mut l, s.preset);
    l
}

pub fn bar_look(s: &Settings, system_dark: bool) -> Look {
    let mut t = s.clone();
    match s.menubar_style {
        BarStyle::Light => t.theme = Theme::Light,
        BarStyle::Dark => t.theme = Theme::Dark,
    }
    let mut l = look(&t, system_dark, Material::Glass);
    l.tint[0] = 238;
    l.blur_sigma = l.blur_sigma.min(14.0);
    l.saturation = 1.0;
    l.highlight = 0.25;
    l.rim[0] = l.rim[0].min(60);
    l.liquify = false;
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_styles_force_theme_and_are_denser_than_glass() {
        let mut s = Settings::default();
        s.menubar_style = BarStyle::Light;
        let light = bar_look(&s, true);
        s.menubar_style = BarStyle::Dark;
        let dark = bar_look(&s, false);
        assert!(!light.dark && dark.dark);
        assert!(light.tint[0] >= 230 && dark.tint[0] >= 230);
        assert_eq!(BarStyle::parse("terang"), Some(BarStyle::Light));
        let again = Settings::parse(&s.serialize());
        assert_eq!(again.menubar_style, BarStyle::Dark);
    }

    #[test]
    fn legacy_glass_bar_becomes_transparent_and_light_glass_is_brighter() {
        let s = Settings::parse("menubar_glass=true
menubar_style=glass
");
        assert!(!s.menubar_glass, "latar kaca lama menjadi transparan");
        let mut st = Settings::default();
        st.theme = Theme::Light;
        let light = look(&st, false, Material::Glass);
        st.theme = Theme::Dark;
        let dark = look(&st, false, Material::Glass);
        assert!(light.tint[0] >= 40 && light.tint[0] <= 60, "kaca terang cukup terang: {}", light.tint[0]);
        assert!(dark.tint[0] >= 60);
    }

    #[test]
    fn roundtrip() {
        let s = Settings { takeover: false, dock: Behavior::Smart, notch: Behavior::Overlay, blur: 12, style: BlurStyle::Clear, transparency: 90, theme: Theme::Light, lang: Lang::Vi, dock_material: Material::Black, dock_round: 80, notch_material: Material::Glass, notch_round: 10, autostart: true, dock_size: 60, dock_gap: 10, dock_zoom: 200, dock_margin: 30, anim: false, refraction: false, dock_enabled: false, notch_enabled: false, notch_hover_open: false, win_shadow: false, menubar: true, clock_24h: false, menubar_glass: true, hot: [HotAction::Mission, HotAction::Launchpad, HotAction::Lock, HotAction::Stage], stage: true, switcher: false, ..Default::default() };
        assert_eq!(Settings::parse(&s.serialize()), s);
    }

    #[test]
    fn defaults_show_rail_visor_and_dock() {
        let d = Settings::default();
        assert!(d.takeover && d.menubar && d.notch_enabled && d.dock_enabled && d.notch.reserves());
        assert_eq!((d.dock, d.menubar_style, d.autostart), (Behavior::Smart, BarStyle::Dark, false));
    }

    #[test]
    fn dock_metrics_clamped() {
        let s = Settings::parse("dock_size=500
dock_zoom=5
dock_margin=900");
        assert_eq!((s.dock_size, s.dock_zoom, s.dock_margin), (72, 100, 60));
    }

    #[test]
    fn junk_is_ignored_and_clamped() {
        let s = Settings::parse("\u{feff}blur=999\ntransparency=-5\ndock=banana\nwat\ntheme=gelap\n#x=1");
        assert_eq!(s.blur, 100);
        assert_eq!(s.transparency, 0);
        assert_eq!(s.dock, Behavior::Smart);
        assert_eq!(s.theme, Theme::Dark);
    }

    #[test]
    fn behavior_maps_to_policy() {
        assert_eq!(Behavior::Fixed.mode(), Mode::Always);
        assert!(Behavior::Fixed.reserves());
        assert!(!Behavior::Overlay.reserves());
        assert_eq!(Behavior::Overlay.mode(), Mode::Always);
        assert_eq!(Behavior::Smart.mode(), Mode::AutoHide);
        assert!(!Behavior::Smart.reserves());
        for old in ["autohide", "peek", "smart"] {
            assert_eq!(Behavior::parse(old), Some(Behavior::Smart));
        }
    }

    #[test]
    fn materials_differ() {
        let s = Settings::default();
        let g = look(&s, false, Material::Glass);
        let k = look(&s, false, Material::Black);
        assert!(!g.dark && k.dark, "hitam selalu gelap");
        assert!(g.tint[0] < k.tint[0], "alfa: glass < black");
        assert_eq!(Material::parse("clear"), Some(Material::Glass));
    }

    #[test]
    fn round_factor_scales() {
        assert!((round_factor(50) - 1.0).abs() < 1e-6);
        assert!(round_factor(0) < 0.5 && round_factor(100) > 1.5);
    }

    #[test]
    fn blur_material_is_foggier_than_glass_and_follows_theme() {
        let s = Settings::default();
        let glass = look(&s, true, Material::Glass);
        let blur = look(&s, true, Material::Blur);
        assert!(blur.blur_sigma > glass.blur_sigma * 1.5, "blur {} vs glass {}", blur.blur_sigma, glass.blur_sigma);
        assert!(glass.liquify && !blur.liquify, "Crystal = gaya Liquify");
        assert!(glass.blur_sigma <= 7.0 && glass.rim[0] == 140, "Liquify: blur tipis, kilau atas 55%");
        let no_refr = Settings { refraction: false, ..Settings::default() };
        assert!(!look(&no_refr, true, Material::Glass).liquify, "refraksi dimatikan: kaca biasa");
        assert!(blur.dark && !look(&s, false, Material::Blur).dark);
        assert!(blur.tint[0] >= 60 && blur.rim[0] > 0 && blur.rim[0] < 60, "rim Blur sangat tipis");
        assert_eq!(Material::parse("blur"), Some(Material::Blur));
        assert_eq!(Material::Blur.key(), "blur");
        assert_eq!(Material::ALL.len(), 3);
    }

    #[test]
    fn notch_greeting_settings_round_trip() {
        let s = Settings::parse("notch_idle=1\ngreet_name=  BreezeOS =)  \ngreet_text=Semangat, \u{1F680}!\n");
        assert_eq!(s.notch_idle, 1);
        assert_eq!(s.greet_name.as_str(), "BreezeOS =)");
        let again = Settings::parse(&s.serialize());
        assert_eq!((again.notch_idle, again.greet_name, again.greet_text), (1, s.greet_name, s.greet_text));
        assert_eq!(Settings::default().notch_idle, 0);
        assert_eq!(Settings::parse(&Settings::parse("notch_idle=2\n").serialize()).notch_idle, 2, "musik & lirik");
        assert_eq!(Settings::parse("notch_idle=9\n").notch_idle, 2);
        let d = Settings::default();
        assert!(d.lyrics_compact && d.lyrics_online, "lirik online aktif bawaan (keputusan pengguna)");
        let off = Settings::parse(&Settings::parse("lyrics_online=false\nlyrics_compact=0\nnotch_downloads=0\n").serialize());
        assert!(!off.lyrics_online && !off.lyrics_compact && !off.notch_downloads);
        assert!(d.notch_downloads, "unduhan di notch aktif bawaan");
        assert!(d.notch_send, "zona Kirim aktif bawaan");
        assert!(!Settings::parse(&Settings::parse("notch_send=0\n").serialize()).notch_send);
        assert!(Settings::default().greet_name.is_empty());
        assert_eq!(ShortText::new("a\nb").as_str(), "ab");
        let long = "é".repeat(80);
        let t = ShortText::new(&long);
        assert!(t.as_str().len() <= 96 && t.as_str().chars().all(|c| c == 'é'));
    }

    #[test]
    fn multicolor_accent_and_icon_variant_round_trip() {
        let s = Settings::parse("accent=255\nicon_variant=2\n");
        assert_eq!(s.accent, ACCENT_MULTI);
        assert_eq!(s.icon_variant, 2);
        let again = Settings::parse(&s.serialize());
        assert_eq!((again.accent, again.icon_variant), (ACCENT_MULTI, 2));
        assert_eq!(Settings::parse("accent=9\n").accent, 7);
    }

    #[test]
    fn blur_style_option_is_gone_and_always_clear() {
        let s = Settings::parse("style=vivid
menubar_material=true
");
        assert_eq!(s.style, BlurStyle::Clear);
        assert!(s.menubar_material);
        assert!(Settings::default().style == BlurStyle::Clear);
    }

    #[test]
    fn look_responds_to_controls() {
        let mut s = Settings::default();
        let base = look(&s, true, Material::Glass);
        s.blur = 20;
        assert!(look(&s, true, Material::Glass).blur_sigma < base.blur_sigma);
        s = Settings::default();
        s.transparency = 100;
        assert!(look(&s, true, Material::Glass).tint[0] < base.tint[0]);
        s = Settings::default();
        s.theme = Theme::Light;
        assert!(!look(&s, true, Material::Glass).dark);
        s.theme = Theme::Auto;
        assert!(look(&s, true, Material::Glass).dark && !look(&s, false, Material::Glass).dark);
    }

    #[test]
    fn search_hotkey_requires_ctrl_alt_or_win_and_round_trips() {
        assert_eq!(Settings::default().search_hotkey, Hotkey::WIN_F);
        assert_eq!(Hotkey::WIN_F.label(), "Win + F");
        // Single keys or Shift-only combos must be rejected.
        assert_eq!(Hotkey::parse("F"), None);
        assert_eq!(Hotkey::parse("Space"), None);
        assert_eq!(Hotkey::parse("Shift+F"), None);
        assert_eq!(Hotkey::from_parts(HK_SHIFT, 0x46), None);
        // Valid combos with Win, Ctrl, Alt (+ optional Shift) work and round-trip.
        let hk = Hotkey::parse("Ctrl + Alt + Space").unwrap();
        assert_eq!(hk, Hotkey { mods: HK_CTRL | HK_ALT, vk: 0x20 });
        assert_eq!(Hotkey::parse(&hk.key()), Some(hk));
        let s = Settings::parse("search_hotkey=Alt+K\ndock_preview=false\n");
        assert_eq!(s.search_hotkey.label(), "Alt + K");
        assert!(!s.dock_preview);
        assert_eq!(Settings::parse(&s.serialize()), s);
    }

    #[test]
    fn scale_island_and_widget_style_round_trip() {
        let s = Settings::parse("ui_scale=125%\nnotch_island=1\nwidget_style=1\n");
        assert_eq!((s.ui_scale, s.notch_island, s.widget_style), (125, true, 1));
        let again = Settings::parse(&s.serialize());
        assert_eq!((again.ui_scale, again.notch_island, again.widget_style), (125, true, 1));
        assert_eq!(Settings::parse("ui_scale=999").ui_scale, 200);
        assert_eq!(Settings::default().ui_scale, 100);
        assert_eq!(Settings::parse(&Settings::parse("battery_pct=2").serialize()).battery_pct, 2);
        assert_eq!(Settings::parse("battery_pct=9").battery_pct, 2);
        assert_eq!(Settings::default().battery_pct, 2);
        let r = Settings::parse(&Settings::parse("rail=smart\nnotch_scale=120%\nisland_gap=18\n").serialize());
        assert_eq!((r.rail, r.notch_scale, r.notch_height, r.island_gap), (Behavior::Smart, 120, 120, 18));
        assert_eq!((Settings::parse("notch_scale=300").notch_scale, Settings::parse("island_gap=99").island_gap), (180, 40));
        let sep = Settings::parse(&Settings::parse("notch_scale=150\nnotch_height=85\nrail_scale=135\n").serialize());
        assert_eq!((sep.notch_scale, sep.notch_height, sep.rail_scale), (150, 85, 135));
        assert_eq!((Settings::default().rail, Settings::default().notch_scale, Settings::default().notch_height, Settings::default().rail_scale, Settings::default().island_gap), (Behavior::Fixed, 100, 100, 100, 7));
        let d = Settings::default();
        assert_eq!((d.update_check, d.notch_glow, d.pill_devices, d.pill_notices, d.pill_reminders, d.todo_hotkey), (true, true, true, false, true, true));
        let f = Settings::parse(&Settings::parse("update_check=0\nnotch_glow=0\npill_devices=0\npill_notices=1\npill_reminders=0\ntodo_hotkey=0\n").serialize());
        assert_eq!((f.update_check, f.notch_glow, f.pill_devices, f.pill_notices, f.pill_reminders, f.todo_hotkey), (false, false, false, true, false, false));
    }

    #[test]
    fn screen_choices_default_to_primary_and_round_trip() {
        let d = Settings::default();
        assert_eq!((d.deck_screen, d.rail_screen, d.notch_screen, d.widget_screen), (Screen::Primary, Screen::Primary, Screen::Primary, Screen::Primary));
        let f = Settings::parse(&Settings::parse("deck_screen=2\nrail_screen=1\nnotch_screen=primary\nwidget_screen=3\n").serialize());
        assert_eq!((f.deck_screen, f.rail_screen, f.notch_screen, f.widget_screen), (Screen::Nth(2), Screen::Nth(1), Screen::Primary, Screen::Nth(3)));
        assert_eq!(Settings::parse("deck_screen=9\n").deck_screen, Screen::Primary);
    }

    #[test]
    fn appearance_settings_round_trip() {
        let d = Settings::default();
        assert_eq!((d.preset, d.visualizer, d.font, d.reduce_motion), (Preset::Default, VisStyle::Bars, FontChoice::Inter, false));
        let f = Settings::parse(&Settings::parse("preset=neon\nvisualizer=wave_\nvisualizer=dots\nfont=mono\nreduce_motion=1\n").serialize());
        assert_eq!((f.preset, f.visualizer, f.font, f.reduce_motion), (Preset::Neon, VisStyle::Dots, FontChoice::Mono, true));
    }

    #[test]
    fn focus_settings_round_trip() {
        let d = Settings::default();
        assert_eq!((d.focus_cat, d.focus_mins, d.break_mins, d.focus_name.as_str()), (0, crate::focus::DEFAULT_FOCUS, crate::focus::DEFAULT_BREAK, ""));
        let f = Settings::parse(&Settings::parse("focus_cat=3\nfocus_mins=15,45,20,90\nbreak_mins=0,5,5,15\nfocus_name=Ngoding BreezeOS\n").serialize());
        assert_eq!((f.focus_cat, f.focus_mins, f.break_mins, f.focus_name.as_str()), (3, [15, 45, 20, 90], [0, 5, 5, 15], "Ngoding BreezeOS"));
        assert_eq!(Settings::parse("focus_cat=9\n").focus_cat, 0);
    }

    #[test]
    fn reveal_modes_round_trip_and_calm_is_always_slower() {
        let d = Settings::default();
        assert_eq!((d.dock_reveal, d.rail_reveal, d.notch_reveal), (Reveal::Quick, Reveal::Quick, Reveal::Quick));
        let f = Settings::parse(&Settings::parse("dock_reveal=santai\nrail_reveal=calm\nnotch_reveal=cepat\n").serialize());
        assert_eq!((f.dock_reveal, f.rail_reveal, f.notch_reveal), (Reveal::Calm, Reveal::Calm, Reveal::Quick));
        assert_eq!(Settings::parse("dock_reveal=999\n").dock_reveal, Reveal::Quick);
        for r in Reveal::ALL {
            assert_eq!(Reveal::parse(r.key()), Some(r));
        }
        let (q, c) = (Reveal::Quick, Reveal::Calm);
        assert!(c.deck_ms() >= q.deck_ms() + 250 && c.rail_ms() >= q.rail_ms() + 250);
        assert!(c.visor_edge_ms() > q.visor_edge_ms() && c.visor_open_ms() > q.visor_open_ms());
        assert_eq!((q.deck_ms(), q.rail_ms(), q.visor_edge_ms()), (200, 150, 200), "Cepat sama dengan perilaku lama");
        assert!(q.visor_open_ms() < 300, "Cepat membuka Visor lebih cepat dari 300 ms lama");
    }

    #[test]
    fn capsule_settings_round_trip() {
        let d = Settings::default();
        assert_eq!((d.capsule, d.capsule_layout, d.capsule_decor, d.accent_rgb), (Capsule::default(), Layout::Rotate, Decor::None, None));
        assert_eq!(Settings::parse(&d.serialize()), d);
        let f = Settings::parse(&Settings::parse("capsule=clock,system,agenda\ncapsule_layout=split\ncapsule_decor=edge\naccent_rgb=#7f77dd\n").serialize());
        assert_eq!(f.capsule.text(), "clock,system,agenda");
        assert_eq!((f.capsule_layout, f.capsule_decor, f.accent_rgb), (Layout::Split, Decor::Edge, Some([0x7F, 0x77, 0xDD])));
        assert_eq!(Settings::parse(&Settings::parse("capsule=\n").serialize()).capsule.on, 0);
        assert_eq!(Settings::parse("accent_rgb=#12345\n").accent_rgb, None);
        assert_eq!(Settings::parse("accent_rgb=red\n").accent_rgb, None);
        assert_eq!((parse_hex("#FFFFFF"), hex([1, 171, 255])), (Some([255, 255, 255]), "#01ABFF".to_string()));
    }
}
