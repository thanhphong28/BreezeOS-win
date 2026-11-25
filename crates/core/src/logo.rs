fn rr_dist(px: f32, py: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
    let (qx, qy) = ((px - cx).abs() - (hw - r), (py - cy).abs() - (hh - r));
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) - r
}

pub fn render_battery(w: u32, h: u32, pct: u8, rgb: [u8; 3], fill: [u8; 3]) -> Vec<u8> {
    let (wf, hf) = (w as f32, h as f32);
    let nub = (wf * 0.07).max(1.5);
    let bw = wf - nub - 1.0;
    let stroke = (hf * 0.09).max(1.0);
    let r = hf * 0.30;
    let gap = stroke + (hf * 0.10).max(1.0);
    let inner_w = (bw - 2.0 * gap) * (pct.min(100) as f32 / 100.0);
    let ss = 4;
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (mut a_line, mut a_fill) = (0f32, 0f32);
            for sy in 0..ss {
                for sx in 0..ss {
                    let (px, py) = (x as f32 + (sx as f32 + 0.5) / ss as f32, y as f32 + (sy as f32 + 0.5) / ss as f32);
                    let d = rr_dist(px, py, bw / 2.0, hf / 2.0, bw / 2.0, hf / 2.0, r);
                    if d <= 0.0 && d >= -stroke {
                        a_line += 1.0;
                    }
                    if rr_dist(px, py, bw + 1.0 + nub / 2.0 - nub * 0.5, hf / 2.0, nub, hf * 0.19, nub * 0.9) <= 0.0 && px > bw + 0.8 {
                        a_line += 1.0;
                    }
                    if inner_w > 0.5 && rr_dist(px, py, gap + inner_w / 2.0, hf / 2.0, inner_w / 2.0, hf / 2.0 - gap, (r - gap).max(0.8).min(inner_w / 2.0)) <= 0.0 {
                        a_fill += 1.0;
                    }
                }
            }
            let n = (ss * ss) as f32;
            let (al, af) = (a_line / n * 0.45, a_fill / n);
            let a = (af + al * (1.0 - af)).min(1.0);
            let i = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                let col = fill[2 - c] as f32 * af + rgb[2 - c] as f32 * al * (1.0 - af);
                out[i + c] = col.round().min(255.0) as u8;
            }
            out[i + 3] = (a * 255.0).round() as u8;
        }
    }
    out
}

pub fn render_battery_pill(w: u32, h: u32, pct: u8, rgb: [u8; 3], fill: [u8; 3], snug: bool) -> Vec<u8> {
    let (wf, hf) = (w as f32, h as f32);
    let nub = (wf * 0.06).max(1.5);
    let bw = wf - nub - 1.0;
    let stroke = (hf * 0.085).max(1.0);
    let r = hf * 0.38;
    let gap = stroke + if snug { (hf * 0.05).max(0.6) } else { (hf * 0.11).max(1.0) };
    let inner = bw - 2.0 * gap;
    let cut = gap + inner * (pct.min(100) as f32 / 100.0);
    let ir = (r - gap).max(0.8);
    let ss = 4;
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (mut a_line, mut a_on, mut a_off) = (0f32, 0f32, 0f32);
            for sy in 0..ss {
                for sx in 0..ss {
                    let (px, py) = (x as f32 + (sx as f32 + 0.5) / ss as f32, y as f32 + (sy as f32 + 0.5) / ss as f32);
                    let d = rr_dist(px, py, bw / 2.0, hf / 2.0, bw / 2.0, hf / 2.0, r);
                    if d <= 0.0 && d >= -stroke {
                        a_line += 1.0;
                    }
                    if px > bw + 0.8 && rr_dist(px, py, bw + 1.0, hf / 2.0, nub, hf * 0.2, nub * 0.9) <= 0.0 {
                        a_line += 1.0;
                    }
                    if rr_dist(px, py, bw / 2.0, hf / 2.0, inner / 2.0, hf / 2.0 - gap, ir) <= 0.0 {
                        if px <= cut {
                            a_on += 1.0;
                        } else {
                            a_off += 1.0;
                        }
                    }
                }
            }
            let n = (ss * ss) as f32;
            let (al, aon, aoff) = (a_line / n * 0.5, a_on / n, a_off / n * 0.3);
            let body = aon + aoff;
            let a = (body + al * (1.0 - body)).min(1.0);
            let i = ((y * w + x) * 4) as usize;
            for c in 0..3 {
                let col = fill[2 - c] as f32 * aon + rgb[2 - c] as f32 * aoff + rgb[2 - c] as f32 * al * (1.0 - body);
                out[i + c] = col.round().min(255.0) as u8;
            }
            out[i + 3] = (a * 255.0).round() as u8;
        }
    }
    out
}

pub fn render_wifi(n: u32, level: u8, rgb: [u8; 3], dim: f32) -> Vec<u8> {
    let nf = n as f32;
    let (cx, cy) = (nf / 2.0, nf * 0.86);
    let big = nf * 0.80;
    let bands = [(0.0f32, 0.30f32), (0.44, 0.64), (0.78, 1.0)];
    let half = 45f32.to_radians();
    let ss = 4;
    let mut out = vec![0u8; (n * n * 4) as usize];
    for y in 0..n {
        for x in 0..n {
            let mut acc = 0f32;
            for sy in 0..ss {
                for sx in 0..ss {
                    let (px, py) = (x as f32 + (sx as f32 + 0.5) / ss as f32 - cx, y as f32 + (sy as f32 + 0.5) / ss as f32 - cy);
                    let d = (px * px + py * py).sqrt() / big;
                    let ang = px.atan2(-py);
                    if ang.abs() > half || py > 0.0 {
                        continue;
                    }
                    for (k, (r0, r1)) in bands.iter().enumerate() {
                        if d >= *r0 && d <= *r1 {
                            acc += if (k as u8) < level { 1.0 } else { dim };
                        }
                    }
                }
            }
            let a = (acc / (ss * ss) as f32).min(1.0);
            let i = ((y * n + x) * 4) as usize;
            for c in 0..3 {
                out[i + c] = (rgb[2 - c] as f32 * a).round() as u8;
            }
            out[i + 3] = (a * 255.0).round() as u8;
        }
    }
    out
}

pub fn render_grip(w: u32, h: u32, center: (f32, f32), radius: f32, thick: f32, from_deg: f32, to_deg: f32) -> Vec<u8> {
    let (a0, a1) = (from_deg.to_radians(), to_deg.to_radians());
    let ends = [(center.0 + radius * a0.cos(), center.1 + radius * a0.sin()), (center.0 + radius * a1.cos(), center.1 + radius * a1.sin())];
    let half = thick / 2.0;
    let ss = 4;
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (mut fill, mut rim, mut shine) = (0f32, 0f32, 0f32);
            for sy in 0..ss {
                for sx in 0..ss {
                    let (px, py) = (x as f32 + (sx as f32 + 0.5) / ss as f32, y as f32 + (sy as f32 + 0.5) / ss as f32);
                    let (dx, dy) = (px - center.0, py - center.1);
                    let ang = dy.atan2(dx);
                    let d = if ang >= a0 && ang <= a1 {
                        ((dx * dx + dy * dy).sqrt() - radius).abs()
                    } else {
                        ends.iter().map(|e| ((px - e.0).powi(2) + (py - e.1).powi(2)).sqrt()).fold(f32::MAX, f32::min)
                    };
                    if d <= half {
                        fill += 1.0;
                        if d >= half - 1.4 {
                            rim += 1.0;
                        }
                        if (dx * dx + dy * dy).sqrt() < radius && d >= half - 3.0 {
                            shine += 1.0;
                        }
                    }
                }
            }
            let n = (ss * ss) as f32;
            let a = (fill / n * 0.42 + rim / n * 0.5 + shine / n * 0.18).min(1.0);
            let i = ((y * w + x) * 4) as usize;
            let v = (255.0 * a).round() as u8;
            out[i] = v;
            out[i + 1] = v;
            out[i + 2] = v;
            out[i + 3] = v;
        }
    }
    out
}

pub fn wifi_level(bars5: Option<u8>) -> u8 {
    match bars5 {
        None => 0,
        Some(0..=1) => 1,
        Some(2..=3) => 2,
        Some(_) => 3,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn wifi_level_dims_upper_bands() {
        use super::*;
        let n = 40u32;
        let a = |b: &[u8], x: u32, y: u32| b[((y * n + x) * 4 + 3) as usize];
        let full = render_wifi(n, 3, [255, 255, 255], 0.3);
        let weak = render_wifi(n, 1, [255, 255, 255], 0.3);
        let top = (20, (40.0 * 0.86 - 40.0 * 0.80 * 0.9) as u32);
        assert!(a(&full, top.0, top.1) > 200);
        assert!(a(&weak, top.0, top.1) < 100 && a(&weak, top.0, top.1) > 30);
        let dot = (20, (40.0 * 0.86 - 40.0 * 0.80 * 0.15) as u32);
        assert!(a(&weak, dot.0, dot.1) > 200);
        assert_eq!((wifi_level(None), wifi_level(Some(1)), wifi_level(Some(3)), wifi_level(Some(5))), (0, 1, 2, 3));
    }

    #[test]
    fn grip_follows_the_corner_arc() {
        use super::*;
        let (w, h) = (60u32, 60u32);
        let g = render_grip(w, h, (10.0, 10.0), 36.0, 14.0, 10.0, 80.0);
        let a = |x: u32, y: u32| g[((y * w + x) * 4 + 3) as usize];
        let mid = 45f32.to_radians();
        let (mx, my) = ((10.0 + 36.0 * mid.cos()) as u32, (10.0 + 36.0 * mid.sin()) as u32);
        assert!(a(mx, my) > 40, "pita di sudut terisi");
        assert_eq!(a(12, 12), 0, "pusat lengkung kosong");
        assert_eq!(a(58, 2), 0, "di luar busur kosong");
    }

    #[test]
    fn pill_battery_splits_fill_and_rest() {
        use super::*;
        let (w, h) = (60u32, 26u32);
        let b = render_battery_pill(w, h, 50, [255, 255, 255], [255, 255, 255], true);
        let a = |x: u32, y: u32| b[((y * w + x) * 4 + 3) as usize];
        assert!(a(12, 13) > 240, "bagian terisi solid");
        assert!(a(44, 13) < 120 && a(44, 13) > 40, "sisa transparan");
    }

    #[test]
    fn battery_outline_fill_and_nub() {
        use super::*;
        let (w, h) = (52u32, 24u32);
        let full = render_battery(w, h, 100, [255, 255, 255], [255, 255, 255]);
        let empty = render_battery(w, h, 0, [255, 255, 255], [255, 255, 255]);
        let a = |b: &[u8], x: u32, y: u32| b[((y * w + x) * 4 + 3) as usize];
        assert!(a(&full, 24, 12) > 240, "isian penuh di tengah");
        assert_eq!(a(&empty, 24, 12), 0, "kosong: tengah transparan");
        assert!(a(&empty, 24, 0) > 60 && a(&empty, 24, 0) < 160, "garis tepi semi-transparan");
        assert!(a(&empty, w - 2, 12) > 40, "tonjolan kanan");
        assert_eq!(a(&empty, w - 1, 0), 0);
    }
}
