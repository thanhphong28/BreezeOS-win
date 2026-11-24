pub const UP: u32 = 0xAF;
pub const DOWN: u32 = 0xAE;
pub const MUTE: u32 = 0xAD;
pub const STEP: f32 = 0.02;
pub const CHAIN_MS: u128 = 700;

pub fn is_volume_key(vk: u32) -> bool {
    matches!(vk, UP | DOWN | MUTE)
}

pub fn step(cur: f32, up: bool) -> f32 {
    let pos = cur.clamp(0.0, 1.0) / STEP;
    let next = if up { (pos + 1e-3).floor() as i32 + 1 } else { (pos - 1e-3).ceil() as i32 - 1 };
    (next as f32 * STEP).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_follow_the_two_percent_grid_and_stay_in_range() {
        assert!((step(0.5, true) - 0.52).abs() < 1e-4);
        assert!((step(0.5, false) - 0.48).abs() < 1e-4);
        assert!((step(0.51, true) - 0.52).abs() < 1e-4, "dari angka ganjil naik ke kelipatan 2% berikutnya");
        assert!((step(0.51, false) - 0.50).abs() < 1e-4, "dari angka ganjil turun ke kelipatan 2% sebelumnya");
        assert_eq!(step(1.0, true), 1.0);
        assert_eq!(step(0.0, false), 0.0);
        assert!((step(0.99, true) - 1.0).abs() < 1e-4);
        let mut v = 0.3;
        for _ in 0..10 {
            v = step(v, true);
        }
        assert!((v - 0.5).abs() < 1e-4, "sepuluh tekanan = 20%");
    }

    #[test]
    fn only_volume_keys_are_taken() {
        assert!(is_volume_key(UP) && is_volume_key(DOWN) && is_volume_key(MUTE));
        assert!(!is_volume_key(0xB3) && !is_volume_key(0x41));
    }
}
