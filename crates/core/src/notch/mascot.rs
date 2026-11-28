pub const INTERVALS_MIN: [u8; 5] = [1, 3, 5, 10, 15];
pub const DEFAULT_INTERVAL_MIN: u8 = 3;
pub const INITIAL_GREET_MS: u32 = 7_500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Curious,
    Happy,
    Wink,
    Sleepy,
    Peekaboo,
    Excited,
}

impl Mood {
    pub const ALL: [Mood; 6] = [
        Mood::Curious,
        Mood::Happy,
        Mood::Wink,
        Mood::Sleepy,
        Mood::Peekaboo,
        Mood::Excited,
    ];
}

pub struct Routine {
    pub duration_ms: u32,
    pub offset_x: &'static [(f32, f32)],
    pub offset_y: &'static [(f32, f32)],
    pub tilt_deg: &'static [(f32, f32)],
    pub left_scale: &'static [(f32, f32, f32)],
    pub right_scale: &'static [(f32, f32, f32)],
    pub opacity: &'static [(f32, f32)],
    pub glow: &'static [(f32, f32)],
    pub blush: &'static [(f32, f32)],
}

pub fn snap_interval(mins: u8) -> u8 {
    let mut best = DEFAULT_INTERVAL_MIN;
    let mut best_diff = u8::MAX;
    for candidate in INTERVALS_MIN {
        let diff = candidate.abs_diff(mins);
        if diff < best_diff {
            best = candidate;
            best_diff = diff;
        }
    }
    best
}

pub fn interval_index(mins: u8) -> usize {
    let snapped = snap_interval(mins);
    INTERVALS_MIN.iter().position(|&m| m == snapped).unwrap_or(1)
}

pub fn next_delay_ms(mins: u8, seed: u64) -> u32 {
    let base_ms = snap_interval(mins) as u64 * 60_000;
    let mix = mix64(seed);
    let permille = 920 + (mix % 161);
    ((base_ms * permille) / 1_000).clamp(15_000, 3_600_000) as u32
}

pub fn pick_mood(seed: u64, last: Option<Mood>) -> Mood {
    let mix = mix64(seed);
    let mut idx = (mix as usize) % Mood::ALL.len();
    if Some(Mood::ALL[idx]) == last {
        idx = (idx + 1 + ((mix >> 16) as usize % (Mood::ALL.len() - 1))) % Mood::ALL.len();
    }
    Mood::ALL[idx]
}

fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

pub fn routine(mood: Mood, reduce_motion: bool) -> Routine {
    if reduce_motion {
        return CALM_ROUTINE;
    }
    match mood {
        Mood::Curious => CURIOUS,
        Mood::Happy => HAPPY,
        Mood::Wink => WINK,
        Mood::Sleepy => SLEEPY,
        Mood::Peekaboo => PEEKABOO,
        Mood::Excited => EXCITED,
    }
}

const ZERO_1D: &[(f32, f32)] = &[(0.0, 0.0), (1.0, 0.0)];

const CALM_ROUTINE: Routine = Routine {
    duration_ms: 4_200,
    offset_x: ZERO_1D,
    offset_y: &[(0.0, -3.0), (0.14, 0.0), (0.86, 0.0), (1.0, -3.0)],
    tilt_deg: ZERO_1D,
    left_scale: &[
        (0.0, 0.92, 0.85),
        (0.14, 1.0, 1.0),
        (0.46, 1.0, 1.0),
        (0.50, 1.08, 0.10),
        (0.55, 1.0, 1.0),
        (0.86, 1.0, 1.0),
        (1.0, 0.92, 0.85),
    ],
    right_scale: &[
        (0.0, 0.92, 0.85),
        (0.14, 1.0, 1.0),
        (0.46, 1.0, 1.0),
        (0.50, 1.08, 0.10),
        (0.55, 1.0, 1.0),
        (0.86, 1.0, 1.0),
        (1.0, 0.92, 0.85),
    ],
    opacity: &[(0.0, 0.0), (0.14, 1.0), (0.86, 1.0), (1.0, 0.0)],
    glow: &[(0.0, 0.0), (0.16, 0.88), (0.84, 0.88), (1.0, 0.0)],
    blush: ZERO_1D,
};

const CURIOUS: Routine = Routine {
    duration_ms: 5_200,
    offset_x: &[
        (0.0, 0.0),
        (0.24, 0.0),
        (0.36, -11.5),
        (0.46, -11.5),
        (0.60, 11.5),
        (0.70, 11.5),
        (0.80, 0.0),
        (1.0, 0.0),
    ],
    offset_y: &[
        (0.0, -10.5),
        (0.09, 1.4),
        (0.14, 0.0),
        (0.36, 0.8),
        (0.60, -0.6),
        (0.80, 0.0),
        (0.88, 0.5),
        (1.0, -10.0),
    ],
    tilt_deg: &[
        (0.0, 0.0),
        (0.24, 0.0),
        (0.36, -5.5),
        (0.46, -4.5),
        (0.60, 5.5),
        (0.70, 4.5),
        (0.79, -3.2),
        (0.86, 0.0),
        (1.0, 0.0),
    ],
    left_scale: &[
        (0.0, 0.65, 0.35),
        (0.09, 1.06, 1.10),
        (0.14, 1.0, 1.0),
        (0.18, 1.0, 1.0),
        (0.205, 1.12, 0.08),
        (0.24, 1.0, 1.03),
        (0.36, 1.06, 1.08),
        (0.46, 1.06, 1.08),
        (0.60, 0.93, 0.94),
        (0.63, 1.06, 0.09),
        (0.665, 0.93, 0.95),
        (0.80, 1.0, 1.0),
        (0.88, 1.0, 1.0),
        (1.0, 0.70, 0.35),
    ],
    right_scale: &[
        (0.0, 0.65, 0.35),
        (0.09, 1.06, 1.10),
        (0.14, 1.0, 1.0),
        (0.18, 1.0, 1.0),
        (0.205, 1.12, 0.08),
        (0.24, 1.0, 1.03),
        (0.36, 0.93, 0.94),
        (0.46, 0.93, 0.94),
        (0.60, 1.06, 1.08),
        (0.63, 1.12, 0.09),
        (0.665, 1.06, 1.08),
        (0.80, 1.0, 1.0),
        (0.88, 1.0, 1.0),
        (1.0, 0.70, 0.35),
    ],
    opacity: &[(0.0, 0.0), (0.08, 1.0), (0.90, 1.0), (1.0, 0.0)],
    glow: &[(0.0, 0.0), (0.12, 0.95), (0.50, 1.0), (0.88, 0.92), (1.0, 0.0)],
    blush: ZERO_1D,
};

const HAPPY: Routine = Routine {
    duration_ms: 4_800,
    offset_x: &[
        (0.0, 0.0),
        (0.26, 0.0),
        (0.36, -2.2),
        (0.48, 2.2),
        (0.60, -1.8),
        (0.72, 0.0),
        (1.0, 0.0),
    ],
    offset_y: &[
        (0.0, -10.5),
        (0.09, 1.6),
        (0.14, 0.0),
        (0.26, 0.5),
        (0.34, -3.4),
        (0.42, 0.6),
        (0.50, -3.1),
        (0.58, 0.6),
        (0.66, -2.4),
        (0.74, 0.0),
        (0.88, 0.0),
        (1.0, -10.0),
    ],
    tilt_deg: &[
        (0.0, 0.0),
        (0.26, 0.0),
        (0.36, -3.5),
        (0.48, 3.5),
        (0.60, -2.8),
        (0.72, 0.0),
        (1.0, 0.0),
    ],
    left_scale: &[
        (0.0, 0.65, 0.35),
        (0.09, 1.08, 1.12),
        (0.14, 1.0, 1.0),
        (0.17, 1.0, 1.0),
        (0.195, 1.12, 0.08),
        (0.23, 1.0, 1.0),
        (0.30, 1.24, 0.33),
        (0.68, 1.24, 0.33),
        (0.75, 1.04, 1.06),
        (0.80, 1.12, 0.09),
        (0.84, 1.0, 1.0),
        (0.89, 1.0, 1.0),
        (1.0, 0.68, 0.35),
    ],
    right_scale: &[
        (0.0, 0.65, 0.35),
        (0.09, 1.08, 1.12),
        (0.14, 1.0, 1.0),
        (0.17, 1.0, 1.0),
        (0.195, 1.12, 0.08),
        (0.23, 1.0, 1.0),
        (0.30, 1.24, 0.33),
        (0.68, 1.24, 0.33),
        (0.75, 1.04, 1.06),
        (0.80, 1.12, 0.09),
        (0.84, 1.0, 1.0),
        (0.89, 1.0, 1.0),
        (1.0, 0.68, 0.35),
    ],
    opacity: &[(0.0, 0.0), (0.08, 1.0), (0.90, 1.0), (1.0, 0.0)],
    glow: &[(0.0, 0.0), (0.12, 0.92), (0.48, 1.0), (0.88, 0.90), (1.0, 0.0)],
    blush: &[(0.0, 0.0), (0.24, 0.0), (0.33, 0.85), (0.68, 0.85), (0.80, 0.0), (1.0, 0.0)],
};

const WINK: Routine = Routine {
    duration_ms: 4_600,
    offset_x: &[
        (0.0, 0.0),
        (0.30, 0.0),
        (0.40, 2.8),
        (0.56, 2.8),
        (0.68, 0.0),
        (1.0, 0.0),
    ],
    offset_y: &[
        (0.0, -10.5),
        (0.10, 1.4),
        (0.15, 0.0),
        (0.28, 1.0),
        (0.40, -1.2),
        (0.56, -1.0),
        (0.65, -2.8),
        (0.74, 0.0),
        (0.88, 0.0),
        (1.0, -10.0),
    ],
    tilt_deg: &[
        (0.0, 0.0),
        (0.30, 0.0),
        (0.40, 7.2),
        (0.55, 7.2),
        (0.68, 0.0),
        (1.0, 0.0),
    ],
    left_scale: &[
        (0.0, 0.65, 0.35),
        (0.10, 1.06, 1.10),
        (0.15, 1.0, 1.0),
        (0.30, 1.04, 1.04),
        (0.40, 1.08, 1.10),
        (0.55, 1.08, 1.10),
        (0.66, 1.0, 1.0),
        (0.76, 1.0, 1.0),
        (0.79, 1.10, 0.09),
        (0.83, 1.0, 1.0),
        (0.89, 1.0, 1.0),
        (1.0, 0.68, 0.35),
    ],
    right_scale: &[
        (0.0, 0.65, 0.35),
        (0.10, 1.06, 1.10),
        (0.15, 1.0, 1.0),
        (0.34, 1.04, 1.04),
        (0.40, 1.24, 0.11),
        (0.53, 1.24, 0.11),
        (0.60, 1.08, 1.12),
        (0.66, 1.0, 1.0),
        (0.76, 1.0, 1.0),
        (0.79, 1.10, 0.09),
        (0.83, 1.0, 1.0),
        (0.89, 1.0, 1.0),
        (1.0, 0.68, 0.35),
    ],
    opacity: &[(0.0, 0.0), (0.08, 1.0), (0.90, 1.0), (1.0, 0.0)],
    glow: &[(0.0, 0.0), (0.12, 0.94), (0.48, 1.0), (0.88, 0.92), (1.0, 0.0)],
    blush: &[(0.0, 0.0), (0.34, 0.0), (0.42, 0.72), (0.58, 0.72), (0.70, 0.0), (1.0, 0.0)],
};

const SLEEPY: Routine = Routine {
    duration_ms: 5_600,
    offset_x: &[
        (0.0, 0.0),
        (0.35, 0.0),
        (0.58, -1.8),
        (0.65, 0.0),
        (1.0, 0.0),
    ],
    offset_y: &[
        (0.0, -9.0),
        (0.14, 0.0),
        (0.34, 0.6),
        (0.58, 4.0),
        (0.64, -1.6),
        (0.70, 0.0),
        (0.86, 0.5),
        (1.0, -9.5),
    ],
    tilt_deg: &[
        (0.0, 0.0),
        (0.34, 0.0),
        (0.58, -4.0),
        (0.65, 1.5),
        (0.72, 0.0),
        (1.0, 0.0),
    ],
    left_scale: &[
        (0.0, 0.75, 0.40),
        (0.14, 1.0, 0.96),
        (0.24, 1.06, 0.22),
        (0.34, 1.02, 0.68),
        (0.58, 1.14, 0.13),
        (0.64, 1.10, 1.14),
        (0.70, 1.0, 1.0),
        (0.75, 1.10, 0.09),
        (0.785, 1.0, 1.0),
        (0.82, 1.10, 0.09),
        (0.855, 1.0, 0.95),
        (1.0, 0.75, 0.30),
    ],
    right_scale: &[
        (0.0, 0.75, 0.40),
        (0.14, 1.0, 0.96),
        (0.24, 1.06, 0.22),
        (0.34, 1.02, 0.68),
        (0.58, 1.14, 0.13),
        (0.64, 1.10, 1.14),
        (0.70, 1.0, 1.0),
        (0.75, 1.10, 0.09),
        (0.785, 1.0, 1.0),
        (0.82, 1.10, 0.09),
        (0.855, 1.0, 0.95),
        (1.0, 0.75, 0.30),
    ],
    opacity: &[(0.0, 0.0), (0.12, 1.0), (0.88, 1.0), (1.0, 0.0)],
    glow: &[(0.0, 0.0), (0.15, 0.85), (0.58, 0.68), (0.66, 0.98), (0.86, 0.85), (1.0, 0.0)],
    blush: ZERO_1D,
};

const PEEKABOO: Routine = Routine {
    duration_ms: 5_000,
    offset_x: &[
        (0.0, -18.0),
        (0.14, -18.0),
        (0.34, -18.0),
        (0.48, 18.0),
        (0.60, 18.0),
        (0.72, 0.0),
        (1.0, 0.0),
    ],
    offset_y: &[
        (0.0, -10.5),
        (0.12, 1.4),
        (0.18, 0.6),
        (0.34, 0.6),
        (0.41, -1.5),
        (0.48, 0.6),
        (0.60, 0.6),
        (0.66, -3.0),
        (0.73, 0.0),
        (0.88, 0.0),
        (1.0, -10.0),
    ],
    tilt_deg: &[
        (0.0, 6.0),
        (0.34, 6.0),
        (0.48, -6.0),
        (0.60, -6.0),
        (0.72, 0.0),
        (1.0, 0.0),
    ],
    left_scale: &[
        (0.0, 0.65, 0.35),
        (0.12, 0.95, 0.98),
        (0.22, 0.95, 0.98),
        (0.25, 1.08, 0.09),
        (0.285, 0.95, 0.98),
        (0.41, 1.22, 0.82),
        (0.48, 0.96, 1.0),
        (0.60, 0.96, 1.0),
        (0.66, 0.94, 1.10),
        (0.73, 1.04, 1.04),
        (0.79, 1.12, 0.09),
        (0.83, 1.0, 1.0),
        (0.89, 1.0, 1.0),
        (1.0, 0.68, 0.35),
    ],
    right_scale: &[
        (0.0, 0.65, 0.35),
        (0.12, 1.04, 1.05),
        (0.22, 1.04, 1.05),
        (0.25, 1.12, 0.09),
        (0.285, 1.04, 1.05),
        (0.41, 1.22, 0.82),
        (0.48, 1.05, 1.06),
        (0.60, 1.05, 1.06),
        (0.66, 0.94, 1.10),
        (0.73, 1.04, 1.04),
        (0.79, 1.12, 0.09),
        (0.83, 1.0, 1.0),
        (0.89, 1.0, 1.0),
        (1.0, 0.68, 0.35),
    ],
    opacity: &[(0.0, 0.0), (0.09, 1.0), (0.90, 1.0), (1.0, 0.0)],
    glow: &[(0.0, 0.0), (0.12, 0.92), (0.72, 1.0), (0.88, 0.92), (1.0, 0.0)],
    blush: &[(0.0, 0.0), (0.64, 0.0), (0.73, 0.68), (0.84, 0.68), (0.92, 0.0), (1.0, 0.0)],
};

const EXCITED: Routine = Routine {
    duration_ms: 4_700,
    offset_x: &[
        (0.0, 0.0),
        (0.14, 0.0),
        (0.24, -9.5),
        (0.36, 9.5),
        (0.45, 0.0),
        (1.0, 0.0),
    ],
    offset_y: &[
        (0.0, -10.5),
        (0.08, 1.8),
        (0.13, 0.0),
        (0.45, 0.8),
        (0.53, -4.0),
        (0.61, 0.9),
        (0.69, -3.8),
        (0.77, 0.0),
        (0.88, 0.0),
        (1.0, -10.0),
    ],
    tilt_deg: &[
        (0.0, 0.0),
        (0.14, 0.0),
        (0.24, -4.5),
        (0.36, 4.5),
        (0.45, 0.0),
        (1.0, 0.0),
    ],
    left_scale: &[
        (0.0, 0.55, 0.35),
        (0.08, 1.12, 1.15),
        (0.13, 1.0, 1.0),
        (0.45, 1.12, 0.86),
        (0.53, 0.92, 1.16),
        (0.61, 1.14, 0.84),
        (0.69, 0.92, 1.15),
        (0.77, 1.04, 1.04),
        (0.81, 1.12, 0.08),
        (0.85, 1.0, 1.0),
        (0.89, 1.0, 1.0),
        (1.0, 0.65, 0.35),
    ],
    right_scale: &[
        (0.0, 0.55, 0.35),
        (0.08, 1.12, 1.15),
        (0.13, 1.0, 1.0),
        (0.45, 1.12, 0.86),
        (0.53, 0.92, 1.16),
        (0.61, 1.14, 0.84),
        (0.69, 0.92, 1.15),
        (0.77, 1.04, 1.04),
        (0.81, 1.12, 0.08),
        (0.85, 1.0, 1.0),
        (0.89, 1.0, 1.0),
        (1.0, 0.65, 0.35),
    ],
    opacity: &[(0.0, 0.0), (0.07, 1.0), (0.90, 1.0), (1.0, 0.0)],
    glow: &[(0.0, 0.0), (0.10, 0.96), (0.55, 1.0), (0.88, 0.94), (1.0, 0.0)],
    blush: &[(0.0, 0.0), (0.44, 0.0), (0.53, 0.78), (0.76, 0.78), (0.88, 0.0), (1.0, 0.0)],
};

#[cfg(test)]
mod tests {
    use super::*;

    fn check_1d(name: &str, track: &[(f32, f32)]) {
        assert!(track.len() >= 2, "{name}: needs at least 2 keyframes");
        assert!((track[0].0 - 0.0).abs() < 1e-6, "{name}: must start at 0.0");
        assert!((track.last().unwrap().0 - 1.0).abs() < 1e-6, "{name}: must end at 1.0");
        for pair in track.windows(2) {
            assert!(pair[1].0 > pair[0].0, "{name}: non-increasing times {} -> {}", pair[0].0, pair[1].0);
        }
    }

    fn check_2d(name: &str, track: &[(f32, f32, f32)]) {
        assert!(track.len() >= 2, "{name}: needs at least 2 keyframes");
        assert!((track[0].0 - 0.0).abs() < 1e-6, "{name}: must start at 0.0");
        assert!((track.last().unwrap().0 - 1.0).abs() < 1e-6, "{name}: must end at 1.0");
        for pair in track.windows(2) {
            assert!(pair[1].0 > pair[0].0, "{name}: non-increasing times {} -> {}", pair[0].0, pair[1].0);
        }
        for &(_, sx, sy) in track {
            assert!(sx > 0.05 && sy > 0.05, "{name}: invalid scale ({sx}, {sy})");
        }
    }

    #[test]
    fn all_routines_have_valid_monotonic_keyframes_and_clean_fades() {
        for reduce in [false, true] {
            for m in Mood::ALL {
                let r = routine(m, reduce);
                assert!((3_500..=7_000).contains(&r.duration_ms));
                check_1d("offset_x", r.offset_x);
                check_1d("offset_y", r.offset_y);
                check_1d("tilt_deg", r.tilt_deg);
                check_2d("left_scale", r.left_scale);
                check_2d("right_scale", r.right_scale);
                check_1d("opacity", r.opacity);
                check_1d("glow", r.glow);
                check_1d("blush", r.blush);
                assert_eq!(r.opacity[0].1, 0.0);
                assert_eq!(r.opacity.last().unwrap().1, 0.0);
                assert_eq!(r.glow[0].1, 0.0);
                assert_eq!(r.glow.last().unwrap().1, 0.0);
                for &(_, v) in r.opacity.iter().chain(r.glow.iter()).chain(r.blush.iter()) {
                    assert!((0.0..=1.0).contains(&v));
                }
            }
        }
    }

    #[test]
    fn mood_picker_never_repeats_immediately_and_covers_all_moods() {
        let mut last = None;
        let mut seen = [false; Mood::ALL.len()];
        for seed in 1..=64 {
            let next = pick_mood(seed * 7919, last);
            assert_ne!(Some(next), last, "mood repeated back-to-back at seed {seed}");
            let idx = Mood::ALL.iter().position(|&m| m == next).unwrap();
            seen[idx] = true;
            last = Some(next);
        }
        assert!(seen.into_iter().all(|b| b), "all 6 moods should be reachable");
    }

    #[test]
    fn intervals_snap_and_apply_natural_jitter() {
        assert_eq!(snap_interval(0), 1);
        assert_eq!(snap_interval(1), 1);
        assert_eq!(snap_interval(2), 1);
        assert_eq!(snap_interval(3), 3);
        assert_eq!(snap_interval(4), 3);
        assert_eq!(snap_interval(5), 5);
        assert_eq!(snap_interval(8), 10);
        assert_eq!(snap_interval(99), 15);
        for (i, &m) in INTERVALS_MIN.iter().enumerate() {
            assert_eq!(interval_index(m), i);
            let ms = next_delay_ms(m, 42);
            let exact = m as u32 * 60_000;
            assert!(ms >= exact * 92 / 100 && ms <= exact * 108 / 100);
        }
    }
}
