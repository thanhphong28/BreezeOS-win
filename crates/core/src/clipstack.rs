#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pin {
    Text(String),
    Image { id: u64, w: u32, h: u32 },
}

pub const MAX_PINS: usize = 50;

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

pub fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(o) => {
                out.push('\\');
                out.push(o);
            }
            None => out.push('\\'),
        }
    }
    out
}

pub fn parse_pins(text: &str) -> Vec<Pin> {
    let mut out: Vec<Pin> = Vec::new();
    for line in text.trim_start_matches('\u{feff}').lines() {
        let mut it = line.splitn(2, '\t');
        let pin = match (it.next(), it.next()) {
            (Some("T"), Some(body)) if !body.is_empty() => Pin::Text(unescape(body)),
            (Some("I"), Some(body)) => {
                let v: Vec<&str> = body.split('\t').collect();
                let [id, w, h] = v[..] else { continue };
                match (id.parse(), w.parse(), h.parse()) {
                    (Ok(id), Ok(w), Ok(h)) if w > 0 && h > 0 => Pin::Image { id, w, h },
                    _ => continue,
                }
            }
            _ => continue,
        };
        if !out.contains(&pin) {
            out.push(pin);
        }
        if out.len() >= MAX_PINS {
            break;
        }
    }
    out
}

pub fn pins_text(pins: &[Pin]) -> String {
    pins.iter()
        .take(MAX_PINS)
        .map(|p| match p {
            Pin::Text(t) => format!("T\t{}\n", escape(t)),
            Pin::Image { id, w, h } => format!("I\t{id}\t{w}\t{h}\n"),
        })
        .collect()
}

pub fn parse_color(s: &str) -> Option<[u8; 3]> {
    let t = s.trim();
    if t.len() > 40 {
        return None;
    }
    if let Some(hex) = t.strip_prefix('#') {
        let v: Vec<u8> = hex.chars().map(|c| c.to_digit(16).map(|d| d as u8)).collect::<Option<_>>()?;
        return match v.len() {
            3 => Some([v[0] * 17, v[1] * 17, v[2] * 17]),
            6 => Some([v[0] * 16 + v[1], v[2] * 16 + v[3], v[4] * 16 + v[5]]),
            _ => None,
        };
    }
    let lower = t.to_ascii_lowercase();
    let inner = lower.strip_prefix("rgb(").or_else(|| lower.strip_prefix("rgba("))?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
    if parts.len() < 3 || parts.len() > 4 {
        return None;
    }
    let mut out = [0u8; 3];
    for (i, p) in parts.iter().take(3).enumerate() {
        out[i] = p.parse::<u16>().ok().filter(|n| *n <= 255)? as u8;
    }
    Some(out)
}

fn u32_at(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?))
}

fn i32_at(b: &[u8], i: usize) -> Option<i32> {
    Some(i32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?))
}

fn u16_at(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?))
}

pub const MAX_SIDE: u32 = 8192;

pub fn dib_to_bgra(dib: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let header = u32_at(dib, 0)? as usize;
    let w = i32_at(dib, 4)?;
    let h = i32_at(dib, 8)?;
    let bits = u16_at(dib, 14)?;
    let compression = u32_at(dib, 16)?;
    if header < 40 || w <= 0 || h == 0 || !(bits == 24 || bits == 32) || !(compression == 0 || compression == 3) {
        return None;
    }
    let (w, top_down, h) = (w as u32, h < 0, h.unsigned_abs());
    if w > MAX_SIDE || h > MAX_SIDE {
        return None;
    }
    let colors = u32_at(dib, 32).unwrap_or(0) as usize;
    let masks = if compression == 3 && header == 40 { 12 } else { 0 };
    let offset = header + masks + colors * 4;
    let bpp = bits as usize / 8;
    let stride = (w as usize * bpp).div_ceil(4) * 4;
    let need = offset + stride * h as usize;
    if dib.len() < need {
        return None;
    }
    let mut out = vec![0u8; w as usize * h as usize * 4];
    let mut any_alpha = false;
    for y in 0..h as usize {
        let src_row = if top_down { y } else { h as usize - 1 - y };
        let src = &dib[offset + src_row * stride..];
        for x in 0..w as usize {
            let s = &src[x * bpp..x * bpp + bpp];
            let o = (y * w as usize + x) * 4;
            out[o] = s[0];
            out[o + 1] = s[1];
            out[o + 2] = s[2];
            out[o + 3] = if bpp == 4 { s[3] } else { 255 };
            any_alpha |= bpp == 4 && s[3] != 0;
        }
    }
    if !any_alpha {
        for px in out.chunks_exact_mut(4) {
            px[3] = 255;
        }
    }
    Some((w, h, out))
}

pub fn thumbnail(bgra: &[u8], w: u32, h: u32, max: u32) -> (u32, u32, Vec<u8>) {
    let k = (max as f32 / w.max(h) as f32).min(1.0);
    let (tw, th) = (((w as f32 * k).round() as u32).max(1), ((h as f32 * k).round() as u32).max(1));
    let mut out = vec![0u8; (tw * th * 4) as usize];
    for ty in 0..th {
        let (y0, y1) = (ty * h / th, ((ty + 1) * h / th).max(ty * h / th + 1).min(h));
        for tx in 0..tw {
            let (x0, x1) = (tx * w / tw, ((tx + 1) * w / tw).max(tx * w / tw + 1).min(w));
            let mut acc = [0u32; 4];
            let mut n = 0u32;
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = ((y * w + x) * 4) as usize;
                    for c in 0..4 {
                        acc[c] += bgra[i + c] as u32;
                    }
                    n += 1;
                }
            }
            let o = ((ty * tw + tx) * 4) as usize;
            for c in 0..4 {
                out[o + c] = (acc[c] / n.max(1)) as u8;
            }
        }
    }
    (tw, th, out)
}

pub fn fingerprint(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let step = (bytes.len() / 4096).max(1);
    for b in bytes.iter().step_by(step) {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h ^ bytes.len() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escaping_round_trips() {
        for s in ["plain", "two\nlines", "tab\there", "back\\slash\\n", "\r\n", "trailing\\"] {
            assert_eq!(unescape(&escape(s)), s);
        }
        assert!(!escape("a\nb").contains('\n'));
    }

    #[test]
    fn pins_round_trip_dedupe_and_skip_junk() {
        let pins = vec![Pin::Text("line one\nline two".into()), Pin::Image { id: 42, w: 1920, h: 1080 }];
        let mut text = pins_text(&pins);
        text.push_str("T\tline one\\nline two\nI\tbad\nX\tnope\nI\t1\t0\t5\nT\t\n");
        assert_eq!(parse_pins(&text), pins);
    }

    #[test]
    fn colors_are_recognised() {
        assert_eq!(parse_color("#7F77DD"), Some([127, 119, 221]));
        assert_eq!(parse_color(" #fff "), Some([255, 255, 255]));
        assert_eq!(parse_color("rgb(10, 20, 30)"), Some([10, 20, 30]));
        assert_eq!(parse_color("RGBA(1,2,3,0.5)"), Some([1, 2, 3]));
        assert_eq!(parse_color("#12345"), None);
        assert_eq!(parse_color("#gggggg"), None);
        assert_eq!(parse_color("rgb(300,0,0)"), None);
        assert_eq!(parse_color("hello"), None);
    }

    fn dib(w: i32, h: i32, bits: u16, rows: &[&[u8]]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&40u32.to_le_bytes());
        b.extend_from_slice(&w.to_le_bytes());
        b.extend_from_slice(&h.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&bits.to_le_bytes());
        b.extend_from_slice(&[0u8; 24]);
        for r in rows {
            b.extend_from_slice(r);
        }
        b
    }

    #[test]
    fn bottom_up_24_bit_dib_becomes_top_down_bgra() {
        let d = dib(1, 2, 24, &[&[1, 2, 3, 0], &[4, 5, 6, 0]]);
        let (w, h, px) = dib_to_bgra(&d).unwrap();
        assert_eq!((w, h), (1, 2));
        assert_eq!(px, vec![4, 5, 6, 255, 1, 2, 3, 255]);
    }

    #[test]
    fn top_down_32_bit_without_alpha_is_made_opaque() {
        let d = dib(2, -1, 32, &[&[1, 2, 3, 0, 4, 5, 6, 0]]);
        let (_, _, px) = dib_to_bgra(&d).unwrap();
        assert_eq!(px, vec![1, 2, 3, 255, 4, 5, 6, 255]);
    }

    #[test]
    fn truncated_or_odd_dibs_are_rejected() {
        assert!(dib_to_bgra(&dib(4, 4, 24, &[&[0; 4]])).is_none());
        assert!(dib_to_bgra(&dib(1, 1, 8, &[&[0; 4]])).is_none());
        assert!(dib_to_bgra(&[1, 2, 3]).is_none());
    }

    #[test]
    fn thumbnails_keep_aspect_and_average() {
        let px = vec![255u8; 4 * 4 * 2];
        let (w, h, out) = thumbnail(&px, 4, 2, 2);
        assert_eq!((w, h), (2, 1));
        assert!(out.iter().all(|v| *v == 255));
        let (w, h, _) = thumbnail(&px, 4, 2, 100);
        assert_eq!((w, h), (4, 2));
    }

    #[test]
    fn fingerprint_changes_with_content() {
        assert_ne!(fingerprint(&[1, 2, 3]), fingerprint(&[1, 2, 4]));
        assert_eq!(fingerprint(&[9; 10]), fingerprint(&[9; 10]));
    }
}
