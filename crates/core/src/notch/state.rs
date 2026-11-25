#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Play,
    Shelf,
    Timer,
    Todo,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Play, Tab::Shelf, Tab::Timer, Tab::Todo];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Play => "Putar",
            Tab::Shelf => "Rak",
            Tab::Timer => "Timer",
            Tab::Todo => "Tugas",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenOn {
    Hover,
    Click,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Rest,
    Peek,
    Open(Tab),
    Drop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Enter,
    Leave,
    Dwell,
    Click,
    SelectTab(Tab),
    DragEnter,
    DragLeave,
    Dropped,
    Escape,
}

pub fn step(mode: Mode, ev: Event, open_on: OpenOn, hovering: bool) -> Mode {
    let rest = if hovering { Mode::Peek } else { Mode::Rest };
    match (mode, ev) {
        (_, Event::DragEnter) => Mode::Drop,
        (Mode::Drop, Event::DragLeave) => rest,
        (Mode::Drop, Event::Dropped) => Mode::Open(Tab::Shelf),
        (Mode::Drop, _) => Mode::Drop,
        (Mode::Rest, Event::Enter) => Mode::Peek,
        (Mode::Peek, Event::Leave) => Mode::Rest,
        (Mode::Peek, Event::Dwell) if open_on == OpenOn::Hover => Mode::Open(Tab::Play),
        (Mode::Rest | Mode::Peek, Event::Click) => Mode::Open(Tab::Play),
        (Mode::Open(_), Event::SelectTab(t)) => Mode::Open(t),
        (Mode::Open(_), Event::Leave) => Mode::Rest,
        (Mode::Open(_), Event::Escape) => rest,
        (m, _) => m,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity {
    TimerEnding,
    Download,
    Pill,
    Meeting,
    Player,
    Lyrics,
    Music,
    Charging,
    Timer,
    Shelf,
    Glance,
    GlancePair,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Live {
    pub media: bool,
    pub playing: bool,
    pub lyric_line: bool,
    pub lyrics_on: bool,
    pub timer_left_s: Option<i64>,
    pub charging_fresh: bool,
    pub shelf: usize,
    pub glance: bool,
    pub download: bool,
    pub wide_music: bool,
    pub pill: bool,
    pub meeting: bool,
    pub glance_pair: bool,
}

pub fn activity(l: &Live) -> Option<Activity> {
    if l.timer_left_s.is_some_and(|s| s <= 60) {
        return Some(Activity::TimerEnding);
    }
    if l.download {
        return Some(Activity::Download);
    }
    if l.pill {
        return Some(Activity::Pill);
    }
    if l.meeting {
        return Some(Activity::Meeting);
    }
    if l.media && l.wide_music {
        return Some(Activity::Player);
    }
    if l.media && l.playing && l.lyrics_on && l.lyric_line {
        return Some(Activity::Lyrics);
    }
    if l.media {
        return Some(Activity::Music);
    }
    if l.charging_fresh {
        return Some(Activity::Charging);
    }
    if l.timer_left_s.is_some() {
        return Some(Activity::Timer);
    }
    if l.shelf > 0 {
        return Some(Activity::Shelf);
    }
    if l.glance {
        return Some(if l.glance_pair { Activity::GlancePair } else { Activity::Glance });
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Rest,
    Compact(Activity),
    Open(Tab),
    Drop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    pub view: View,
    pub peek: bool,
    pub rows: u8,
}

pub const TODO_MAX_ROWS: u8 = 7;
pub const TODO_ROW: f32 = 38.0;

pub fn todo_height(rows: u8) -> f32 {
    let list = rows.min(TODO_MAX_ROWS + 1) as f32 * TODO_ROW;
    48.0 + list.max(140.0) + 30.0
}

pub fn look(mode: Mode, act: Option<Activity>) -> Look {
    let collapsed = |peek| Look { view: act.map_or(View::Rest, View::Compact), peek, rows: 0 };
    match mode {
        Mode::Rest => collapsed(false),
        Mode::Peek => collapsed(true),
        Mode::Open(t) => Look { view: View::Open(t), peek: false, rows: 0 },
        Mode::Drop => Look { view: View::Drop, peek: false, rows: 0 },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub w: f32,
    pub h: f32,
    pub radius: f32,
}

pub const PEEK_GROW: (f32, f32) = (16.0, 4.0);

pub fn size(l: Look, dpi: f32) -> Size {
    let (mut w, mut h, mut r): (f32, f32, f32) = match l.view {
        View::Rest => (184.0, 30.0, 15.0),
        View::Compact(Activity::Lyrics) => (420.0, 32.0, 16.0),
        View::Compact(Activity::Glance) => (220.0, 30.0, 15.0),
        View::Compact(Activity::GlancePair) => (300.0, 30.0, 15.0),
        View::Compact(Activity::Download) => (300.0, 36.0, 18.0),
        View::Compact(Activity::Pill) => (330.0, 36.0, 18.0),
        View::Compact(Activity::Player) => (540.0, 100.0, 30.0),
        View::Compact(_) => (292.0, 30.0, 15.0),
        View::Open(Tab::Play) => (600.0, 226.0, 28.0),
        View::Open(Tab::Todo) => (600.0, todo_height(l.rows), 28.0),
        View::Open(_) => (600.0, 188.0, 28.0),
        View::Drop => (520.0, 120.0, 28.0),
    };
    if l.peek && matches!(l.view, View::Rest | View::Compact(_)) {
        w += PEEK_GROW.0;
        h += PEEK_GROW.1;
        r += PEEK_GROW.1 / 2.0;
    }
    Size { w: w * dpi, h: h * dpi, radius: r * dpi }
}

pub fn size_wh(l: Look, dpi: f32, kw: f32, kh: f32) -> Size {
    let base = size(l, dpi);
    let w = base.w * kw;
    let h = base.h * kh;
    let radius = (base.radius * kw.min(kh)).min(w / 2.0).min(h / 2.0);
    Size { w, h, radius }
}

pub fn shadow(l: Look) -> (f32, f32, f32) {
    let strong = l.peek || matches!(l.view, View::Open(_) | View::Drop);
    if strong {
        (18.0, 4.0, 0.34)
    } else {
        (18.0, 4.0, 0.24)
    }
}

pub mod motion {
    pub const PEEK_RATIO: f32 = 0.9;
    pub const PEEK_PERIOD_MS: u32 = 34;
    pub const OPEN_RATIO: f32 = 0.78;
    pub const OPEN_PERIOD_MS: u32 = 68;
    pub const CLOSE_RATIO: f32 = 1.0;
    pub const CLOSE_PERIOD_MS: u32 = 45;
    pub const SWAP_RATIO: f32 = 0.84;
    pub const SWAP_PERIOD_MS: u32 = 55;
    pub const CONTENT_DELAY_MS: u32 = 60;
    pub const CONTENT_FADE_MS: u32 = 170;
    pub const CONTENT_SCALE_FROM: f32 = 0.96;
    pub const CONTENT_RISE_DP: f32 = 6.0;
    pub const CLOSE_FADE_MS: u32 = 90;
    pub const SWAP_FADE_MS: u32 = 120;
    pub const LYRIC_SWAP_MS: u32 = 220;
    pub const LYRIC_RISE_DP: f32 = 8.0;
    pub const LEAVE_GRACE_MS: u32 = 260;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_peeks_then_dwell_opens_play_and_leaving_closes() {
        let m = step(Mode::Rest, Event::Enter, OpenOn::Hover, true);
        assert_eq!(m, Mode::Peek);
        let m = step(m, Event::Dwell, OpenOn::Hover, true);
        assert_eq!(m, Mode::Open(Tab::Play));
        assert_eq!(step(m, Event::Leave, OpenOn::Hover, false), Mode::Rest);
    }

    #[test]
    fn click_mode_ignores_dwell_but_opens_on_click() {
        let m = step(Mode::Peek, Event::Dwell, OpenOn::Click, true);
        assert_eq!(m, Mode::Peek);
        assert_eq!(step(m, Event::Click, OpenOn::Click, true), Mode::Open(Tab::Play));
    }

    #[test]
    fn dragging_files_shows_drop_zones_and_dropping_opens_the_shelf() {
        for from in [Mode::Rest, Mode::Peek, Mode::Open(Tab::Timer)] {
            assert_eq!(step(from, Event::DragEnter, OpenOn::Hover, false), Mode::Drop);
        }
        assert_eq!(step(Mode::Drop, Event::Leave, OpenOn::Hover, false), Mode::Drop, "keluar biasa diabaikan selama menyeret");
        assert_eq!(step(Mode::Drop, Event::DragLeave, OpenOn::Hover, false), Mode::Rest);
        assert_eq!(step(Mode::Drop, Event::DragLeave, OpenOn::Hover, true), Mode::Peek);
        assert_eq!(step(Mode::Drop, Event::Dropped, OpenOn::Hover, true), Mode::Open(Tab::Shelf));
    }

    #[test]
    fn tabs_switch_only_while_open_and_escape_collapses() {
        assert_eq!(step(Mode::Open(Tab::Play), Event::SelectTab(Tab::Shelf), OpenOn::Hover, true), Mode::Open(Tab::Shelf));
        assert_eq!(step(Mode::Rest, Event::SelectTab(Tab::Shelf), OpenOn::Hover, false), Mode::Rest);
        assert_eq!(step(Mode::Open(Tab::Shelf), Event::Escape, OpenOn::Hover, true), Mode::Peek);
        assert_eq!(step(Mode::Open(Tab::Shelf), Event::Escape, OpenOn::Hover, false), Mode::Rest);
    }

    #[test]
    fn activity_priority() {
        let base = Live { media: true, playing: true, lyric_line: true, lyrics_on: true, timer_left_s: Some(30), charging_fresh: true, shelf: 2, glance: true, download: true, wide_music: false, pill: false, meeting: false, glance_pair: false };
        assert_eq!(activity(&base), Some(Activity::TimerEnding));
        assert_eq!(activity(&Live { timer_left_s: Some(300), ..base }), Some(Activity::Download), "unduhan di atas lirik dan musik");
        let l = Live { timer_left_s: Some(300), download: false, ..base };
        assert_eq!(activity(&Live { wide_music: true, ..l }), Some(Activity::Player), "lebar menggantikan lirik/musik ringkas");
        assert_eq!(activity(&Live { wide_music: true, playing: false, ..l }), Some(Activity::Player), "dijeda: tetap tampil");
        assert_eq!(activity(&Live { wide_music: true, media: false, playing: false, charging_fresh: false, timer_left_s: None, shelf: 0, glance: false, ..l }), None, "tanpa lagu: notch kosong");
        assert_eq!(activity(&Live { wide_music: true, download: true, ..l }), Some(Activity::Download));
        assert_eq!(activity(&Live { pill: true, ..l }), Some(Activity::Pill), "pill di atas musik, di bawah unduhan");
        assert_eq!(activity(&Live { pill: true, download: true, ..l }), Some(Activity::Download));
        assert_eq!(activity(&l), Some(Activity::Lyrics));
        assert_eq!(activity(&Live { lyrics_on: false, ..l }), Some(Activity::Music));
        assert_eq!(activity(&Live { playing: false, ..l }), Some(Activity::Music), "dijeda: lirik ringkas tidak tampil");
        let no_media = Live { media: false, playing: false, ..l };
        assert_eq!(activity(&no_media), Some(Activity::Charging));
        assert_eq!(activity(&Live { charging_fresh: false, ..no_media }), Some(Activity::Timer));
        assert_eq!(activity(&Live { charging_fresh: false, timer_left_s: None, ..no_media }), Some(Activity::Shelf));
        let idle = Live { charging_fresh: false, timer_left_s: None, shelf: 0, ..no_media };
        assert_eq!(activity(&idle), Some(Activity::Glance), "sapaan & info hanya bila tidak ada apa-apa lagi");
        assert_eq!(activity(&Live { glance: false, ..idle }), None);
        assert_eq!(activity(&Live { glance_pair: true, ..idle }), Some(Activity::GlancePair));
        assert_eq!(activity(&Live { glance_pair: true, glance: false, ..idle }), None);
        assert_eq!(activity(&Live::default()), None);
    }

    #[test]
    fn sizes_follow_the_view_and_peek_grows_only_when_collapsed() {
        let rest = size(look(Mode::Rest, None), 1.0);
        let peek = size(look(Mode::Peek, None), 1.0);
        assert_eq!((rest.w, rest.h), (184.0, 30.0));
        assert_eq!((peek.w - rest.w, peek.h - rest.h), PEEK_GROW);
        let glance = size(look(Mode::Rest, Some(Activity::Glance)), 1.0);
        assert!(glance.w > rest.w && glance.w < 240.0 && glance.h == rest.h, "sapaan & info sedikit lebih lebar dari notch kosong");
        let pair = size(look(Mode::Rest, Some(Activity::GlancePair)), 1.0);
        assert!(pair.w > glance.w && pair.h == glance.h, "dua modul butuh Capsule lebih lebar");
        let player = size(look(Mode::Rest, Some(Activity::Player)), 1.0);
        assert!(player.w > 500.0 && player.w < 600.0 && player.h > 80.0 && player.h < 214.0, "lebar: lebih kecil dari panel terbuka");
        let dl = size(look(Mode::Rest, Some(Activity::Download)), 1.0);
        assert!(dl.h > rest.h && dl.h <= 40.0, "unduhan sedikit lebih tinggi untuk busur + angka");
        let music = size(look(Mode::Rest, Some(Activity::Music)), 1.0);
        let lyr = size(look(Mode::Rest, Some(Activity::Lyrics)), 1.0);
        assert!(music.w > rest.w && music.h == rest.h);
        assert!(lyr.w > music.w && lyr.h >= music.h && lyr.h < 40.0, "lirik ringkas: satu baris yang lebih lebar");
        let open = size(look(Mode::Open(Tab::Play), Some(Activity::Music)), 1.0);
        assert_eq!((open.w, open.h, open.radius), (600.0, 226.0, 28.0));
        for t in Tab::ALL {
            assert_eq!(size(look(Mode::Open(t), None), 1.0).w, 600.0);
        }
        let few = size(Look { rows: 2, ..look(Mode::Open(Tab::Todo), None) }, 1.0);
        let many = size(Look { rows: 6, ..look(Mode::Open(Tab::Todo), None) }, 1.0);
        let capped = size(Look { rows: 30, ..look(Mode::Open(Tab::Todo), None) }, 1.0);
        assert_eq!(few.h, 48.0 + 140.0 + 30.0);
        assert!(many.h > few.h && capped.h == todo_height(TODO_MAX_ROWS + 1) && capped.h <= 384.0);
        let b = size(look(Mode::Open(Tab::Play), None), 1.5);
        assert_eq!((b.w, b.radius), (900.0, 42.0));
        let wh = size_wh(look(Mode::Rest, None), 1.0, 1.5, 0.8);
        assert_eq!((wh.w, wh.h), (184.0 * 1.5, 30.0 * 0.8));
        assert!(wh.radius <= wh.w / 2.0 && wh.radius <= wh.h / 2.0);
    }

    #[test]
    fn shadow_keeps_its_shape_and_only_deepens_when_active() {
        let (b0, y0, o0) = shadow(look(Mode::Rest, None));
        let (b1, y1, o1) = shadow(look(Mode::Open(Tab::Play), None));
        assert!(o0 < 0.3 && o1 > o0);
        assert_eq!((b0, y0), (b1, y1));
    }

    #[test]
    fn meeting_sits_between_pills_and_music() {
        let l = Live { media: true, playing: true, meeting: true, ..Live::default() };
        assert_eq!(activity(&l), Some(Activity::Meeting));
        assert_eq!(activity(&Live { pill: true, ..l }), Some(Activity::Pill));
        assert_eq!(activity(&Live { meeting: false, ..l }), Some(Activity::Music));
    }
}
