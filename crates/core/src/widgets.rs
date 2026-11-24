#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Clock,
    WorldClock,
    Calendar,
    Month,
    Digital,
    Battery,
    System,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Clock,
    Calendar,
    Battery,
    System,
}

impl Category {
    pub const ALL: [Category; 4] = [Category::Clock, Category::Calendar, Category::Battery, Category::System];

    pub fn title(self) -> &'static str {
        match self {
            Category::Clock => "Jam",
            Category::Calendar => "Kalender",
            Category::Battery => "Baterai",
            Category::System => "Sistem",
        }
    }
}

impl Kind {
    pub const ALL: [Kind; 7] = [Kind::Clock, Kind::WorldClock, Kind::Digital, Kind::Calendar, Kind::Month, Kind::Battery, Kind::System];

    pub fn key(self) -> &'static str {
        match self {
            Kind::Clock => "clock",
            Kind::WorldClock => "world",
            Kind::Calendar => "calendar",
            Kind::Month => "month",
            Kind::Digital => "digital",
            Kind::Battery => "battery",
            Kind::System => "system",
        }
    }

    pub fn parse(s: &str) -> Option<Kind> {
        Kind::ALL.iter().copied().find(|k| k.key() == s.trim())
    }

    pub fn cells(self) -> (u32, u32) {
        match self {
            Kind::WorldClock => (2, 1),
            _ => (1, 1),
        }
    }

    pub fn category(self) -> Category {
        match self {
            Kind::Clock | Kind::WorldClock | Kind::Digital => Category::Clock,
            Kind::Calendar | Kind::Month => Category::Calendar,
            Kind::Battery => Category::Battery,
            Kind::System => Category::System,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Kind::Clock => "Jam",
            Kind::WorldClock => "Jam Dunia",
            Kind::Calendar => "Hari Ini",
            Kind::Month => "Bulan",
            Kind::Digital => "Jam Digital",
            Kind::Battery => "Status",
            Kind::System => "Monitor Sistem",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            Kind::Clock => "Lihat waktu saat ini.",
            Kind::WorldClock => "Lihat waktu di kota-kota lain.",
            Kind::Calendar => "Lihat hari dan tanggal hari ini.",
            Kind::Month => "Lihat kalender bulan ini.",
            Kind::Digital => "Lihat jam dan tanggal dalam angka.",
            Kind::Battery => "Lihat status baterai perangkat.",
            Kind::System => "Lihat pemakaian CPU dan memori.",
        }
    }

    pub fn dark_card(self) -> bool {
        matches!(self, Kind::Clock | Kind::WorldClock | Kind::System)
    }

    pub fn frosted_card(self) -> bool {
        matches!(self, Kind::Battery)
    }
}

pub const CELL: f32 = 164.0;
pub const GAP: f32 = 16.0;

pub fn size_of(k: Kind, dpi: f32) -> (f32, f32) {
    let (cw, ch) = k.cells();
    ((cw as f32 * CELL + (cw - 1) as f32 * GAP) * dpi, (ch as f32 * CELL + (ch - 1) as f32 * GAP) * dpi)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
}

pub fn to_text(list: &[Placed]) -> String {
    list.iter().map(|p| format!("{} {} {}\n", p.kind.key(), p.x, p.y)).collect()
}

pub fn from_text(t: &str) -> Vec<Placed> {
    t.trim_start_matches('\u{feff}')
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let kind = Kind::parse(it.next()?)?;
            let x = it.next()?.parse().ok()?;
            let y = it.next()?.parse().ok()?;
            Some(Placed { kind, x, y })
        })
        .collect()
}

pub type Rect = (i32, i32, i32, i32);

fn overlaps(a: Rect, b: Rect, gap: i32) -> bool {
    a.0 < b.0 + b.2 + gap && b.0 < a.0 + a.2 + gap && a.1 < b.1 + b.3 + gap && b.1 < a.1 + a.3 + gap
}

pub fn snap(v: i32, origin: i32, step: i32) -> i32 {
    if step <= 0 {
        return v;
    }
    origin + ((v - origin) as f32 / step as f32).round() as i32 * step
}

pub fn free_spot(taken: &[Rect], size: (i32, i32), area: Rect, step: i32, gap: i32) -> (i32, i32) {
    let step = step.max(1);
    let mut x = area.0;
    while x + size.0 <= area.0 + area.2 {
        let mut y = area.1;
        while y + size.1 <= area.1 + area.3 {
            let r = (x, y, size.0, size.1);
            if !taken.iter().any(|t| overlaps(r, *t, gap)) {
                return (x, y);
            }
            y += step;
        }
        x += step;
    }
    (area.0, area.1)
}

pub fn nearest_free(taken: &[Rect], size: (i32, i32), want: (i32, i32), area: Rect, step: i32, gap: i32) -> (i32, i32) {
    let step = step.max(1);
    let clamp = |p: (i32, i32)| (p.0.clamp(area.0, (area.0 + area.2 - size.0).max(area.0)), p.1.clamp(area.1, (area.1 + area.3 - size.1).max(area.1)));
    let want = clamp(want);
    for r in 0..80i32 {
        let mut best: Option<((i32, i32), i64)> = None;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let p = clamp((want.0 + dx * step, want.1 + dy * step));
                if taken.iter().any(|t| overlaps((p.0, p.1, size.0, size.1), *t, gap)) {
                    continue;
                }
                let d = ((p.0 - want.0) as i64).pow(2) + ((p.1 - want.1) as i64).pow(2);
                if best.map_or(true, |b| d < b.1) {
                    best = Some((p, d));
                }
            }
        }
        if let Some((p, _)) = best {
            return p;
        }
    }
    want
}

fn cell_hit(a: Rect, b: Rect) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

pub fn nearest_cell(taken: &[Rect], size: (i32, i32), want: (i32, i32), grid: (i32, i32)) -> Option<(i32, i32)> {
    let (maxc, maxr) = ((grid.0 - size.0).max(0), (grid.1 - size.1).max(0));
    let want = (want.0.clamp(0, maxc), want.1.clamp(0, maxr));
    let mut best: Option<((i32, i32), i32)> = None;
    for r in 0..=maxr {
        for c in 0..=maxc {
            if taken.iter().any(|t| cell_hit((c, r, size.0, size.1), *t)) {
                continue;
            }
            let d = (c - want.0).pow(2) + (r - want.1).pow(2);
            if best.map_or(true, |b| d < b.1 || (d == b.1 && (r, c) < (b.0 .1, b.0 .0))) {
                best = Some(((c, r), d));
            }
        }
    }
    best.map(|b| b.0)
}

pub fn first_cell(taken: &[Rect], size: (i32, i32), grid: (i32, i32)) -> Option<(i32, i32)> {
    for c in 0..=(grid.0 - size.0).max(0) {
        for r in 0..=(grid.1 - size.1).max(0) {
            if !taken.iter().any(|t| cell_hit((c, r, size.0, size.1), *t)) {
                return Some((c, r));
            }
        }
    }
    None
}

pub struct City {
    pub name: &'static str,
    pub offset_min: i32,
}

pub const CITIES: [City; 4] = [
    City { name: "Jakarta", offset_min: 7 * 60 },
    City { name: "Singapura", offset_min: 8 * 60 },
    City { name: "Tokyo", offset_min: 9 * 60 },
    City { name: "Dubai", offset_min: 4 * 60 },
];

pub fn city_time(utc_min: i32, local_offset_min: i32, city_offset_min: i32) -> (u32, u32, i32, i32) {
    let local = utc_min + local_offset_min;
    let there = utc_min + city_offset_min;
    let day = |m: i32| m.div_euclid(1440);
    let mm = there.rem_euclid(1440);
    ((mm / 60) as u32, (mm % 60) as u32, day(there) - day(local), (city_offset_min - local_offset_min) / 60)
}

pub fn hands(h: u32, m: u32, s: u32) -> (f32, f32, f32) {
    let s = s as f32;
    let m = m as f32 + s / 60.0;
    let h = (h % 12) as f32 + m / 60.0;
    (h * 30.0, m * 6.0, s * 6.0)
}

pub fn cpu_percent(prev: (u64, u64), now: (u64, u64)) -> u8 {
    let idle = now.0.saturating_sub(prev.0);
    let total = now.1.saturating_sub(prev.1);
    if total == 0 {
        return 0;
    }
    (100 - (idle * 100 / total).min(100)) as u8
}

pub fn ring(n: u32, thickness: f32, frac: f32, fg: [u8; 3], track: [u8; 4]) -> Vec<u8> {
    let mut out = vec![0u8; (n * n * 4) as usize];
    let c = n as f32 / 2.0;
    let (ro, ri) = (c, c - thickness);
    let sweep = frac.clamp(0.0, 1.0) * std::f32::consts::TAU;
    const SS: u32 = 3;
    for y in 0..n {
        for x in 0..n {
            let (mut on, mut tr) = (0u32, 0u32);
            for sy in 0..SS {
                for sx in 0..SS {
                    let px = x as f32 + (sx as f32 + 0.5) / SS as f32 - c;
                    let py = y as f32 + (sy as f32 + 0.5) / SS as f32 - c;
                    let d = (px * px + py * py).sqrt();
                    if d > ro || d < ri {
                        continue;
                    }
                    let a = px.atan2(-py).rem_euclid(std::f32::consts::TAU);
                    if a <= sweep {
                        on += 1;
                    } else {
                        tr += 1;
                    }
                }
            }
            let n2 = (SS * SS) as f32;
            let (fa, ta) = (on as f32 / n2, tr as f32 / n2 * track[3] as f32 / 255.0);
            let a = (fa + ta).min(1.0);
            let i = ((y * n + x) * 4) as usize;
            for k in 0..3 {
                out[i + 2 - k] = (fg[k] as f32 * fa + track[k] as f32 * ta).round().min(255.0) as u8;
            }
            out[i + 3] = (a * 255.0).round() as u8;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placed_round_trips_and_skips_junk() {
        let list = vec![Placed { kind: Kind::Clock, x: 10, y: -20 }, Placed { kind: Kind::WorldClock, x: 300, y: 40 }];
        assert_eq!(from_text(&to_text(&list)), list);
        assert_eq!(from_text("nope 1 2\nclock x 3\nbattery 5 6\n"), vec![Placed { kind: Kind::Battery, x: 5, y: 6 }]);
    }

    #[test]
    fn free_spot_fills_column_then_moves_right() {
        let area = (0, 0, 1000, 400);
        let a = free_spot(&[], (164, 164), area, 20, 16);
        assert_eq!(a, (0, 0));
        let b = free_spot(&[(0, 0, 164, 164)], (164, 164), area, 20, 16);
        assert_eq!(b.0, 0);
        assert!(b.1 >= 180);
        let c = free_spot(&[(0, 0, 164, 164), (0, b.1, 164, 164)], (164, 164), area, 20, 16);
        assert!(c.0 >= 180);
    }

    #[test]
    fn nearest_free_slides_off_an_occupied_spot() {
        let area = (0, 0, 1000, 1000);
        assert_eq!(nearest_free(&[], (100, 100), (300, 300), area, 10, 10), (300, 300));
        let p = nearest_free(&[(300, 300, 100, 100)], (100, 100), (310, 300), area, 10, 10);
        assert!(!overlaps((p.0, p.1, 100, 100), (300, 300, 100, 100), 10));
        assert!((p.0 - 310).abs() <= 130 && (p.1 - 300).abs() <= 130);
        assert_eq!(nearest_free(&[], (100, 100), (2000, -50), area, 10, 10), (900, 0));
    }

    #[test]
    fn cells_never_overlap_and_stay_on_grid() {
        let grid = (8, 4);
        assert_eq!(first_cell(&[], (1, 1), grid), Some((0, 0)));
        assert_eq!(first_cell(&[(0, 0, 1, 1)], (1, 1), grid), Some((0, 1)));
        assert_eq!(first_cell(&[(0, 0, 1, 4)], (2, 1), grid), Some((1, 0)));
        assert_eq!(nearest_cell(&[(2, 1, 1, 1)], (1, 1), (2, 1), grid).map(|p| (p.0 - 2).abs() + (p.1 - 1).abs()), Some(1));
        assert_eq!(nearest_cell(&[], (2, 1), (20, -3), grid), Some((6, 0)));
        let full: Vec<Rect> = (0..8).flat_map(|c| (0..4).map(move |r| (c, r, 1, 1))).collect();
        assert_eq!(nearest_cell(&full, (1, 1), (0, 0), grid), None);
    }

    #[test]
    fn snap_rounds_to_grid() {
        assert_eq!(snap(27, 10, 20), 30);
        assert_eq!(snap(9, 10, 20), 10);
        assert_eq!(snap(-15, 0, 20), -20);
    }

    #[test]
    fn city_time_crosses_midnight() {
        let utc = 20 * 60 + 30;
        let (h, m, day, diff) = city_time(utc, 7 * 60, 9 * 60);
        assert_eq!((h, m, day, diff), (5, 30, 0, 2));
        let (h, _, day, diff) = city_time(18 * 60, 7 * 60, 4 * 60);
        assert_eq!((h, day, diff), (22, -1, -3));
    }

    #[test]
    fn hand_angles() {
        assert_eq!(hands(3, 0, 0), (90.0, 0.0, 0.0));
        let (h, m, s) = hands(15, 30, 30);
        assert!((h - 105.25).abs() < 0.01 && (m - 183.0).abs() < 0.01 && (s - 180.0).abs() < 0.01);
    }

    #[test]
    fn cpu_percent_from_deltas() {
        assert_eq!(cpu_percent((100, 1000), (150, 1100)), 50);
        assert_eq!(cpu_percent((0, 0), (0, 0)), 0);
    }

    #[test]
    fn ring_draws_progress_from_the_top() {
        let n = 40;
        let px = ring(n, 6.0, 0.3, [0, 255, 0], [255, 255, 255, 60]);
        let at = |x: u32, y: u32| &px[((y * n + x) * 4) as usize..((y * n + x) * 4 + 4) as usize];
        assert!(at(36, 20)[1] > 200, "kanan (jam 3) masuk progres");
        assert!(at(4, 20)[1] < 120, "kiri (jam 9) belum");
        assert_eq!(at(20, 20)[3], 0, "tengah kosong");
    }
}
