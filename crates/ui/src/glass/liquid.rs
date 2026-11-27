use crate::gfx::Gfx;
use std::cell::RefCell;
use std::collections::HashMap;
use windows::UI::Composition::{
    CompositionBitmapInterpolationMode, CompositionDrawingSurface, CompositionNineGridBrush, CompositionStretch, Compositor,
};
use windows_core::Result;

struct CachedMaps {
    mask: CompositionDrawingSurface,
    caustic: CompositionDrawingSurface,
    inset: f32,
}

thread_local! {
    static MAPS: RefCell<HashMap<(u32, bool), CachedMaps>> = RefCell::new(HashMap::new());
}

fn sdf_and_normal(px: f32, py: f32, side: f32, rad: f32) -> (f32, f32, f32) {
    let c = side * 0.5;
    let h = c - rad;
    let (dx, dy) = (px - c, py - c);
    let (qx, qy) = (dx.abs() - h, dy.abs() - h);
    let mx = qx.max(0.0);
    let my = qy.max(0.0);
    let out_len = (mx * mx + my * my).sqrt();
    let d = out_len + qx.max(qy).min(0.0) - rad;
    let (nx, ny) = if out_len > 1e-4 {
        ((mx / out_len) * dx.signum(), (my / out_len) * dy.signum())
    } else if qx > qy {
        (dx.signum(), 0.0)
    } else {
        (0.0, dy.signum())
    };
    (d, nx, ny)
}

fn build_maps(r_px: u32, dark: bool) -> (u32, f32, Vec<u8>, Vec<u8>) {
    let r = r_px.clamp(4, 48);
    let mid = 16u32;
    let side = 2 * r + mid;
    let sf = side as f32;
    let rf = r as f32;
    // Keep bevel width strictly inside the Nine-Grid corner inset `r` so the stretched center patch is 0-alpha.
    let bw = (rf * 0.82).clamp(3.4, 18.5);
    let mut mask = vec![0u8; (side * side * 4) as usize];
    let mut caustic = vec![0u8; (side * side * 4) as usize];

    for y in 0..side {
        for x in 0..side {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let (d, nx, ny) = sdf_and_normal(px, py, sf, rf);
            let cov = (0.5 - d).clamp(0.0, 1.0);
            if cov <= 0.0 {
                continue;
            }
            let inward = (-d).max(0.0);
            let idx = ((y * side + x) * 4) as usize;

            // 1. Meniscus refraction mask (1.0 at curved perimeter, smoothly easing to 0.0 before `r`).
            if inward < bw {
                let u = (inward / bw).clamp(0.0, 1.0);
                let w = (1.0 - u * u * (3.0 - 2.0 * u)).powf(1.15) * cov;
                let a = (w * 255.0).round() as u8;
                mask[idx] = a;
                mask[idx + 1] = a;
                mask[idx + 2] = a;
                mask[idx + 3] = a;
            }

            // 2. 3D Liquid Glass normal-mapped specular lip + volumetric cushion + subtle prism caustics.
            if inward < bw {
                let key = (nx * -0.62 + ny * -0.78).max(0.0);
                let bounce = (nx * 0.65 + ny * 0.76).max(0.0);
                let prism_tr = (nx * 0.72 + ny * -0.69).max(0.0);
                let prism_bl = (nx * -0.72 + ny * 0.69).max(0.0);
                let prism = (prism_tr + prism_bl).clamp(0.0, 1.0);

                // Diamond-crisp outer rim specular (top-left primary + bottom-right internal reflection).
                let lip_t = (1.0 - (inward / 2.1).clamp(0.0, 1.0)).powi(2);
                let lip = lip_t * (0.32 + 0.68 * key.powf(1.2) + 0.56 * bounce.powf(1.35));

                // Soft 3D rounded glass thickness cushion right inside the lip.
                let cush_span = (bw * 0.60).max(2.5);
                let tc = ((inward - 0.7) / cush_span).clamp(0.0, 1.0);
                let cushion = (4.0 * tc * (1.0 - tc)).clamp(0.0, 1.0) * (0.07 + 0.24 * key + 0.15 * bounce);

                // Spectral dispersion kiss along curved meniscus (cyan outer, violet/magenta inner).
                let cyan_w = (4.0 * ((inward - 0.8) / (bw * 0.44).max(2.0)).clamp(0.0, 1.0)
                    * (1.0 - ((inward - 0.8) / (bw * 0.44).max(2.0)).clamp(0.0, 1.0)))
                    * prism.powf(1.25)
                    * if dark { 0.24 } else { 0.17 };
                let violet_w = (4.0 * ((inward - bw * 0.24) / (bw * 0.52).max(2.0)).clamp(0.0, 1.0)
                    * (1.0 - ((inward - bw * 0.24) / (bw * 0.52).max(2.0)).clamp(0.0, 1.0)))
                    * prism.powf(1.25)
                    * if dark { 0.20 } else { 0.14 };

                // Whisper-thin inner depth groove where the curved bevel meets the flat interior.
                let tg = ((inward - bw * 0.55) / (bw * 0.45).max(1.5)).clamp(0.0, 1.0);
                let groove = (4.0 * tg * (1.0 - tg)).clamp(0.0, 1.0) * if dark { 0.08 } else { 0.045 };

                let white_a = (lip + cushion).clamp(0.0, 1.0);
                let mut r_p = white_a + cyan_w * 0.34 + violet_w * 0.78;
                let mut g_p = white_a + cyan_w * 0.82 + violet_w * 0.40;
                let mut b_p = white_a + cyan_w * 1.00 + violet_w * 1.00;
                let mut a_p = (white_a + cyan_w + violet_w + groove).clamp(0.0, 1.0);

                // Premultiply coverage and clamp valid premultiplied RGBA.
                a_p = (a_p * cov).clamp(0.0, 1.0);
                r_p = (r_p * cov).clamp(0.0, a_p);
                g_p = (g_p * cov).clamp(0.0, a_p);
                b_p = (b_p * cov).clamp(0.0, a_p);

                caustic[idx] = (b_p * 255.0).round() as u8;
                caustic[idx + 1] = (g_p * 255.0).round() as u8;
                caustic[idx + 2] = (r_p * 255.0).round() as u8;
                caustic[idx + 3] = (a_p * 255.0).round() as u8;
            }
        }
    }
    (side, rf, mask, caustic)
}

fn nine_grid(comp: &Compositor, surface: &CompositionDrawingSurface, inset: f32) -> Result<CompositionNineGridBrush> {
    let sb = comp.CreateSurfaceBrushWithSurface(surface)?;
    sb.SetStretch(CompositionStretch::Fill)?;
    sb.SetBitmapInterpolationMode(CompositionBitmapInterpolationMode::Linear)?;
    let ng = comp.CreateNineGridBrush()?;
    ng.SetSource(&sb)?;
    ng.SetLeftInset(inset)?;
    ng.SetTopInset(inset)?;
    ng.SetRightInset(inset)?;
    ng.SetBottomInset(inset)?;
    Ok(ng)
}

pub fn brushes(comp: &Compositor, radius: f32, w: f32, h: f32, dark: bool) -> Result<(CompositionNineGridBrush, CompositionNineGridBrush)> {
    let eff_r = radius.min(w.max(8.0) * 0.5).min(h.max(8.0) * 0.5);
    let r_px = (eff_r.round() as u32).clamp(4, 48);
    let (mask_surf, caustic_surf, inset) = MAPS.with(|cell| -> Result<_> {
        let mut map = cell.borrow_mut();
        if Gfx::shared_lost() {
            map.clear();
        }
        if let Some(c) = map.get(&(r_px, dark)) {
            return Ok((c.mask.clone(), c.caustic.clone(), c.inset));
        }
        if map.len() >= 16 {
            map.clear();
        }
        let gfx = Gfx::shared(comp)?;
        let (side, inset, mask_px, caustic_px) = build_maps(r_px, dark);
        let mask = gfx.surface_from_bgra(side, side, &mask_px)?;
        let caustic = gfx.surface_from_bgra(side, side, &caustic_px)?;
        map.insert((r_px, dark), CachedMaps { mask: mask.clone(), caustic: caustic.clone(), inset });
        Ok((mask, caustic, inset))
    })?;
    Ok((nine_grid(comp, &mask_surf, inset)?, nine_grid(comp, &caustic_surf, inset)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_grid_center_patch_is_transparent_and_premultiplied() {
        for r_in in [4, 10, 22, 48] {
            for dark in [false, true] {
                let (side, inset, mask, caustic) = build_maps(r_in, dark);
                let r = inset as u32;
                assert_eq!(side, 2 * r + 16);
                // Entire 16x16 stretched center patch must be 0-alpha so Nine-Grid stretching never smears.
                for y in r..(r + 16) {
                    for x in r..(r + 16) {
                        let i = ((y * side + x) * 4) as usize;
                        assert_eq!(mask[i + 3], 0, "mask center non-zero at r={r_in} ({x},{y})");
                        assert_eq!(caustic[i + 3], 0, "caustic center non-zero at r={r_in} ({x},{y})");
                    }
                }
                // Every pixel must obey premultiplied alpha (B, G, R <= A).
                for px in caustic.chunks_exact(4) {
                    assert!(px[0] <= px[3] && px[1] <= px[3] && px[2] <= px[3]);
                }
            }
        }
    }
}
