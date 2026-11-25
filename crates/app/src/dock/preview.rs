use crate::popup::Popup;
use nacre_platform::text::{render, render_face, Style, ICON_FACE};
use nacre_platform::wins::thumbnail;
use nacre_ui::gfx::Gfx;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::core::w;
use windows::UI::Color;
use windows::UI::Composition::{CompositionDrawingSurface, CompositionStretch, Compositor, SpriteVisual};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_core::Result;
use windows_numerics::{Vector2, Vector3};

pub const PREVIEW_CLICK: u32 = WM_APP + 7;
pub const PREVIEW_ENTER: u32 = WM_APP + 8;
pub const PREVIEW_LEAVE: u32 = WM_APP + 9;
pub const PREVIEW_CLOSE_WIN: u32 = WM_APP + 10;
const WM_MOUSELEAVE: u32 = 0x02A3;

static INSIDE: AtomicBool = AtomicBool::new(false);

struct HitCell {
    rect: (f32, f32, f32, f32),
    close: (f32, f32, f32, f32),
    hwnd: isize,
    card_bg: SpriteVisual,
    close_bg: SpriteVisual,
}

struct Live {
    target: HWND,
    cells: Vec<HitCell>,
    hover: Option<(usize, bool)>,
}

thread_local! {
    static LIVE: RefCell<Option<Live>> = const { RefCell::new(None) };
}

fn contains(r: (f32, f32, f32, f32), x: f32, y: f32) -> bool {
    x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3
}

fn rounded_box(comp: &Compositor, r: (f32, f32, f32, f32), radius: f32, color: Color) -> Result<SpriteVisual> {
    let v = comp.CreateSpriteVisual()?;
    v.SetBrush(&comp.CreateColorBrushWithColor(color)?)?;
    v.SetSize(Vector2 { X: r.2, Y: r.3 })?;
    v.SetOffset(Vector3 { X: r.0, Y: r.1, Z: 0.0 })?;
    let g = comp.CreateRoundedRectangleGeometry()?;
    g.SetSize(Vector2 { X: r.2, Y: r.3 })?;
    g.SetCornerRadius(Vector2 { X: radius, Y: radius })?;
    let clip = comp.CreateGeometricClip()?;
    clip.SetGeometry(&g)?;
    v.SetClip(&clip)?;
    v.SetOpacity(0.0)?;
    Ok(v)
}

fn update_hover(live: &mut Live, next: Option<(usize, bool)>) {
    if live.hover == next {
        return;
    }
    if let Some((old_i, _)) = live.hover {
        if let Some(c) = live.cells.get(old_i) {
            let _ = c.card_bg.SetOpacity(0.0);
            let _ = c.close_bg.SetOpacity(0.0);
        }
    }
    live.hover = next;
    if let Some((i, on_close)) = next {
        if let Some(c) = live.cells.get(i) {
            let _ = c.card_bg.SetOpacity(1.0);
            let _ = c.close_bg.SetOpacity(if on_close { 1.0 } else { 0.0 });
        }
    }
}

unsafe extern "system" fn proc(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match m {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_MOUSEMOVE => {
            let (x, y) = ((l.0 as i16) as f32, ((l.0 >> 16) as i16) as f32);
            LIVE.with(|c| {
                if let Some(live) = c.borrow_mut().as_mut() {
                    if !INSIDE.swap(true, Ordering::Relaxed) {
                        let mut t = TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE,
                            hwndTrack: h,
                            dwHoverTime: 0,
                        };
                        let _ = TrackMouseEvent(&mut t);
                        let _ = PostMessageW(Some(live.target), PREVIEW_ENTER, WPARAM(0), LPARAM(0));
                    }
                    let hit = live
                        .cells
                        .iter()
                        .enumerate()
                        .find(|(_, cell)| contains(cell.rect, x, y))
                        .map(|(i, cell)| (i, contains(cell.close, x, y)));
                    update_hover(live, hit);
                }
            });
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            INSIDE.store(false, Ordering::Relaxed);
            LIVE.with(|c| {
                if let Some(live) = c.borrow_mut().as_mut() {
                    update_hover(live, None);
                    let _ = PostMessageW(Some(live.target), PREVIEW_LEAVE, WPARAM(0), LPARAM(0));
                }
            });
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let (x, y) = ((l.0 as i16) as f32, ((l.0 >> 16) as i16) as f32);
            LIVE.with(|c| {
                if let Some(live) = c.borrow().as_ref() {
                    if let Some(cell) = live.cells.iter().find(|cell| contains(cell.rect, x, y)) {
                        let msg = if contains(cell.close, x, y) { PREVIEW_CLOSE_WIN } else { PREVIEW_CLICK };
                        let _ = PostMessageW(Some(live.target), msg, WPARAM(cell.hwnd as usize), LPARAM(0));
                    }
                }
            });
            LRESULT(0)
        }
        _ => DefWindowProcW(h, m, w, l),
    }
}

pub struct Preview {
    target: HWND,
    popup: Popup,
    thumbs: Vec<isize>,
    dpi: f32,
}

impl Preview {
    pub fn new(comp: &Compositor, dock_hwnd: HWND, dpi: f32) -> Result<Preview> {
        let popup = Popup::new(
            comp,
            w!("NacrePreview"),
            Some(proc),
            WINDOW_EX_STYLE(0),
            crate::prefs::glass((10.0, 10.0), 14.0 * dpi, crate::prefs::Role::Dark),
        )?;
        Ok(Preview { target: dock_hwnd, popup, thumbs: vec![], dpi })
    }

    pub fn panel_mut(&mut self) -> &mut nacre_ui::glass::GlassPanel {
        &mut self.popup.panel
    }

    pub fn is_visible(&self) -> bool {
        self.popup.is_visible()
    }

    pub fn show(
        &mut self,
        comp: &Compositor,
        gfx: &Gfx,
        windows: &[(isize, String, bool)],
        icon: Option<&CompositionDrawingSurface>,
        cx: i32,
        bottom: i32,
    ) -> Result<()> {
        self.clear();
        let d = self.dpi;
        let n = windows.len().min(6);
        if n == 0 {
            return Ok(());
        }
        let (pad, cgap, cpad) = ((8.0 * d).round() as i32, (6.0 * d).round() as i32, (8.0 * d).round() as i32);
        let (hdr_h, inner_gap) = ((24.0 * d).round() as i32, (6.0 * d).round() as i32);
        let (tw, th) = ((208.0 * d).round() as i32, (124.0 * d).round() as i32);
        let cw = tw + 2 * cpad;
        let ch = cpad + hdr_h + inner_gap + th + cpad;
        let w = 2 * pad + n as i32 * cw + (n as i32 - 1) * cgap;
        let h = 2 * pad + ch;
        let x = nacre_platform::monitors::at(cx, bottom - 1).area.clamp_x(cx - w / 2, w, 4);
        self.popup.set_rect(x, bottom - h, w, h)?;
        self.popup.panel.set_radius(14.0 * d)?;
        self.popup.show();

        let title_style = Style { px: (12.0 * d).round() as i32, weight: 500 };
        let close_sz = (22.0 * d).round() as i32;
        let close_bmp = render_face("\u{E8BB}", &Style { px: (10.0 * d).round() as i32, weight: 400 }, ICON_FACE, close_sz, close_sz)
            .or_else(|| render("\u{00D7}", &Style { px: (15.0 * d).round() as i32, weight: 500 }, close_sz, close_sz));
        let close_surf = match close_bmp {
            Some(b) => Some(gfx.surface_from_bgra(b.w, b.h, &b.bgra)?),
            None => None,
        };

        let mut hit_cells = Vec::with_capacity(n);
        for (i, (hwnd, title, _minimized)) in windows.iter().take(n).enumerate() {
            let x0 = pad + i as i32 * (cw + cgap);
            let y0 = pad;
            let card_rect = (x0 as f32, y0 as f32, cw as f32, ch as f32);
            let card_bg = rounded_box(comp, card_rect, 10.0 * d, Color { A: 24, R: 255, G: 255, B: 255 })?;
            self.popup.content.Children()?.InsertAtTop(&card_bg)?;

            // Windows 11 header: small app icon + window title + quick-close (X) button.
            let hdr_y = y0 + cpad;
            let icon_sz = (16.0 * d).round() as i32;
            let mut title_x = x0 + cpad;
            if let Some(s) = icon {
                let brush = comp.CreateSurfaceBrushWithSurface(s)?;
                brush.SetStretch(CompositionStretch::Uniform)?;
                let sp = comp.CreateSpriteVisual()?;
                sp.SetBrush(&brush)?;
                sp.SetSize(Vector2 { X: icon_sz as f32, Y: icon_sz as f32 })?;
                sp.SetOffset(Vector3 { X: title_x as f32, Y: (hdr_y + (hdr_h - icon_sz) / 2) as f32, Z: 0.0 })?;
                self.popup.content.Children()?.InsertAtTop(&sp)?;
                title_x += icon_sz + (6.0 * d).round() as i32;
            }

            let close_x = x0 + cw - cpad - close_sz;
            let close_y = hdr_y + (hdr_h - close_sz) / 2;
            let close_rect = (close_x as f32, close_y as f32, close_sz as f32, close_sz as f32);
            let close_bg = rounded_box(comp, close_rect, 6.0 * d, Color { A: 235, R: 196, G: 43, B: 28 })?;
            self.popup.content.Children()?.InsertAtTop(&close_bg)?;
            if let Some(cs) = &close_surf {
                let brush = comp.CreateSurfaceBrushWithSurface(cs)?;
                brush.SetStretch(CompositionStretch::None)?;
                let sp = comp.CreateSpriteVisual()?;
                sp.SetBrush(&brush)?;
                sp.SetSize(Vector2 { X: close_sz as f32, Y: close_sz as f32 })?;
                sp.SetOffset(Vector3 { X: close_x as f32, Y: close_y as f32, Z: 0.0 })?;
                sp.SetOpacity(0.85)?;
                self.popup.content.Children()?.InsertAtTop(&sp)?;
            }

            let title_w = (close_x - (6.0 * d).round() as i32 - title_x).max(24);
            if !title.is_empty() {
                if let Some(bmp) = render(title, &title_style, title_w, hdr_h) {
                    let surface = gfx.surface_from_bgra(bmp.w, bmp.h, &bmp.bgra)?;
                    let brush = comp.CreateSurfaceBrushWithSurface(&surface)?;
                    brush.SetStretch(CompositionStretch::None)?;
                    let sp = comp.CreateSpriteVisual()?;
                    sp.SetBrush(&brush)?;
                    sp.SetSize(Vector2 { X: title_w as f32, Y: hdr_h as f32 })?;
                    sp.SetOffset(Vector3 { X: title_x as f32, Y: hdr_y as f32, Z: 0.0 })?;
                    self.popup.content.Children()?.InsertAtTop(&sp)?;
                }
            }

            // Body: native zero-copy Windows DWM thumbnail first (works for minimized windows too
            // whenever DWM holds the cached frame); fallback to fx GPU surface if DWM reports iconic stub.
            let bx = x0 + cpad;
            let by = hdr_y + hdr_h + inner_gap;
            let mut thumb = thumbnail::register(self.popup.hwnd.0 as isize, *hwnd);
            let dwm_size = thumb.and_then(thumbnail::source_size).filter(|&(sw, sh)| sw >= 180 && sh >= 80);
            if let (Some(t), Some((sw, sh))) = (thumb, dwm_size) {
                let k = (tw as f32 / sw as f32).min(th as f32 / sh as f32);
                let (dw, dh) = (((sw as f32 * k).round() as i32).max(1), ((sh as f32 * k).round() as i32).max(1));
                let (dx, dy) = (bx + (tw - dw) / 2, by + (th - dh) / 2);
                thumbnail::place(t, (dx, dy, dx + dw, dy + dh), 255);
                self.thumbs.push(t);
            } else {
                if let Some(t) = thumb.take() {
                    thumbnail::unregister(t);
                }
                if let Some((surf, sw, sh)) = crate::fx::surface_for(*hwnd).filter(|&(_, w, h)| w > 0 && h > 0) {
                    let k = (tw as f32 / sw as f32).min(th as f32 / sh as f32);
                    let (dw, dh) = (((sw as f32 * k).round() as i32).max(1), ((sh as f32 * k).round() as i32).max(1));
                    let brush = comp.CreateSurfaceBrushWithSurface(&surf)?;
                    brush.SetStretch(CompositionStretch::Fill)?;
                    let sp = comp.CreateSpriteVisual()?;
                    sp.SetBrush(&brush)?;
                    sp.SetSize(Vector2 { X: dw as f32, Y: dh as f32 })?;
                    sp.SetOffset(Vector3 { X: (bx + (tw - dw) / 2) as f32, Y: (by + (th - dh) / 2) as f32, Z: 0.0 })?;
                    self.popup.content.Children()?.InsertAtTop(&sp)?;
                } else if let Some(s) = icon {
                    let sz = (56.0 * d).round() as i32;
                    let brush = comp.CreateSurfaceBrushWithSurface(s)?;
                    brush.SetStretch(CompositionStretch::Uniform)?;
                    let sp = comp.CreateSpriteVisual()?;
                    sp.SetBrush(&brush)?;
                    sp.SetSize(Vector2 { X: sz as f32, Y: sz as f32 })?;
                    sp.SetOffset(Vector3 { X: (bx + (tw - sz) / 2) as f32, Y: (by + (th - sz) / 2) as f32, Z: 0.0 })?;
                    sp.SetOpacity(0.7)?;
                    self.popup.content.Children()?.InsertAtTop(&sp)?;
                }
            }

            hit_cells.push(HitCell { rect: card_rect, close: close_rect, hwnd: *hwnd, card_bg, close_bg });
        }

        LIVE.with(|c| {
            *c.borrow_mut() = Some(Live { target: self.target, cells: hit_cells, hover: None });
        });
        Ok(())
    }

    fn clear(&mut self) {
        for t in self.thumbs.drain(..) {
            thumbnail::unregister(t);
        }
        LIVE.with(|c| {
            if c.borrow().as_ref().is_some_and(|l| l.target == self.target) {
                *c.borrow_mut() = None;
            }
        });
        if let Ok(kids) = self.popup.content.Children() {
            let _ = kids.RemoveAll();
        }
    }

    pub fn hide(&mut self) {
        if self.popup.is_visible() || !self.thumbs.is_empty() {
            self.clear();
            self.popup.hide();
        }
    }

    pub fn destroy(&mut self) {
        self.clear();
        unsafe {
            let _ = DestroyWindow(self.popup.hwnd);
        }
    }
}
