use nacre_core::notch::mascot::{routine, Mood};
use windows::Foundation::TimeSpan;
use windows::UI::Color;
use windows::UI::Composition::{
    CompositionEasingFunction, CompositionObject, CompositionRoundedRectangleGeometry, Compositor,
    ContainerVisual, ShapeVisual, SpriteVisual,
};
use windows_core::{Interface, Result, HSTRING};
use windows_numerics::{Vector2, Vector3};

const EYE_GAP_HALF_DP: f32 = 11.0;
const EYE_W_DP: f32 = 8.6;
const EYE_H_DP: f32 = 12.4;
const HALO_W_DP: f32 = 30.0;
const HALO_H_DP: f32 = 34.0;
const INNER_GLOW_W_DP: f32 = 16.0;
const INNER_GLOW_H_DP: f32 = 20.0;
const BLUSH_X_DP: f32 = 18.5;
const BLUSH_Y_DP: f32 = 5.2;
const BLUSH_W_DP: f32 = 14.0;
const BLUSH_H_DP: f32 = 9.0;

struct EyeRig {
    root: ContainerVisual,
    halo: SpriteVisual,
    inner_glow: SpriteVisual,
    core_vis: ShapeVisual,
    core_geo: CompositionRoundedRectangleGeometry,
}

pub struct MascotLayer {
    comp: Compositor,
    glow_root: ContainerVisual,
    backdrop: SpriteVisual,
    orbs: [SpriteVisual; 3],
    rim: SpriteVisual,
    anchor: ContainerVisual,
    head: ContainerVisual,
    left: EyeRig,
    right: EyeRig,
    blush_root: ContainerVisual,
    blushes: [SpriteVisual; 2],
    active: bool,
}

fn dur(ms: u32) -> TimeSpan {
    TimeSpan { Duration: ms as i64 * 10_000 }
}

fn argb(a: u8, r: u8, g: u8, b: u8) -> Color {
    Color { A: a, R: r, G: g, B: b }
}

impl MascotLayer {
    pub fn new(comp: &Compositor, depth: &ContainerVisual, dpi: f32) -> Result<Self> {
        let glow_root = comp.CreateContainerVisual()?;
        glow_root.SetOpacity(0.0)?;
        depth.Children()?.InsertAtTop(&glow_root)?;

        // Subtle dark interior veil so glowing white eyes & Nusa aurora pop even on Crystal/Blur material.
        let backdrop = comp.CreateSpriteVisual()?;
        let dark_brush = comp.CreateLinearGradientBrush()?;
        dark_brush.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 })?;
        dark_brush.SetEndPoint(Vector2 { X: 0.0, Y: 1.0 })?;
        dark_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.0, argb(235, 8, 9, 14))?)?;
        dark_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(1.0, argb(215, 10, 14, 26))?)?;
        backdrop.SetBrush(&dark_brush)?;
        glow_root.Children()?.InsertAtTop(&backdrop)?;

        // Nusa-inspired tri-color ambient interior aurora: Cyan (left), Electric Blue (center), Vivid Violet (right).
        let orb_specs: [(Color, Color, Color); 3] = [
            (argb(215, 0, 214, 255), argb(96, 0, 182, 255), argb(0, 0, 150, 255)),
            (argb(225, 48, 112, 255), argb(108, 42, 96, 245), argb(0, 32, 78, 235)),
            (argb(215, 138, 68, 255), argb(96, 118, 52, 245), argb(0, 96, 38, 230)),
        ];
        let mut orbs_vec = Vec::with_capacity(3);
        for (c0, c1, c2) in orb_specs {
            let sp = comp.CreateSpriteVisual()?;
            let rb = comp.CreateRadialGradientBrush()?;
            rb.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.0, c0)?)?;
            rb.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.48, c1)?)?;
            rb.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(1.0, c2)?)?;
            sp.SetBrush(&rb)?;
            glow_root.Children()?.InsertAtTop(&sp)?;
            orbs_vec.push(sp);
        }
        let [o0, o1, o2]: [SpriteVisual; 3] = orbs_vec.try_into().ok().unwrap();

        // Delicate luminous bottom rim highlight clipped smoothly by depth_geo.
        let rim = comp.CreateSpriteVisual()?;
        let rim_brush = comp.CreateLinearGradientBrush()?;
        rim_brush.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 })?;
        rim_brush.SetEndPoint(Vector2 { X: 1.0, Y: 0.0 })?;
        rim_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.0, argb(95, 56, 224, 255))?)?;
        rim_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.5, argb(115, 110, 165, 255))?)?;
        rim_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(1.0, argb(95, 175, 115, 255))?)?;
        rim.SetBrush(&rim_brush)?;
        glow_root.Children()?.InsertAtTop(&rim)?;

        // Eye rig hierarchy: anchor (follows notch geometry) -> head (animated gaze/bounce/tilt/opacity).
        let anchor = comp.CreateContainerVisual()?;
        depth.Children()?.InsertAtTop(&anchor)?;
        let head = comp.CreateContainerVisual()?;
        head.SetOpacity(0.0)?;
        head.SetCenterPoint(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 })?;
        anchor.Children()?.InsertAtTop(&head)?;

        // Soft cheek blush sprites (shown during Happy / Wink / Peekaboo / Excited).
        let blush_root = comp.CreateContainerVisual()?;
        blush_root.SetOpacity(0.0)?;
        head.Children()?.InsertAtTop(&blush_root)?;
        let blush_brush = comp.CreateRadialGradientBrush()?;
        blush_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.0, argb(175, 255, 128, 196))?)?;
        blush_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.55, argb(72, 220, 110, 225))?)?;
        blush_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(1.0, argb(0, 180, 95, 255))?)?;
        let blush_l = comp.CreateSpriteVisual()?;
        let blush_r = comp.CreateSpriteVisual()?;
        blush_l.SetBrush(&blush_brush)?;
        blush_r.SetBrush(&blush_brush)?;
        blush_root.Children()?.InsertAtTop(&blush_l)?;
        blush_root.Children()?.InsertAtTop(&blush_r)?;

        // Shared halo & inner-bloom radial brushes for the two glowing eyes (zero extra GPU textures).
        let halo_brush = comp.CreateRadialGradientBrush()?;
        halo_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.0, argb(175, 185, 228, 255))?)?;
        halo_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.42, argb(78, 100, 168, 255))?)?;
        halo_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(1.0, argb(0, 65, 125, 255))?)?;

        let inner_brush = comp.CreateRadialGradientBrush()?;
        inner_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.0, argb(215, 245, 252, 255))?)?;
        inner_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(0.55, argb(92, 198, 235, 255))?)?;
        inner_brush.ColorStops()?.Append(&comp.CreateColorGradientStopWithOffsetAndColor(1.0, argb(0, 150, 210, 255))?)?;

        let white_brush = comp.CreateColorBrushWithColor(argb(255, 255, 255, 255))?;

        let make_eye = || -> Result<EyeRig> {
            let root = comp.CreateContainerVisual()?;
            root.SetCenterPoint(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 })?;

            let halo = comp.CreateSpriteVisual()?;
            halo.SetBrush(&halo_brush)?;
            root.Children()?.InsertAtTop(&halo)?;

            let inner_glow = comp.CreateSpriteVisual()?;
            inner_glow.SetBrush(&inner_brush)?;
            root.Children()?.InsertAtTop(&inner_glow)?;

            let core_geo = comp.CreateRoundedRectangleGeometry()?;
            let shape = comp.CreateSpriteShapeWithGeometry(&core_geo)?;
            shape.SetFillBrush(&white_brush)?;
            let core_vis = comp.CreateShapeVisual()?;
            core_vis.Shapes()?.Append(&shape)?;
            root.Children()?.InsertAtTop(&core_vis)?;

            head.Children()?.InsertAtTop(&root)?;
            Ok(EyeRig { root, halo, inner_glow, core_vis, core_geo })
        };

        let left = make_eye()?;
        let right = make_eye()?;

        let mut layer = Self {
            comp: comp.clone(),
            glow_root,
            backdrop,
            orbs: [o0, o1, o2],
            rim,
            anchor,
            head,
            left,
            right,
            blush_root,
            blushes: [blush_l, blush_r],
            active: false,
        };
        layer.set_dpi(dpi);
        Ok(layer)
    }

    pub fn set_dpi(&mut self, dpi: f32) {
        let ew = (EYE_W_DP * dpi).max(4.0);
        let eh = (EYE_H_DP * dpi).max(6.0);
        let pad = (2.0 * dpi).ceil();
        let sv_w = ew + 2.0 * pad;
        let sv_h = eh + 2.0 * pad;
        let hw = HALO_W_DP * dpi;
        let hh = HALO_H_DP * dpi;
        let iw = INNER_GLOW_W_DP * dpi;
        let ih = INNER_GLOW_H_DP * dpi;

        for (side, eye) in [(-1.0f32, &self.left), (1.0f32, &self.right)] {
            let _ = eye.root.SetOffset(Vector3 { X: side * EYE_GAP_HALF_DP * dpi, Y: 0.0, Z: 0.0 });
            let _ = eye.halo.SetSize(Vector2 { X: hw, Y: hh });
            let _ = eye.halo.SetOffset(Vector3 { X: -hw * 0.5, Y: -hh * 0.5, Z: 0.0 });
            let _ = eye.inner_glow.SetSize(Vector2 { X: iw, Y: ih });
            let _ = eye.inner_glow.SetOffset(Vector3 { X: -iw * 0.5, Y: -ih * 0.5, Z: 0.0 });
            let _ = eye.core_vis.SetSize(Vector2 { X: sv_w, Y: sv_h });
            let _ = eye.core_vis.SetOffset(Vector3 { X: -sv_w * 0.5, Y: -sv_h * 0.5, Z: 0.0 });
            let _ = eye.core_geo.SetOffset(Vector2 { X: pad, Y: pad });
            let _ = eye.core_geo.SetSize(Vector2 { X: ew, Y: eh });
            let _ = eye.core_geo.SetCornerRadius(Vector2 { X: ew * 0.5, Y: ew * 0.5 });
        }

        let bw = BLUSH_W_DP * dpi;
        let bh = BLUSH_H_DP * dpi;
        for (side, b) in [(-1.0f32, &self.blushes[0]), (1.0f32, &self.blushes[1])] {
            let _ = b.SetSize(Vector2 { X: bw, Y: bh });
            let _ = b.SetOffset(Vector3 {
                X: side * BLUSH_X_DP * dpi - bw * 0.5,
                Y: BLUSH_Y_DP * dpi - bh * 0.5,
                Z: 0.0,
            });
        }
    }

    pub fn layout(&self, x0: f32, w: f32, h: f32, over: f32, springy: Option<(f32, u32)>) {
        let _ = self.backdrop.SetOffset(Vector3 { X: x0, Y: -over, Z: 0.0 });
        let _ = self.backdrop.SetSize(Vector2 { X: w, Y: h + over });

        let orb_w = w * 0.92;
        let orb_h = (h * 1.65).max(46.0);
        let centers_x = [x0 + w * 0.16, x0 + w * 0.50, x0 + w * 0.84];
        let centers_y = [h * 0.98, h * 1.02, h * 0.98];
        for (i, orb) in self.orbs.iter().enumerate() {
            let _ = orb.SetSize(Vector2 { X: orb_w, Y: orb_h });
            let _ = orb.SetOffset(Vector3 {
                X: centers_x[i] - orb_w * 0.5,
                Y: centers_y[i] - orb_h * 0.5,
                Z: 0.0,
            });
        }

        let rim_h = (h * 0.06).clamp(1.5, 3.0);
        let _ = self.rim.SetSize(Vector2 { X: w, Y: rim_h });
        let _ = self.rim.SetOffset(Vector3 { X: x0, Y: h - rim_h, Z: 0.0 });

        let cx = x0 + w * 0.5;
        let cy = h * 0.52;
        match springy {
            Some((ratio, period)) => {
                if let Ok(obj) = self.anchor.cast::<CompositionObject>() {
                    super::spring(&self.comp, &obj, "Offset.X", cx, ratio, period);
                    super::spring(&self.comp, &obj, "Offset.Y", cy, ratio, period);
                }
            }
            None => {
                let _ = self.anchor.SetOffset(Vector3 { X: cx, Y: cy, Z: 0.0 });
            }
        }
    }

    pub fn play(&mut self, mood: Mood, dpi: f32, reduce_motion: bool) {
        self.active = true;
        let r = routine(mood, reduce_motion);
        let total = dur(r.duration_ms);
        let go = || -> Result<()> {
            let smooth: CompositionEasingFunction = self
                .comp
                .CreateCubicBezierEasingFunction(Vector2 { X: 0.28, Y: 0.0 }, Vector2 { X: 0.18, Y: 1.0 })?
                .cast()?;
            let snappy: CompositionEasingFunction = self
                .comp
                .CreateCubicBezierEasingFunction(Vector2 { X: 0.22, Y: 1.28 }, Vector2 { X: 0.32, Y: 1.0 })?
                .cast()?;

            let head_obj: CompositionObject = self.head.cast()?;
            let glow_obj: CompositionObject = self.glow_root.cast()?;
            let blush_obj: CompositionObject = self.blush_root.cast()?;
            let left_obj: CompositionObject = self.left.root.cast()?;
            let right_obj: CompositionObject = self.right.root.cast()?;

            let ax = self.comp.CreateScalarKeyFrameAnimation()?;
            for &(t, vx) in r.offset_x {
                if t == 0.0 {
                    ax.InsertKeyFrame(0.0, vx * dpi)?;
                } else {
                    ax.InsertKeyFrameWithEasingFunction(t, vx * dpi, &smooth)?;
                }
            }
            ax.SetDuration(total)?;

            let ay = self.comp.CreateScalarKeyFrameAnimation()?;
            for &(t, vy) in r.offset_y {
                if t == 0.0 {
                    ay.InsertKeyFrame(0.0, vy * dpi)?;
                } else {
                    ay.InsertKeyFrameWithEasingFunction(t, vy * dpi, &snappy)?;
                }
            }
            ay.SetDuration(total)?;

            let rot = self.comp.CreateScalarKeyFrameAnimation()?;
            for &(t, deg) in r.tilt_deg {
                if t == 0.0 {
                    rot.InsertKeyFrame(0.0, deg)?;
                } else {
                    rot.InsertKeyFrameWithEasingFunction(t, deg, &smooth)?;
                }
            }
            rot.SetDuration(total)?;

            let op = self.comp.CreateScalarKeyFrameAnimation()?;
            for &(t, v) in r.opacity {
                if t == 0.0 {
                    op.InsertKeyFrame(0.0, v)?;
                } else {
                    op.InsertKeyFrameWithEasingFunction(t, v, &smooth)?;
                }
            }
            op.SetDuration(total)?;

            let gl = self.comp.CreateScalarKeyFrameAnimation()?;
            for &(t, v) in r.glow {
                if t == 0.0 {
                    gl.InsertKeyFrame(0.0, v)?;
                } else {
                    gl.InsertKeyFrameWithEasingFunction(t, v, &smooth)?;
                }
            }
            gl.SetDuration(total)?;

            let bl = self.comp.CreateScalarKeyFrameAnimation()?;
            for &(t, v) in r.blush {
                if t == 0.0 {
                    bl.InsertKeyFrame(0.0, v)?;
                } else {
                    bl.InsertKeyFrameWithEasingFunction(t, v, &smooth)?;
                }
            }
            bl.SetDuration(total)?;

            let make_scale = |track: &[(f32, f32, f32)]| -> Result<_> {
                let sc = self.comp.CreateVector3KeyFrameAnimation()?;
                for &(t, sx, sy) in track {
                    let v = Vector3 { X: sx, Y: sy, Z: 1.0 };
                    if t == 0.0 {
                        sc.InsertKeyFrame(0.0, v)?;
                    } else {
                        sc.InsertKeyFrameWithEasingFunction(t, v, &smooth)?;
                    }
                }
                sc.SetDuration(total)?;
                Ok(sc)
            };
            let ls = make_scale(r.left_scale)?;
            let rs = make_scale(r.right_scale)?;

            head_obj.StartAnimation(&HSTRING::from("Offset.X"), &ax)?;
            head_obj.StartAnimation(&HSTRING::from("Offset.Y"), &ay)?;
            head_obj.StartAnimation(&HSTRING::from("RotationAngleInDegrees"), &rot)?;
            head_obj.StartAnimation(&HSTRING::from("Opacity"), &op)?;
            glow_obj.StartAnimation(&HSTRING::from("Opacity"), &gl)?;
            blush_obj.StartAnimation(&HSTRING::from("Opacity"), &bl)?;
            left_obj.StartAnimation(&HSTRING::from("Scale"), &ls)?;
            right_obj.StartAnimation(&HSTRING::from("Scale"), &rs)?;
            Ok(())
        };
        let _ = go();
    }

    pub fn stop(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        if let Ok(head_obj) = self.head.cast::<CompositionObject>() {
            let _ = head_obj.StopAnimation(&HSTRING::from("Offset.X"));
            let _ = head_obj.StopAnimation(&HSTRING::from("Offset.Y"));
            let _ = head_obj.StopAnimation(&HSTRING::from("RotationAngleInDegrees"));
            let _ = head_obj.StopAnimation(&HSTRING::from("Opacity"));
        }
        if let Ok(glow_obj) = self.glow_root.cast::<CompositionObject>() {
            let _ = glow_obj.StopAnimation(&HSTRING::from("Opacity"));
        }
        if let Ok(blush_obj) = self.blush_root.cast::<CompositionObject>() {
            let _ = blush_obj.StopAnimation(&HSTRING::from("Opacity"));
        }
        for eye in [&self.left, &self.right] {
            if let Ok(o) = eye.root.cast::<CompositionObject>() {
                let _ = o.StopAnimation(&HSTRING::from("Scale"));
            }
            let _ = eye.root.SetScale(Vector3 { X: 1.0, Y: 1.0, Z: 1.0 });
        }
        let _ = self.head.SetOffset(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 });
        let _ = self.head.SetRotationAngleInDegrees(0.0);
        let _ = self.head.SetOpacity(0.0);
        let _ = self.glow_root.SetOpacity(0.0);
        let _ = self.blush_root.SetOpacity(0.0);
    }
}
