#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

// Two-phase macOS-style Genie curve: the lower window pinches into a cosine S-curve funnel
// anchored to the target icon edges, then slides smoothly down inside those curved walls.
const PINCH_END: f32 = 0.42;
const SLIDE_START: f32 = 0.22;
const SLIDE_LAG: f32 = 0.35;

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn smoother(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

fn s_curve(u: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    0.5 * (1.0 - (u * std::f32::consts::PI).cos())
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn progress(i: usize, n: usize, tau: f32) -> f32 {
    let v = (i as f32 + 0.5) / n.max(1) as f32;
    let slide_t = ((tau - SLIDE_START) / (1.0 - SLIDE_START)).clamp(0.0, 1.0);
    (slide_t * (1.0 + SLIDE_LAG) - SLIDE_LAG * (1.0 - v)).clamp(0.0, 1.0)
}

pub fn slice(i: usize, n: usize, tau: f32, win: Rect, icon: Rect) -> ((f32, f32), (f32, f32)) {
    let n = n.max(1);
    let sh = win.h / n as f32;
    let pinch = smoother(tau / PINCH_END);
    let slide_t = ((tau - SLIDE_START) / (1.0 - SLIDE_START)).clamp(0.0, 1.0);
    let y_of = |j: usize| -> f32 {
        let v = j as f32 / n as f32;
        let p = (slide_t * (1.0 + SLIDE_LAG) - SLIDE_LAG * (1.0 - v)).clamp(0.0, 1.0);
        lerp(win.y + j as f32 * sh, icon.y + j as f32 * icon.h / n as f32, smoother(p))
    };
    let y = y_of(i);
    let y_next = y_of(i + 1);
    let row_h = (y_next - y).max(0.0);
    let y_mid = (y + y_next) * 0.5;
    let span = icon.y - win.y;
    let u = if span.abs() < 1.0 { 1.0 } else { ((y_mid - win.y) / span).clamp(0.0, 1.0) };
    let k = pinch * s_curve(u);
    let left = lerp(win.x, icon.x, k);
    let right = lerp(win.x + win.w, icon.x + icon.w, k);
    let width = (right - left).max(1.0);
    ((left, y), (width / win.w.max(1.0), ((row_h + 1.4) / sh.max(0.001)).max(0.001)))
}

pub fn tau_at(t: f32, restore: bool) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if restore {
        let r = 1.0 - (1.0 - t).powi(3);
        1.0 - smoother(r)
    } else {
        lerp(t, smooth(t), 0.65)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIN: Rect = Rect { x: 200.0, y: 100.0, w: 800.0, h: 600.0 };
    const ICON: Rect = Rect { x: 900.0, y: 1000.0, w: 48.0, h: 48.0 };

    #[test]
    fn starts_as_the_window_and_ends_in_the_icon() {
        let n = 60;
        for i in [0, 30, 59] {
            let ((x, y), (sx, _)) = slice(i, n, 0.0, WIN, ICON);
            assert!((x - WIN.x).abs() < 0.01 && (y - (WIN.y + i as f32 * 10.0)).abs() < 0.01 && (sx - 1.0).abs() < 1e-4);
            let ((x, y), (sx, _)) = slice(i, n, 1.0, WIN, ICON);
            assert!((x - ICON.x).abs() < 0.01, "irisan {i} x={x}");
            assert!(y >= ICON.y - 0.01 && y <= ICON.y + ICON.h, "irisan {i} y={y}");
            assert!((sx - ICON.w / WIN.w).abs() < 1e-3);
        }
    }

    #[test]
    fn bottom_slices_lead_and_narrow_first_forming_a_funnel() {
        let n = 60;
        let tau = 0.45;
        let w_top = slice(0, n, tau, WIN, ICON).1 .0;
        let w_mid = slice(30, n, tau, WIN, ICON).1 .0;
        let w_bot = slice(59, n, tau, WIN, ICON).1 .0;
        assert!(w_top > w_mid && w_mid > w_bot, "corong: atas {w_top} > tengah {w_mid} > bawah {w_bot}");
        assert!(progress(59, n, tau) > progress(0, n, tau));
    }

    #[test]
    fn neighbouring_slices_change_smoothly() {
        let n = 80;
        for k in 0..=20 {
            let tau = k as f32 / 20.0;
            for i in 1..n {
                let a = slice(i - 1, n, tau, WIN, ICON).1 .0;
                let b = slice(i, n, tau, WIN, ICON).1 .0;
                assert!((a - b).abs() < 0.08, "lompatan lebar di tau={tau} irisan {i}: {a} → {b}");
            }
        }
    }

    #[test]
    fn slices_stay_joined_without_gaps() {
        let n = 60;
        for k in 0..=24 {
            let tau = k as f32 / 24.0;
            for i in 0..n - 1 {
                let ((_, y), (_, sy)) = slice(i, n, tau, WIN, ICON);
                let ((_, y2), _) = slice(i + 1, n, tau, WIN, ICON);
                let bottom = y + sy * (WIN.h / n as f32);
                assert!(bottom >= y2 - 0.01, "celah di tau={tau} irisan {i}: bawah {bottom} < atas berikutnya {y2}");
            }
        }
    }

    #[test]
    fn timing_curves_cover_the_full_range() {
        assert_eq!(tau_at(0.0, false), 0.0);
        assert!((tau_at(1.0, false) - 1.0).abs() < 1e-6);
        assert!((tau_at(0.0, true) - 1.0).abs() < 1e-6);
        assert!(tau_at(1.0, true).abs() < 1e-6);
        assert!(tau_at(0.35, true) < 0.5);
    }
}
