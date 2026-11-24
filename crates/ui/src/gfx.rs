use windows::Graphics::DirectX::{DirectXAlphaMode, DirectXPixelFormat};
use windows::Foundation::Size;
use windows::UI::Composition::{CompositionDrawingSurface, CompositionGraphicsDevice, Compositor};
use windows::Win32::Foundation::{HMODULE, POINT};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D, D3D11_BOX, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{IDXGIAdapter, IDXGIDevice};
use windows::Win32::System::WinRT::Composition::{ICompositionDrawingSurfaceInterop, ICompositorInterop};
use windows_core::{Interface, Result};

#[derive(Clone)]
pub struct Gfx {
    device: ID3D11Device,
    ctx: ID3D11DeviceContext,
    graphics: CompositionGraphicsDevice,
}

static REPLACED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

thread_local! {
    static SHARED: std::cell::RefCell<Option<Gfx>> = const { std::cell::RefCell::new(None) };
}

impl Gfx {
    pub fn shared(comp: &Compositor) -> Result<Gfx> {
        if let Some(g) = SHARED.with(|c| c.borrow().clone()) {
            return Ok(g);
        }
        let g = Self::new(comp)?;
        let _ = g.graphics.RenderingDeviceReplaced(&windows::Foundation::TypedEventHandler::new(|_, _| {
            REPLACED.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        }));
        SHARED.with(|c| *c.borrow_mut() = Some(g.clone()));
        Ok(g)
    }

    pub fn lost(&self) -> bool {
        unsafe { self.device.GetDeviceRemovedReason().is_err() }
    }

    pub fn shared_lost() -> bool {
        REPLACED.load(std::sync::atomic::Ordering::Relaxed) || SHARED.with(|c| c.borrow().as_ref().is_some_and(|g| g.lost()))
    }

    pub fn new(comp: &Compositor) -> Result<Gfx> {
        Self::with_hardware(comp, std::env::var_os("NACRE_GFX").is_some_and(|v| v == "hw"))
    }

    pub fn new_hardware(comp: &Compositor) -> Result<Gfx> {
        Self::with_hardware(comp, true)
    }

    fn with_hardware(comp: &Compositor, prefer_hw: bool) -> Result<Gfx> {
        let (device, ctx) = if prefer_hw {
            create_device(D3D_DRIVER_TYPE_HARDWARE).or_else(|_| create_device(D3D_DRIVER_TYPE_WARP))?
        } else {
            create_device(D3D_DRIVER_TYPE_WARP).or_else(|_| create_device(D3D_DRIVER_TYPE_HARDWARE))?
        };
        let interop: ICompositorInterop = comp.cast()?;
        let graphics = unsafe { interop.CreateGraphicsDevice(&device)? };
        Ok(Gfx { device, ctx, graphics })
    }

    pub fn d3d_device(&self) -> Result<windows::Graphics::DirectX::Direct3D11::IDirect3DDevice> {
        let dxgi: IDXGIDevice = self.device.cast()?;
        let insp = unsafe { windows::Win32::System::WinRT::Direct3D11::CreateDirect3D11DeviceFromDXGIDevice(&dxgi)? };
        insp.cast()
    }

    pub fn new_surface(&self, w: u32, h: u32) -> Result<CompositionDrawingSurface> {
        self.graphics.CreateDrawingSurface(Size { Width: w as f32, Height: h as f32 }, DirectXPixelFormat::B8G8R8A8UIntNormalized, DirectXAlphaMode::Premultiplied)
    }

    pub fn copy_into(&self, surface: &CompositionDrawingSurface, tex: &ID3D11Texture2D, w: u32, h: u32) -> Result<()> {
        let cur = surface.SizeInt32()?;
        if cur.Width != w as i32 || cur.Height != h as i32 {
            surface.Resize(windows::Graphics::SizeInt32 { Width: w as i32, Height: h as i32 })?;
        }
        let interop: ICompositionDrawingSurfaceInterop = surface.cast()?;
        unsafe {
            let mut offset = POINT::default();
            let dest: ID3D11Texture2D = interop.BeginDraw(None, &mut offset)?;
            let src = D3D11_BOX { left: 0, top: 0, front: 0, right: w, bottom: h, back: 1 };
            self.ctx.CopySubresourceRegion(&dest, 0, offset.x as u32, offset.y as u32, 0, tex, 0, Some(&src));
            interop.EndDraw()?;
        }
        Ok(())
    }

    pub fn surface_from_bgra(&self, w: u32, h: u32, bgra: &[u8]) -> Result<CompositionDrawingSurface> {
        debug_assert_eq!(bgra.len(), (w * h * 4) as usize);
        let surface = self.graphics.CreateDrawingSurface(
            Size { Width: w as f32, Height: h as f32 },
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            DirectXAlphaMode::Premultiplied,
        )?;
        let interop: ICompositionDrawingSurfaceInterop = surface.cast()?;
        unsafe {
            let mut offset = POINT::default();
            let tex: ID3D11Texture2D = interop.BeginDraw(None, &mut offset)?;
            let region = D3D11_BOX {
                left: offset.x as u32,
                top: offset.y as u32,
                front: 0,
                right: offset.x as u32 + w,
                bottom: offset.y as u32 + h,
                back: 1,
            };
            self.ctx.UpdateSubresource(&tex, 0, Some(&region), bgra.as_ptr() as *const _, w * 4, 0);
            interop.EndDraw()?;
        }
        Ok(surface)
    }
}

fn create_device(kind: windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE) -> Result<(ID3D11Device, ID3D11DeviceContext)> {
    let mut device = None;
    let mut ctx = None;
    unsafe {
        D3D11CreateDevice(
            None::<&IDXGIAdapter>,
            kind,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut ctx),
        )?;
    }
    Ok((device.ok_or(windows_core::Error::empty())?, ctx.ok_or(windows_core::Error::empty())?))
}
