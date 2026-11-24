use crate::paths;
use nacre_platform::icons::Bitmap;
use nacre_platform::log;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

static GEN: AtomicU32 = AtomicU32::new(0);
const STEM: &str = "menu_logo";
const EXTS: [&str; 9] = ["png", "jpg", "jpeg", "bmp", "ico", "gif", "webp", "tif", "tiff"];

fn file_for(ext: &str) -> PathBuf {
    paths::config_dir().join(format!("{STEM}.{ext}"))
}

pub fn path() -> Option<PathBuf> {
    EXTS.iter().map(|e| file_for(e)).find(|p| p.exists())
}

pub fn generation() -> u32 {
    GEN.load(Ordering::Relaxed)
}

fn clear() {
    for e in EXTS {
        let _ = std::fs::remove_file(file_for(e));
    }
}

pub fn set(src: &str) -> bool {
    let ext = Path::new(src).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !EXTS.contains(&ext.as_str()) || nacre_platform::wic::decode_file_fit(src, 32).is_none() {
        log::line("menulogo", "berkas bukan gambar yang bisa dibaca");
        return false;
    }
    clear();
    let _ = std::fs::create_dir_all(paths::config_dir());
    if let Err(e) = std::fs::copy(src, file_for(&ext)) {
        log::line("menulogo", &format!("salin gagal: {e}"));
        return false;
    }
    GEN.fetch_add(1, Ordering::Relaxed);
    true
}

pub fn reset() {
    clear();
    GEN.fetch_add(1, Ordering::Relaxed);
}

pub fn bitmap(px: u32, rgb: [u8; 3]) -> Option<Bitmap> {
    let p = path()?;
    let b = nacre_platform::wic::decode_file_fit(&p.to_string_lossy(), (px * 4).max(64))?;
    Some(fit(trim(template(b, rgb)), (px as f32 * 1.3) as u32, px))
}

pub fn trim(b: Bitmap) -> Bitmap {
    let (w, h) = (b.w as usize, b.h as usize);
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
    for y in 0..h {
        for x in 0..w {
            if b.bgra[(y * w + x) * 4 + 3] > 8 {
                x0 = x0.min(x);
                x1 = x1.max(x);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
    }
    if x0 > x1 {
        return b;
    }
    let (cw, ch) = (x1 - x0 + 1, y1 - y0 + 1);
    let mut out = vec![0u8; cw * ch * 4];
    for y in 0..ch {
        let s = ((y0 + y) * w + x0) * 4;
        out[y * cw * 4..(y + 1) * cw * 4].copy_from_slice(&b.bgra[s..s + cw * 4]);
    }
    Bitmap { w: cw as u32, h: ch as u32, bgra: out }
}

pub fn fit(b: Bitmap, max_w: u32, max_h: u32) -> Bitmap {
    let k = (max_w as f32 / b.w.max(1) as f32).min(max_h as f32 / b.h.max(1) as f32).min(1.0);
    let (ow, oh) = (((b.w as f32 * k).round() as u32).max(1), ((b.h as f32 * k).round() as u32).max(1));
    if (ow, oh) == (b.w, b.h) {
        return b;
    }
    let mut out = vec![0u8; (ow * oh * 4) as usize];
    let (sx, sy) = (b.w as f32 / ow as f32, b.h as f32 / oh as f32);
    for y in 0..oh {
        let (ya, yb) = ((y as f32 * sy) as u32, (((y + 1) as f32 * sy).ceil() as u32).min(b.h).max((y as f32 * sy) as u32 + 1));
        for x in 0..ow {
            let (xa, xb) = ((x as f32 * sx) as u32, (((x + 1) as f32 * sx).ceil() as u32).min(b.w).max((x as f32 * sx) as u32 + 1));
            let mut acc = [0u32; 4];
            let mut n = 0u32;
            for yy in ya..yb {
                for xx in xa..xb {
                    let i = ((yy * b.w + xx) * 4) as usize;
                    for c in 0..4 {
                        acc[c] += b.bgra[i + c] as u32;
                    }
                    n += 1;
                }
            }
            let o = ((y * ow + x) * 4) as usize;
            for c in 0..4 {
                out[o + c] = (acc[c] / n.max(1)) as u8;
            }
        }
    }
    Bitmap { w: ow, h: oh, bgra: out }
}

fn unmul(px: &[u8]) -> (u32, u32, u32, u32) {
    let a = px[3] as u32;
    if a == 0 {
        return (0, 0, 0, 0);
    }
    (px[2] as u32 * 255 / a, px[1] as u32 * 255 / a, px[0] as u32 * 255 / a, a)
}

fn lum(r: u32, g: u32, b: u32) -> i32 {
    ((r * 54 + g * 183 + b * 19) / 256) as i32
}

pub fn template(b: Bitmap, rgb: [u8; 3]) -> Bitmap {
    let mut seen = 0u32;
    let mut gray = 0u32;
    let mut opaque = true;
    for p in b.bgra.chunks_exact(4) {
        let (r, g, bl, a) = unmul(p);
        if a < 250 {
            opaque = false;
        }
        if a > 16 {
            seen += 1;
            let hi = r.max(g).max(bl);
            let lo = r.min(g).min(bl);
            if hi - lo <= 24 {
                gray += 1;
            }
        }
    }
    if seen == 0 || gray * 100 < seen * 97 {
        return b;
    }
    let alpha: Vec<u32> = if opaque {
        let bg = {
            let (r, g, bl, _) = unmul(&b.bgra[0..4]);
            lum(r, g, bl)
        };
        let diffs: Vec<i32> = b.bgra.chunks_exact(4).map(|p| {
            let (r, g, bl, _) = unmul(p);
            (lum(r, g, bl) - bg).abs()
        }).collect();
        let peak = diffs.iter().copied().max().unwrap_or(0).max(1);
        diffs.iter().map(|d| (*d as u32 * 255 / peak as u32).min(255)).collect()
    } else {
        b.bgra.chunks_exact(4).map(|p| p[3] as u32).collect()
    };
    let mut out = vec![0u8; b.bgra.len()];
    for (i, a) in alpha.iter().enumerate() {
        let o = i * 4;
        out[o] = (rgb[2] as u32 * a / 255) as u8;
        out[o + 1] = (rgb[1] as u32 * a / 255) as u8;
        out[o + 2] = (rgb[0] as u32 * a / 255) as u8;
        out[o + 3] = *a as u8;
    }
    Bitmap { w: b.w, h: b.h, bgra: out }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bmp(px: &[[u8; 4]]) -> Bitmap {
        Bitmap { w: px.len() as u32, h: 1, bgra: px.iter().flatten().copied().collect() }
    }

    #[test]
    fn black_logo_on_transparent_takes_bar_color() {
        let out = template(bmp(&[[0, 0, 0, 255], [0, 0, 0, 0], [0, 0, 0, 128]]), [255, 255, 255]);
        assert_eq!(out.bgra, vec![255, 255, 255, 255, 0, 0, 0, 0, 128, 128, 128, 128]);
    }

    #[test]
    fn colorful_logo_is_left_alone() {
        let src = bmp(&[[0, 0, 255, 255], [0, 255, 0, 255], [255, 0, 0, 255]]);
        assert_eq!(template(src.clone(), [255, 255, 255]).bgra, src.bgra);
    }

    #[test]
    fn padding_is_trimmed_and_logo_fits_its_box() {
        let mut px = vec![[0u8; 4]; 100];
        for y in 3..7 {
            for x in 4..6 {
                px[y * 10 + x] = [255, 255, 255, 255];
            }
        }
        let b = Bitmap { w: 10, h: 10, bgra: px.iter().flatten().copied().collect() };
        let t = trim(b);
        assert_eq!((t.w, t.h), (2, 4));
        let f = fit(t, 10, 2);
        assert_eq!((f.w, f.h), (1, 2));
        assert_eq!(&f.bgra[0..4], &[255, 255, 255, 255]);
    }

    #[test]
    fn opaque_black_on_white_becomes_a_mask() {
        let out = template(bmp(&[[255, 255, 255, 255], [0, 0, 0, 255]]), [10, 20, 30]);
        assert_eq!(&out.bgra[0..4], &[0, 0, 0, 0]);
        assert_eq!(&out.bgra[4..8], &[30, 20, 10, 255]);
    }
}
