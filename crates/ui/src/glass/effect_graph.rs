use std::cell::RefCell;
use windows::Foundation::{IPropertyValue, PropertyValue};
use windows::Graphics::Effects::{IGraphicsEffect, IGraphicsEffectSource, IGraphicsEffectSource_Impl, IGraphicsEffect_Impl};
use windows::UI::Composition::{CompositionEffectBrush, CompositionEffectSourceParameter, Compositor};
use windows::Win32::Foundation::{E_INVALIDARG, E_NOTIMPL};
use windows::Win32::Graphics::Direct2D::{CLSID_D2D1GaussianBlur, CLSID_D2D1Saturation};
use windows::Win32::System::WinRT::Graphics::Direct2D::{
    IGraphicsEffectD2D1Interop, IGraphicsEffectD2D1Interop_Impl, GRAPHICS_EFFECT_PROPERTY_MAPPING,
};
use windows_core::{implement, Interface, Result, GUID, HSTRING, PCWSTR};

const CLSID_AFFINE_2D: GUID = GUID::from_u128(0x6aa97485_6354_4cfc_908c_e4a74f62c96c);
const CLSID_COLOR_MATRIX: GUID = GUID::from_u128(0x921f03d6_641c_47df_852d_b4bb6153ae11);
const CLSID_ARITHMETIC: GUID = GUID::from_u128(0xfc151437_049a_4784_a24a_f1c4daf20987);

#[implement(IGraphicsEffect, IGraphicsEffectSource, IGraphicsEffectD2D1Interop)]
struct D2dEffect {
    name: RefCell<HSTRING>,
    id: GUID,
    props: Vec<IPropertyValue>,
    sources: Vec<IGraphicsEffectSource>,
}

impl D2dEffect {
    fn make(id: GUID, props: Vec<IPropertyValue>, sources: Vec<IGraphicsEffectSource>) -> IGraphicsEffect {
        D2dEffect { name: RefCell::new(HSTRING::new()), id, props, sources }.into()
    }
}

impl IGraphicsEffectSource_Impl for D2dEffect_Impl {}

impl IGraphicsEffect_Impl for D2dEffect_Impl {
    fn Name(&self) -> Result<HSTRING> {
        Ok(self.name.borrow().clone())
    }
    fn SetName(&self, name: &HSTRING) -> Result<()> {
        *self.name.borrow_mut() = name.clone();
        Ok(())
    }
}

impl IGraphicsEffectD2D1Interop_Impl for D2dEffect_Impl {
    fn GetEffectId(&self) -> Result<GUID> {
        Ok(self.id)
    }
    fn GetNamedPropertyMapping(&self, _: &PCWSTR, _: *mut u32, _: *mut GRAPHICS_EFFECT_PROPERTY_MAPPING) -> Result<()> {
        Err(E_NOTIMPL.into())
    }
    fn GetPropertyCount(&self) -> Result<u32> {
        Ok(self.props.len() as u32)
    }
    fn GetProperty(&self, i: u32) -> Result<IPropertyValue> {
        self.props.get(i as usize).cloned().ok_or(E_INVALIDARG.into())
    }
    fn GetSource(&self, i: u32) -> Result<IGraphicsEffectSource> {
        self.sources.get(i as usize).cloned().ok_or(E_INVALIDARG.into())
    }
    fn GetSourceCount(&self) -> Result<u32> {
        Ok(self.sources.len() as u32)
    }
}

fn prop_f32(v: f32) -> Result<IPropertyValue> {
    PropertyValue::CreateSingle(v)?.cast()
}

fn prop_u32(v: u32) -> Result<IPropertyValue> {
    PropertyValue::CreateUInt32(v)?.cast()
}

fn prop_bool(v: bool) -> Result<IPropertyValue> {
    PropertyValue::CreateBoolean(v)?.cast()
}

fn prop_f32_array(v: &[f32]) -> Result<IPropertyValue> {
    PropertyValue::CreateSingleArray(v)?.cast()
}

fn affine_2d(src: IGraphicsEffectSource, sx: f32, sy: f32, cx: f32, cy: f32, dx: f32, dy: f32) -> Result<IGraphicsEffect> {
    let mat = [sx, 0.0, 0.0, sy, cx * (1.0 - sx) + dx, cy * (1.0 - sy) + dy];
    Ok(D2dEffect::make(CLSID_AFFINE_2D, vec![prop_u32(1)?, prop_u32(1)?, prop_f32_array(&mat)?, prop_f32(0.0)?], vec![src]))
}

fn color_channel(src: IGraphicsEffectSource, r: f32, g: f32, b: f32, a: f32) -> Result<IGraphicsEffect> {
    let m: [f32; 20] = [
        r,   0.0, 0.0, 0.0,
        0.0, g,   0.0, 0.0,
        0.0, 0.0, b,   0.0,
        0.0, 0.0, 0.0, a,
        0.0, 0.0, 0.0, 0.0,
    ];
    Ok(D2dEffect::make(CLSID_COLOR_MATRIX, vec![prop_f32_array(&m)?, prop_u32(1)?, prop_bool(true)?], vec![src]))
}

fn add_sources(a: IGraphicsEffectSource, b: IGraphicsEffectSource) -> Result<IGraphicsEffect> {
    Ok(D2dEffect::make(CLSID_ARITHMETIC, vec![prop_f32_array(&[0.0, 1.0, 1.0, 0.0])?, prop_bool(true)?], vec![a, b]))
}

pub fn backdrop_blur_brush(comp: &Compositor, blur_sigma: f32, saturation: f32) -> Result<CompositionEffectBrush> {
    let blur_sigma = blur_sigma.clamp(0.5, 100.0);
    let saturation = saturation.clamp(0.0, 2.0);
    const SRC: &str = "backdrop";
    let source: IGraphicsEffectSource = CompositionEffectSourceParameter::Create(&HSTRING::from(SRC))?.cast()?;
    let blur = D2dEffect::make(CLSID_D2D1GaussianBlur, vec![prop_f32(blur_sigma)?, prop_u32(0)?, prop_u32(1)?], vec![source]);
    let sat = D2dEffect::make(CLSID_D2D1Saturation, vec![prop_f32(saturation)?], vec![blur.cast()?]);
    let brush = comp.CreateEffectFactory(&sat)?.CreateBrush()?;
    brush.SetSourceParameter(&HSTRING::from(SRC), &comp.CreateBackdropBrush()?)?;
    Ok(brush)
}

/// Crystal-clear interior liquid lens: slight convex magnification + gentle depth blur + boosted vibrance.
pub fn liquid_core_brush(comp: &Compositor, blur_sigma: f32, saturation: f32, w: f32, h: f32) -> Result<CompositionEffectBrush> {
    const SRC: &str = "backdrop";
    let fx = liquid_core_graph(SRC, blur_sigma, saturation, w, h)?;
    let brush = comp.CreateEffectFactory(&fx)?.CreateBrush()?;
    brush.SetSourceParameter(&HSTRING::from(SRC), &comp.CreateBackdropBrush()?)?;
    Ok(brush)
}

fn liquid_core_graph(src_name: &str, blur_sigma: f32, saturation: f32, w: f32, h: f32) -> Result<IGraphicsEffect> {
    let blur_sigma = blur_sigma.clamp(0.8, 40.0);
    let saturation = saturation.clamp(1.0, 2.0);
    let source: IGraphicsEffectSource = CompositionEffectSourceParameter::Create(&HSTRING::from(src_name))?.cast()?;
    // AffineTransform2D must sit directly on a CompositionEffectSourceParameter in Windows.UI.Composition.
    let lens = affine_2d(source, 1.018, 1.018, w.max(8.0) * 0.5, h.max(8.0) * 0.5, 0.0, 0.0)?;
    let blur = D2dEffect::make(CLSID_D2D1GaussianBlur, vec![prop_f32(blur_sigma)?, prop_u32(1)?, prop_u32(1)?], vec![lens.cast()?]);
    Ok(D2dEffect::make(CLSID_D2D1Saturation, vec![prop_f32(saturation)?], vec![blur.cast()?]))
}

/// Live curved-bevel optical refraction + RGB chromatic dispersion executed directly in DWM on GPU.
pub fn liquid_bevel_brush(comp: &Compositor, _blur_sigma: f32, w: f32, h: f32) -> Result<CompositionEffectBrush> {
    let fx = liquid_bevel_graph(w, h)?;
    let brush = comp.CreateEffectFactory(&fx)?.CreateBrush()?;
    let backdrop = comp.CreateBackdropBrush()?;
    for name in ["src_r", "src_g", "src_b"] {
        brush.SetSourceParameter(&HSTRING::from(name), &backdrop)?;
    }
    Ok(brush)
}

fn liquid_bevel_graph(w: f32, h: f32) -> Result<IGraphicsEffect> {
    let (w, h) = (w.max(20.0), h.max(20.0));
    let (cx, cy) = (w * 0.5, h * 0.5);

    // Convex meniscus refraction (> 1.0) bends live interior backdrop outward into the curved bevel
    // while keeping every sampled ray strictly inside [0..w, 0..h] with RGB Cauchy prism dispersion.
    let pull_x = 14.5f32.min(w * 0.18);
    let pull_y = 12.5f32.min(h * 0.22);
    let sx = 1.0 + (2.0 * pull_x / w).clamp(0.035, 0.34);
    let sy = 1.0 + (2.0 * pull_y / h).clamp(0.035, 0.38);
    let disp_x = (5.5 / w).clamp(0.006, 0.055);
    let disp_y = (5.5 / h).clamp(0.006, 0.065);

    let src_r: IGraphicsEffectSource = CompositionEffectSourceParameter::Create(&HSTRING::from("src_r"))?.cast()?;
    let src_g: IGraphicsEffectSource = CompositionEffectSourceParameter::Create(&HSTRING::from("src_g"))?.cast()?;
    let src_b: IGraphicsEffectSource = CompositionEffectSourceParameter::Create(&HSTRING::from("src_b"))?.cast()?;

    let r_ray = color_channel(affine_2d(src_r, sx - disp_x, sy - disp_y, cx, cy, -1.6, -1.2)?.cast()?, 1.04, 0.0, 0.0, 1.0)?;
    let g_ray = color_channel(affine_2d(src_g, sx, sy, cx, cy, 0.0, 0.0)?.cast()?, 0.0, 1.03, 0.0, 1.0)?;
    let b_ray = color_channel(affine_2d(src_b, sx + disp_x, sy + disp_y, cx, cy, 1.6, 1.2)?.cast()?, 0.0, 0.0, 1.08, 1.0)?;

    let rg = add_sources(r_ray.cast()?, g_ray.cast()?)?;
    let rgb = add_sources(rg.cast()?, b_ray.cast()?)?;
    Ok(D2dEffect::make(CLSID_D2D1Saturation, vec![prop_f32(1.85)?], vec![rgb.cast()?]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_liquid_glass_shaders_compile_in_composition() {
        let host = crate::host::Host::new().expect("compositor host");
        let core_fx = liquid_core_graph("backdrop", 3.5, 1.55, 420.0, 84.0).expect("core graph");
        let bevel_fx = liquid_bevel_graph(420.0, 84.0).expect("bevel graph");
        let core_fac = host.compositor.CreateEffectFactory(&core_fx).expect("compile core D2D shader");
        let bevel_fac = host.compositor.CreateEffectFactory(&bevel_fx).expect("compile bevel D2D shader");
        let _ = (core_fac.CreateBrush().expect("core brush"), bevel_fac.CreateBrush().expect("bevel brush"));
        let (mask_ng, caustic_ng) = super::super::liquid::brushes(&host.compositor, 22.0, 420.0, 84.0, true).expect("nine-grid brushes");
        drop((mask_ng, caustic_ng));
    }
}
