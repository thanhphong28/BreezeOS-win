use super::spring;
use nacre_core::notch::shade;
use nacre_ui::gfx::Gfx;
use nacre_ui::host::Surface;
use std::collections::HashMap;
use windows::core::w;
use windows::UI::Composition::{CompositionBrush, CompositionObject, CompositionStretch, Compositor, ContainerVisual, SpriteVisual};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_core::{Interface, HSTRING};
use windows_numerics::{Vector2, Vector3};

pub struct Shadow {
    pub hwnd: HWND,
    comp: Compositor,
    _surface: Surface,
    stage: ContainerVisual,
    sprite: SpriteVisual,
    brushes: HashMap<(i32, i32, i32), (CompositionBrush, f32, f32)>,
    key: (i32, i32, i32),
    pub pad: f32,
    pub island: f32,
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

impl Shadow {
    pub fn new(comp: &Compositor, size: (i32, i32), pad: f32) -> windows_core::Result<Shadow> {
        unsafe {
            let hi: HINSTANCE = GetModuleHandleW(None)?.into();
            let class = w!("BreezeOSNotchShadow");
            RegisterClassW(&WNDCLASSW { lpfnWndProc: Some(proc), hInstance: hi, lpszClassName: class, ..Default::default() });
            let hwnd = CreateWindowExW(
                WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED,
                class,
                w!("breezeos-notch-shadow"),
                WS_POPUP,
                0, 0, size.0, size.1,
                None, None, Some(hi), None,
            )?;
            let _ = SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), 255, LWA_ALPHA);
            let surface = nacre_ui::host::attach(comp, hwnd, (size.0 as f32, size.1 as f32))?;
            let stage = comp.CreateContainerVisual()?;
            surface.root.Children()?.InsertAtTop(&stage)?;
            let sprite = comp.CreateSpriteVisual()?;
            sprite.SetOpacity(0.0)?;
            stage.Children()?.InsertAtTop(&sprite)?;
            Ok(Shadow { hwnd, comp: comp.clone(), _surface: surface, stage, sprite, brushes: HashMap::new(), key: (0, 0, 0), pad, island: 0.0 })
        }
    }

    pub fn place_window(&self, notch: HWND, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            let _ = SetWindowPos(self.hwnd, Some(notch), x, y, w, h, SWP_NOACTIVATE | SWP_SHOWWINDOW);
        }
        let _ = self._surface.root.SetSize(Vector2 { X: w as f32, Y: h as f32 });
    }

    pub fn set_visible(&self, on: bool) {
        unsafe {
            let _ = ShowWindow(self.hwnd, if on { SW_SHOWNOACTIVATE } else { SW_HIDE });
        }
    }

    fn brush(&mut self, gfx: &Gfx, radius: f32, blur: f32, dy: f32) -> Option<(CompositionBrush, f32, f32)> {
        let key = (radius.round() as i32, blur.round() as i32, dy.round() as i32);
        if let Some(b) = self.brushes.get(&key) {
            return Some(b.clone());
        }
        let s = shade::rounded(radius, blur, dy);
        let make = || -> windows_core::Result<CompositionBrush> {
            let surf = gfx.surface_from_bgra(s.w, s.h, &s.bgra)?;
            let src = self.comp.CreateSurfaceBrushWithSurface(&surf)?;
            src.SetStretch(CompositionStretch::Fill)?;
            let ng = self.comp.CreateNineGridBrush()?;
            ng.SetSource(&src)?;
            let (l, t, r, b) = s.insets;
            ng.SetInsetsWithValues(l, t, r, b)?;
            ng.cast()
        };
        let b = (make().ok()?, s.margin as f32, s.dy as f32);
        if self.brushes.len() > 16 {
            self.brushes.clear();
        }
        self.brushes.insert(key, b.clone());
        Some(b)
    }

    pub fn follow(&mut self, gfx: &Gfx, body: (f32, f32, f32, f32), radius: f32, look: (f32, f32, f32), spring_params: Option<(f32, u32)>) {
        let (blur, dy, opacity) = look;
        let Some((brush, m, dy)) = self.brush(gfx, radius, blur, dy) else { return };
        let key = (radius.round() as i32, blur.round() as i32, dy.round() as i32);
        if key != self.key {
            let _ = self.sprite.SetBrush(&brush);
            self.key = key;
        }
        let (x, y, w, h) = (body.0 + self.pad - m, body.1 - m, body.2 + 2.0 * m, body.3 + 2.0 * m + dy);
        match spring_params {
            None => {
                let _ = self.sprite.SetOffset(Vector3 { X: x, Y: y, Z: 0.0 });
                let _ = self.sprite.SetSize(Vector2 { X: w, Y: h });
                let _ = self.sprite.SetOpacity(opacity);
            }
            Some((ratio, period)) => {
                let o: CompositionObject = self.sprite.cast().unwrap();
                spring(&self.comp, &o, "Offset.X", x, ratio, period);
                spring(&self.comp, &o, "Offset.Y", y, ratio, period);
                spring(&self.comp, &o, "Size.X", w, ratio, period);
                spring(&self.comp, &o, "Size.Y", h, ratio, period);
                let _ = fade(&self.comp, &o, opacity, 160);
            }
        }
    }

    pub fn stage_y(&self, y: f32, ratio: f32, period: u32) {
        let o: CompositionObject = self.stage.cast().unwrap();
        spring(&self.comp, &o, "Offset.Y", y + self.island, ratio, period);
    }

    pub fn set_island(&mut self, gap: f32, shown: bool) {
        if (gap - self.island).abs() < 0.5 {
            return;
        }
        self.island = gap;
        if shown {
            if let Ok(o) = self.stage.cast::<CompositionObject>() {
                let _ = o.StopAnimation(&HSTRING::from("Offset.Y"));
            }
            let _ = self.stage.SetOffset(Vector3 { X: 0.0, Y: gap, Z: 0.0 });
        }
    }
}

fn fade(comp: &Compositor, o: &CompositionObject, to: f32, ms: u32) -> windows_core::Result<()> {
    let a = comp.CreateScalarKeyFrameAnimation()?;
    a.InsertKeyFrame(1.0, to)?;
    a.SetDuration(windows::Foundation::TimeSpan { Duration: ms as i64 * 10_000 })?;
    o.StartAnimation(&HSTRING::from("Opacity"), &a)
}
