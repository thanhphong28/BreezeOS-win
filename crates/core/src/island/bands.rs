pub const FFT: usize = 512;
pub const BANDS: usize = 5;
const RANGES: [(usize, usize); BANDS] = [(1, 2), (2, 6), (6, 18), (18, 60), (60, 200)];
const WEIGHT: [f32; BANDS] = [1.2, 1.2, 1.5, 2.8, 5.0];
pub const FLOOR: f32 = 0.18;
pub const SPEC: usize = 16;

pub fn spec_ranges() -> [(usize, usize); SPEC] {
    let (lo, hi) = (1.0f32, 200.0f32);
    let mut out = [(0usize, 0usize); SPEC];
    let mut prev = 1usize;
    for (i, slot) in out.iter_mut().enumerate() {
        let edge = (lo * (hi / lo).powf((i + 1) as f32 / SPEC as f32)).round() as usize;
        let end = edge.max(prev + 1);
        *slot = (prev, end);
        prev = end;
    }
    out
}

fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * std::f32::consts::PI / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..len / 2 {
                let (a, b) = (start + k, start + k + len / 2);
                let (tr, ti) = (re[b] * cr - im[b] * ci, re[b] * ci + im[b] * cr);
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let nr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = nr;
            }
        }
        len <<= 1;
    }
}

pub struct Analyzer {
    buf: Vec<f32>,
    fill: usize,
    peaks: [f32; BANDS],
    prev: [f32; BANDS],
    re: Vec<f32>,
    im: Vec<f32>,
    spec: [f32; SPEC],
    spec_peaks: [f32; SPEC],
    spec_ranges: [(usize, usize); SPEC],
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            buf: vec![0.0; FFT],
            fill: 0,
            peaks: [0.01; BANDS],
            prev: [0.1; BANDS],
            re: vec![0.0; FFT],
            im: vec![0.0; FFT],
            spec: [FLOOR; SPEC],
            spec_peaks: [0.01; SPEC],
            spec_ranges: spec_ranges(),
        }
    }

    pub fn spectrum(&self) -> [f32; SPEC] {
        self.spec
    }

    fn analyze_spectrum(&mut self) {
        let last = SPEC as f32 - 1.0;
        for i in 0..SPEC {
            let (lo, hi) = self.spec_ranges[i];
            let mut sum = 0.0;
            let mut n = 0u32;
            for b in lo..hi.min(FFT / 2) {
                sum += (self.re[b] * self.re[b] + self.im[b] * self.im[b]).sqrt();
                n += 1;
            }
            let scaled = sum / n.max(1) as f32 * (1.0 + 4.0 * i as f32 / last);
            if scaled > self.spec_peaks[i] {
                self.spec_peaks[i] = scaled;
            } else {
                self.spec_peaks[i] *= 0.99;
            }
            let target = (scaled / self.spec_peaks[i].max(0.12)).min(1.0).powf(0.75);
            let smooth = if target > self.spec[i] { 0.15 } else { 0.25 };
            self.spec[i] = (self.spec[i] * smooth + target * (1.0 - smooth)).clamp(FLOOR, 1.0);
        }
    }

    pub fn push(&mut self, mono: &[f32], mut emit: impl FnMut([f32; BANDS])) {
        for &s in mono {
            let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * self.fill as f32 / FFT as f32).cos());
            self.buf[self.fill] = s * w;
            self.fill += 1;
            if self.fill == FFT {
                self.fill = 0;
                emit(self.analyze());
            }
        }
    }

    fn analyze(&mut self) -> [f32; BANDS] {
        self.re.copy_from_slice(&self.buf);
        self.im.iter_mut().for_each(|v| *v = 0.0);
        fft(&mut self.re, &mut self.im);
        let mut out = [0.0; BANDS];
        for (i, &(lo, hi)) in RANGES.iter().enumerate() {
            let mut sum = 0.0;
            let mut n = 0u32;
            for b in lo..hi.min(FFT / 2) {
                sum += (self.re[b] * self.re[b] + self.im[b] * self.im[b]).sqrt();
                n += 1;
            }
            let scaled = sum / n.max(1) as f32 * WEIGHT[i];
            if scaled > self.peaks[i] {
                self.peaks[i] = scaled;
            } else {
                self.peaks[i] *= 0.99;
            }
            let target = (scaled / self.peaks[i].max(0.12)).min(1.0).powf(0.75);
            let smooth = if target > self.prev[i] { 0.10 } else { 0.20 };
            let v = (self.prev[i] * smooth + target * (1.0 - smooth)).clamp(FLOOR, 1.0);
            self.prev[i] = v;
            out[i] = v;
        }
        self.analyze_spectrum();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f32, rate: f32, n: usize) -> Vec<f32> {
        (0..n).map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / rate).sin() * 0.8).collect()
    }

    fn run(samples: &[f32]) -> [f32; BANDS] {
        let mut a = Analyzer::new();
        let mut last = [0.0; BANDS];
        a.push(samples, |b| last = b);
        last
    }

    #[test]
    fn silence_stays_at_the_floor() {
        let out = run(&vec![0.0; FFT * 8]);
        assert!(out.iter().all(|v| (*v - FLOOR).abs() < 1e-6), "{out:?}");
    }

    #[test]
    fn fft_places_a_pure_tone_in_the_right_bin() {
        let tone = sine(937.5, 48_000.0, FFT * 30);
        let out = run(&tone);
        let top = out.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        assert_eq!(top, 2, "{out:?}");
        assert!(out[2] > 0.9, "{out:?}");
    }

    #[test]
    fn low_tone_lands_in_the_low_band_and_not_in_the_highest() {
        let out = run(&sine(280.0, 48_000.0, FFT * 30));
        assert!(out[1] > out[4], "{out:?}");
        assert_eq!(out.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0, 1);
    }

    #[test]
    fn output_is_always_within_bounds() {
        let noisy: Vec<f32> = (0..FFT * 10).map(|i| (((i * 7919) % 2000) as f32 / 1000.0) - 1.0).collect();
        let mut a = Analyzer::new();
        a.push(&noisy, |b| assert!(b.iter().all(|v| (FLOOR..=1.0).contains(v)), "{b:?}"));
    }

    #[test]
    fn spectrum_ranges_are_ordered_and_cover_the_band() {
        let r = spec_ranges();
        assert_eq!(r[0].0, 1);
        assert!(r.iter().all(|(a, b)| b > a));
        assert!(r.windows(2).all(|w| w[0].1 == w[1].0));
        assert!(r[SPEC - 1].1 >= 200);
    }

    #[test]
    fn spectrum_puts_a_low_tone_left_of_a_high_tone() {
        let peak = |freq: f32| {
            let mut a = Analyzer::new();
            a.push(&sine(freq, 48_000.0, FFT * 30), |_| {});
            let s = a.spectrum();
            assert!(s.iter().all(|v| (FLOOR..=1.0).contains(v)), "{s:?}");
            s.iter().enumerate().max_by(|x, y| x.1.partial_cmp(y.1).unwrap()).unwrap().0
        };
        assert!(peak(300.0) < peak(6000.0));
    }

    #[test]
    fn emits_one_frame_per_512_samples() {
        let mut a = Analyzer::new();
        let mut n = 0;
        a.push(&vec![0.1; FFT * 3 + 100], |_| n += 1);
        assert_eq!(n, 3);
        a.push(&vec![0.1; FFT - 100], |_| n += 1);
        assert_eq!(n, 4, "sisa sampel dibawa ke panggilan berikutnya");
    }
}
