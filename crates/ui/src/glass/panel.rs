use super::effect_graph::{backdrop_blur_brush, liquid_bevel_brush, liquid_core_brush};
use super::liquid;
use std::cell::Cell;
use windows::UI::Color;
use windows::UI::Composition::{
    CompositionBrush, CompositionClip, CompositionColorBrush, CompositionLinearGradientBrush, CompositionRoundedRectangleGeometry,
    CompositionSpriteShape, Compositor, ContainerVisual, ShapeVisual, SpriteVisual,
};
use windows_core::{Interface, Result};
use windows_numerics::{Vector2, Vector3};

const RIM_BOTTOM: f32 = 0.8;
const RIM_MID: f32 = 0.42;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Full,
    Light,
    Solid,
}

#[derive(Clone, Copy)]
pub struct GlassParams {
    pub size: (f32, f32),
    pub radius: f32,
    pub blur_sigma: f32,
    pub saturation: f32,
    pub tint: Color,
    pub rim: Color,
    pub solid: Color,
    pub highlight: f32,
    pub liquify: bool,
}

impl GlassParams {
    pub fn light(size: (f32, f32), radius: f32) -> Self {
        GlassParams {
            size,
            radius,
            blur_sigma: 24.0,
            saturation: 1.6,
            tint: Color { A: 46, R: 255, G: 255, B: 255 },
            rim: Color { A: 140, R: 255, G: 255, B: 255 },
            solid: Color { A: 215, R: 226, G: 228, B: 236 },
            highlight: 1.0,
            liquify: false,
        }
    }

    pub fn dark(size: (f32, f32), radius: f32) -> Self {
        GlassParams {
            tint: Color { A: 150, R: 20, G: 20, B: 24 },
            rim: Color { A: 70, R: 255, G: 255, B: 255 },
            solid: Color { A: 235, R: 28, G: 28, B: 32 },
            ..Self::light(size, radius)
        }
    }
}

pub struct GlassPanel {
    comp: Compositor,
    p: Cell<GlassParams>,
    tier: Cell<Tier>,
    glass: SpriteVisual,
    tint: SpriteVisual,
    tint_brush: CompositionColorBrush,
    refract: SpriteVisual,
    spec: SpriteVisual,
    spec_brush: CompositionLinearGradientBrush,
    caustic: SpriteVisual,
    rim_brush: CompositionLinearGradientBrush,
    rim_shape: CompositionSpriteShape,
    rim_wh: Cell<(f32, f32)>,
    fringe_off: Cell<bool>,
    rim: ShapeVisual,
    clip_geo: CompositionRoundedRectangleGeometry,
    rim_geo: CompositionRoundedRectangleGeometry,
    refract_key: Cell<(i32, i32, i32, u32, bool)>,
}

impl GlassPanel {
    pub fn new(comp: &Compositor, parent: &ContainerVisual, p: GlassParams) -> Result<GlassPanel> {
        let clip_geo = comp.CreateRoundedRectangleGeometry()?;
        clip_geo.SetCornerRadius(Vector2 { X: p.radius, Y: p.radius })?;
        let clip = comp.CreateGeometricClip()?;
        clip.SetGeometry(&clip_geo)?;
        let clip: CompositionClip = clip.cast()?;

        let glass = comp.CreateSpriteVisual()?;
        glass.SetClip(&clip)?;

        let tint = comp.CreateSpriteVisual()?;
        let tint_brush = comp.CreateColorBrushWithColor(p.tint)?;
        tint.SetBrush(&tint_brush)?;
        tint.SetClip(&clip)?;

        let refract = comp.CreateSpriteVisual()?;
        refract.SetClip(&clip)?;
        refract.SetOpacity(0.0)?;

        let spec = comp.CreateSpriteVisual()?;
        let spec_brush = comp.CreateLinearGradientBrush()?;
        spec.SetBrush(&spec_brush)?;
        spec.SetClip(&clip)?;

        let caustic = comp.CreateSpriteVisual()?;
        caustic.SetClip(&clip)?;
        caustic.SetOpacity(0.0)?;

        let rim_geo = comp.CreateRoundedRectangleGeometry()?;
        rim_geo.SetOffset(Vector2 { X: 0.5, Y: 0.5 })?;
        rim_geo.SetCornerRadius(Vector2 { X: (p.radius - 0.5).max(0.0), Y: (p.radius - 0.5).max(0.0) })?;
        let rim_shape = comp.CreateSpriteShapeWithGeometry(&rim_geo)?;
        let rim_brush = comp.CreateLinearGradientBrush()?;
        rim_shape.SetStrokeBrush(&rim_brush)?;
        rim_shape.SetStrokeThickness(1.5)?;
        let rim = comp.CreateShapeVisual()?;
        rim.Shapes()?.Append(&rim_shape)?;

        let kids = parent.Children()?;
        kids.InsertAtTop(&glass)?;
        kids.InsertAtTop(&tint)?;
        kids.InsertAtTop(&refract)?;
        kids.InsertAtTop(&spec)?;
        kids.InsertAtTop(&caustic)?;
        kids.InsertAtTop(&rim)?;

        let panel = GlassPanel {
            comp: comp.clone(),
            p: Cell::new(p),
            tier: Cell::new(Tier::Solid),
            glass,
            tint,
            tint_brush,
            refract,
            spec,
            spec_brush,
            caustic,
            rim_brush,
            rim_shape,
            rim_wh: Cell::new(p.size),
            fringe_off: Cell::new(false),
            rim,
            clip_geo,
            rim_geo,
            refract_key: Cell::new((0, 0, -1, 0, false)),
        };
        panel.paint_spec()?;
        panel.paint_rim(p.size.0, p.size.1)?;
        panel.set_tier(Tier::Full)?;
        panel.set_rect(0.0, 0.0, p.size.0, p.size.1)?;
        Ok(panel)
    }

    pub fn tier(&self) -> Tier {
        self.tier.get()
    }

    pub fn set_tier(&self, tier: Tier) -> Result<()> {
        if tier == self.tier.get() && self.glass.Brush().is_ok() {
            return Ok(());
        }
        self.refract_key.set((0, 0, -1, 0, false));
        self.apply_tier(tier)
    }

    pub fn set_params(&self, p: GlassParams) -> Result<()> {
        let mut cur = self.p.get();
        cur.blur_sigma = p.blur_sigma;
        cur.saturation = p.saturation;
        cur.tint = p.tint;
        cur.rim = p.rim;
        cur.solid = p.solid;
        cur.highlight = p.highlight;
        cur.liquify = p.liquify;
        self.p.set(cur);
        self.tint_brush.SetColor(p.tint)?;
        self.refract_key.set((0, 0, -1, 0, false));
        self.paint_spec()?;
        let (w, h) = self.rim_wh.get();
        self.paint_rim(w, h)?;
        self.apply_tier(self.tier.get())
    }

    fn paint_spec(&self) -> Result<()> {
        let liquify = self.p.get().liquify;
        let stops = self.spec_brush.ColorStops()?;
        stops.Clear()?;
        if liquify {
            self.spec_brush.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 })?;
            self.spec_brush.SetEndPoint(Vector2 { X: 0.65, Y: 1.0 })?;
            stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(0.0, Color { A: 58, R: 255, G: 255, B: 255 })?)?;
            stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(0.28, Color { A: 14, R: 255, G: 255, B: 255 })?)?;
            stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(0.62, Color { A: 0, R: 255, G: 255, B: 255 })?)?;
            stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(1.0, Color { A: 22, R: 235, G: 245, B: 255 })?)?;
        } else {
            self.spec_brush.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 })?;
            self.spec_brush.SetEndPoint(Vector2 { X: 0.0, Y: 1.0 })?;
            stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(0.0, Color { A: 70, R: 255, G: 255, B: 255 })?)?;
            stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(0.34, Color { A: 0, R: 255, G: 255, B: 255 })?)?;
        }
        Ok(())
    }

    fn paint_rim(&self, w: f32, h: f32) -> Result<()> {
        let p = self.p.get();
        self.rim_wh.set((w, h));
        let stops = self.rim_brush.ColorStops()?;
        stops.Clear()?;
        let with = |k: f32| Color { A: (p.rim.A as f32 * k).round().clamp(0.0, 255.0) as u8, ..p.rim };
        self.rim_brush.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 })?;
        self.rim_brush.SetEndPoint(Vector2 { X: 1.0, Y: 1.0 })?;
        stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(0.0, p.rim)?)?;
        stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(0.5, with(RIM_MID))?)?;
        stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(1.0, with(RIM_BOTTOM))?)?;
        self.rim_shape.SetStrokeThickness(1.5)?;
        Ok(())
    }

    fn sync_liquid(&self, w: f32, h: f32) -> Result<()> {
        let p = self.p.get();
        let tier = self.tier.get();
        let active = p.liquify && tier == Tier::Full && !self.fringe_off.get() && w >= 8.0 && h >= 8.0 && p.radius >= 2.0;
        if !active {
            self.refract.SetOpacity(0.0)?;
            self.caustic.SetOpacity(0.0)?;
            self.rim.SetOpacity(1.0)?;
            return Ok(());
        }
        let qw = ((w / 24.0).round() as i32).max(1) * 24;
        let qh = ((h / 24.0).round() as i32).max(1) * 24;
        let sigma_k = (p.blur_sigma * 10.0).round() as i32;
        let eff_r = (p.radius.min(w * 0.5).min(h * 0.5).round() as u32).clamp(4, 48);
        let dark = (p.tint.R as u32 + p.tint.G as u32 + p.tint.B as u32) < 384;
        let key = (qw, qh, sigma_k, eff_r, dark);
        if key != self.refract_key.get() {
            if let Ok(core) = liquid_core_brush(&self.comp, p.blur_sigma, p.saturation, w, h) {
                let brush: CompositionBrush = core.cast()?;
                self.glass.SetBrush(&brush)?;
            }
            if let (Ok(bevel), Ok((mask_ng, caustic_ng))) = (
                liquid_bevel_brush(&self.comp, p.blur_sigma, w, h),
                liquid::brushes(&self.comp, p.radius, w, h, dark),
            ) {
                let mb = self.comp.CreateMaskBrush()?;
                mb.SetSource(&bevel.cast::<CompositionBrush>()?)?;
                mb.SetMask(&mask_ng.cast::<CompositionBrush>()?)?;
                self.refract.SetBrush(&mb.cast::<CompositionBrush>()?)?;
                self.caustic.SetBrush(&caustic_ng.cast::<CompositionBrush>()?)?;
                self.refract_key.set(key);
            }
        }
        self.refract.SetOpacity(0.95)?;
        self.caustic.SetOpacity(1.0)?;
        self.rim.SetOpacity(0.0)?;
        Ok(())
    }

    fn apply_tier(&self, tier: Tier) -> Result<()> {
        let p = self.p.get();
        let (w, h) = self.rim_wh.get();
        let effect = match tier {
            Tier::Full if p.liquify => Some(
                liquid_core_brush(&self.comp, p.blur_sigma, p.saturation, w, h)
                    .or_else(|_| backdrop_blur_brush(&self.comp, p.blur_sigma, p.saturation)),
            ),
            Tier::Full => Some(backdrop_blur_brush(&self.comp, p.blur_sigma, p.saturation)),
            Tier::Light => Some(backdrop_blur_brush(&self.comp, p.blur_sigma * 0.4, p.saturation * 0.8)),
            Tier::Solid => None,
        };
        let (brush, tier): (CompositionBrush, Tier) = match effect {
            Some(Ok(b)) => (b.cast()?, tier),
            _ => (self.comp.CreateColorBrushWithColor(p.solid)?.cast()?, Tier::Solid),
        };
        self.glass.SetBrush(&brush)?;
        self.tint.SetOpacity(if tier == Tier::Solid { 0.0 } else { 1.0 })?;
        self.spec.SetOpacity(p.highlight * if tier == Tier::Solid { 0.5 } else { 1.0 })?;
        self.tier.set(tier);
        self.sync_liquid(w, h)?;
        Ok(())
    }

    pub fn animate_rect(&self, x: f32, y: f32, w: f32, h: f32, radius: f32, ratio: f32, period_ms: u32) -> Result<()> {
        use windows::Foundation::{PropertyValue, TimeSpan};
        use windows::UI::Composition::CompositionObject;
        use windows_core::HSTRING;
        let comp = &self.comp;
        let go = |obj: &CompositionObject, prop: &str, v: f32| -> Result<()> {
            let a = comp.CreateSpringScalarAnimation()?;
            a.SetDampingRatio(ratio)?;
            a.SetPeriod(TimeSpan { Duration: period_ms as i64 * 10_000 })?;
            a.SetFinalValue(&PropertyValue::CreateSingle(v)?.cast::<windows::Foundation::IReference<f32>>()?)?;
            obj.StartAnimation(&HSTRING::from(prop), &a)
        };
        for v in [&self.glass, &self.tint, &self.refract, &self.spec, &self.caustic] {
            let o: CompositionObject = v.cast()?;
            go(&o, "Size.X", w)?;
            go(&o, "Size.Y", h)?;
            go(&o, "Offset.X", x)?;
            go(&o, "Offset.Y", y)?;
        }
        let o: CompositionObject = self.rim.cast()?;
        go(&o, "Size.X", w)?;
        go(&o, "Size.Y", h)?;
        go(&o, "Offset.X", x)?;
        go(&o, "Offset.Y", y)?;
        let g: CompositionObject = self.clip_geo.cast()?;
        go(&g, "Size.X", w)?;
        go(&g, "Size.Y", h)?;
        go(&g, "CornerRadius.X", radius)?;
        go(&g, "CornerRadius.Y", radius)?;
        let r: CompositionObject = self.rim_geo.cast()?;
        go(&r, "Size.X", w - 1.0)?;
        go(&r, "Size.Y", h - 1.0)?;
        go(&r, "CornerRadius.X", (radius - 0.5).max(0.0))?;
        go(&r, "CornerRadius.Y", (radius - 0.5).max(0.0))?;
        let mut p = self.p.get();
        p.radius = radius;
        self.p.set(p);
        self.paint_rim(w, h)?;
        self.sync_liquid(w, h)
    }

    pub fn bind_x_width(
        &self,
        make: &dyn Fn(&str) -> Result<windows::UI::Composition::ExpressionAnimation>,
        x_expr: &str,
        w_expr: &str,
    ) -> Result<()> {
        use windows::UI::Composition::CompositionObject;
        use windows_core::HSTRING;
        let (x, w) = (make(x_expr)?, make(w_expr)?);
        let rim_w = make(&format!("({w_expr}) - 1"))?;
        for v in [&self.glass, &self.tint, &self.refract, &self.spec, &self.caustic] {
            let o: CompositionObject = v.cast()?;
            o.StartAnimation(&HSTRING::from("Offset.X"), &x)?;
            o.StartAnimation(&HSTRING::from("Size.X"), &w)?;
        }
        let o: CompositionObject = self.rim.cast()?;
        o.StartAnimation(&HSTRING::from("Offset.X"), &x)?;
        o.StartAnimation(&HSTRING::from("Size.X"), &w)?;
        let g: CompositionObject = self.clip_geo.cast()?;
        g.StartAnimation(&HSTRING::from("Size.X"), &w)?;
        let r: CompositionObject = self.rim_geo.cast()?;
        r.StartAnimation(&HSTRING::from("Size.X"), &rim_w)?;
        Ok(())
    }

    pub fn set_fringe(&self, on: bool) -> Result<()> {
        self.fringe_off.set(!on);
        let (w, h) = self.rim_wh.get();
        self.sync_liquid(w, h)
    }

    pub fn set_radius(&self, r: f32) -> Result<()> {
        self.clip_geo.SetCornerRadius(Vector2 { X: r, Y: r })?;
        self.rim_geo.SetCornerRadius(Vector2 { X: (r - 0.5).max(0.0), Y: (r - 0.5).max(0.0) })?;
        let mut p = self.p.get();
        if (p.radius - r).abs() > 0.5 {
            p.radius = r;
            self.p.set(p);
            let (w, h) = self.rim_wh.get();
            self.paint_rim(w, h)?;
            self.sync_liquid(w, h)?;
        }
        Ok(())
    }

    pub fn set_rect(&self, x: f32, y: f32, w: f32, h: f32) -> Result<()> {
        let size = Vector2 { X: w, Y: h };
        let at = Vector3 { X: x, Y: y, Z: 0.0 };
        for v in [&self.glass, &self.tint, &self.refract, &self.spec, &self.caustic] {
            v.SetSize(size)?;
            v.SetOffset(at)?;
        }
        self.rim.SetSize(size)?;
        self.rim.SetOffset(at)?;
        self.clip_geo.SetSize(size)?;
        self.rim_geo.SetSize(Vector2 { X: (w - 1.0).max(0.0), Y: (h - 1.0).max(0.0) })?;
        let (ow, oh) = self.rim_wh.get();
        if (h - oh).abs() > 0.5 || (w - ow).abs() > 0.5 {
            self.rim_wh.set((w, h));
            if self.p.get().liquify {
                self.sync_liquid(w, h)?;
            }
        }
        Ok(())
    }
}

pub fn add_panel(comp: &Compositor, parent: &ContainerVisual, p: GlassParams) -> Result<()> {
    GlassPanel::new(comp, parent, p)?;
    Ok(())
}
