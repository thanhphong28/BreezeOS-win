use std::sync::Once;
use windows::core::implement;
use windows::Win32::System::Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL};
use windows::Win32::System::Ole::{IDropTarget, IDropTarget_Impl, OleInitialize, RegisterDragDrop, RevokeDragDrop, CF_HDROP, DROPEFFECT, DROPEFFECT_NONE};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::Foundation::POINTL;
use windows::core::w;
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, BLACK_BRUSH, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_core::Result;

const WM_MOUSELEAVE: u32 = 0x02A3;
static REGISTER: Once = Once::new();

struct State {
    target: HWND,
    enter: u32,
    leave: u32,
    inside: bool,
    last: u64,
}

unsafe extern "system" fn proc(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let st = GetWindowLongPtrW(h, GWLP_USERDATA) as *mut State;
    match m {
        WM_MOUSEMOVE if !st.is_null() => {
            let s = &mut *st;
            let now = windows::Win32::System::SystemInformation::GetTickCount64();
            if s.inside && now.saturating_sub(s.last) < 400 {
                return LRESULT(0);
            }
            s.inside = true;
            s.last = now;
            let mut t = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: h,
                dwHoverTime: 0,
            };
            let _ = TrackMouseEvent(&mut t);
            let _ = PostMessageW(Some(s.target), s.enter, WPARAM(0), LPARAM(0));
            LRESULT(0)
        }
        WM_MOUSELEAVE if !st.is_null() => {
            let s = &mut *st;
            s.inside = false;
            let _ = PostMessageW(Some(s.target), s.leave, WPARAM(0), LPARAM(0));
            LRESULT(0)
        }
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_NCDESTROY => {
            if !st.is_null() {
                drop(Box::from_raw(st));
                SetWindowLongPtrW(h, GWLP_USERDATA, 0);
            }
            DefWindowProcW(h, m, w, l)
        }
        _ => DefWindowProcW(h, m, w, l),
    }
}

#[implement(IDropTarget)]
struct DragProbe {
    target: isize,
    msg: u32,
}

impl IDropTarget_Impl for DragProbe_Impl {
    fn DragEnter(&self, data: windows_core::Ref<IDataObject>, _keys: MODIFIERKEYS_FLAGS, _pt: &POINTL, effect: *mut DROPEFFECT) -> windows_core::Result<()> {
        let fmt = FORMATETC { cfFormat: CF_HDROP.0, ptd: std::ptr::null_mut(), dwAspect: DVASPECT_CONTENT.0, lindex: -1, tymed: TYMED_HGLOBAL.0 as u32 };
        let files = data.as_ref().is_some_and(|d| unsafe { d.QueryGetData(&fmt).is_ok() });
        if files {
            unsafe {
                let _ = PostMessageW(Some(HWND(self.target as *mut _)), self.msg, WPARAM(0), LPARAM(0));
            }
        }
        if !effect.is_null() {
            unsafe { *effect = DROPEFFECT_NONE };
        }
        Ok(())
    }

    fn DragOver(&self, _keys: MODIFIERKEYS_FLAGS, _pt: &POINTL, effect: *mut DROPEFFECT) -> windows_core::Result<()> {
        if !effect.is_null() {
            unsafe { *effect = DROPEFFECT_NONE };
        }
        Ok(())
    }

    fn DragLeave(&self) -> windows_core::Result<()> {
        Ok(())
    }

    fn Drop(&self, _data: windows_core::Ref<IDataObject>, _keys: MODIFIERKEYS_FLAGS, _pt: &POINTL, effect: *mut DROPEFFECT) -> windows_core::Result<()> {
        if !effect.is_null() {
            unsafe { *effect = DROPEFFECT_NONE };
        }
        Ok(())
    }
}

pub struct Strip {
    hwnd: HWND,
    target: HWND,
    drag: Option<IDropTarget>,
}

impl Drop for Strip {
    fn drop(&mut self) {
        unsafe {
            if self.drag.take().is_some() {
                let _ = RevokeDragDrop(self.hwnd);
            }
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

impl Strip {
    pub fn new(target: HWND, enter: u32, leave: u32) -> Result<Strip> {
        unsafe {
            let hi: HINSTANCE = GetModuleHandleW(None)?.into();
            let class = w!("NacreEdgeStrip");
            REGISTER.call_once(|| {
                RegisterClassW(&WNDCLASSW {
                    lpfnWndProc: Some(proc),
                    hInstance: hi,
                    lpszClassName: class,
                    hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
                    ..Default::default()
                });
            });
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
                class,
                w!("nacre-strip"),
                WS_POPUP,
                0, 0, 10, 2,
                None, None, Some(hi), None,
            )?;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(Box::new(State { target, enter, leave, inside: false, last: 0 })) as isize);
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 1, LWA_ALPHA)?;
            Ok(Strip { hwnd, target, drag: None })
        }
    }

    pub fn on_file_drag(mut self, msg: u32) -> Strip {
        unsafe {
            let _ = OleInitialize(None);
            let t: IDropTarget = DragProbe { target: self.target.0 as isize, msg }.into();
            if RegisterDragDrop(self.hwnd, &t).is_ok() {
                self.drag = Some(t);
            }
        }
        self
    }

    pub fn place(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe {
            let _ = SetWindowPos(self.hwnd, Some(HWND_TOPMOST), x, y, w, h, SWP_NOACTIVATE);
        }
    }

    pub fn set_visible(&self, on: bool) {
        unsafe {
            let _ = ShowWindow(self.hwnd, if on { SW_SHOWNOACTIVATE } else { SW_HIDE });
        }
    }
}
