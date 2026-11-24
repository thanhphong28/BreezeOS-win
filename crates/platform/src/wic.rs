use crate::icons::Bitmap;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, IWICImagingFactory, WICBitmapDitherTypeNone,
    WICBitmapInterpolationModeFant, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand, WICRect,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Shell::SHCreateMemStream;

pub fn decode_square(bytes: &[u8], px: u32) -> Option<Bitmap> {
    unsafe {
        let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()?;
        let stream = SHCreateMemStream(Some(bytes))?;
        let decoder = factory
            .CreateDecoderFromStream(&stream, std::ptr::null(), WICDecodeMetadataCacheOnDemand)
            .ok()?;
        let frame = decoder.GetFrame(0).ok()?;
        let (mut w, mut h) = (0u32, 0u32);
        frame.GetSize(&mut w, &mut h).ok()?;
        let (x, y, side) = center_square(w, h)?;
        let clipper = factory.CreateBitmapClipper().ok()?;
        clipper.Initialize(&frame, &WICRect { X: x as i32, Y: y as i32, Width: side as i32, Height: side as i32 }).ok()?;
        let scaler = factory.CreateBitmapScaler().ok()?;
        scaler.Initialize(&clipper, px, px, WICBitmapInterpolationModeFant).ok()?;
        let conv = factory.CreateFormatConverter().ok()?;
        conv.Initialize(&scaler, &GUID_WICPixelFormat32bppPBGRA, WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeCustom)
            .ok()?;
        let mut buf = vec![0u8; (px * px * 4) as usize];
        conv.CopyPixels(std::ptr::null(), px * 4, &mut buf).ok()?;
        Some(Bitmap { w: px, h: px, bgra: buf })
    }
}

pub fn center_square(w: u32, h: u32) -> Option<(u32, u32, u32)> {
    if w == 0 || h == 0 {
        return None;
    }
    let side = w.min(h);
    Some(((w - side) / 2, (h - side) / 2, side))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_and_tall_images_are_cropped_from_the_middle() {
        assert_eq!(center_square(1280, 720), Some((280, 0, 720)));
        assert_eq!(center_square(300, 500), Some((0, 100, 300)));
        assert_eq!(center_square(512, 512), Some((0, 0, 512)));
        assert_eq!(center_square(0, 10), None);
    }

    #[test]
    fn a_wide_thumbnail_keeps_its_proportions() {
        unsafe {
            use windows::Win32::Graphics::Imaging::{GUID_ContainerFormatPng, WICBitmapEncoderNoCache};
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).unwrap();
            let (w, h) = (160u32, 90u32);
            let mut px = vec![0u8; (w * h * 4) as usize];
            for y in 0..h {
                for x in 0..w {
                    let i = ((y * w + x) * 4) as usize;
                    let inside = x >= 35 && x < 125;
                    let c = if inside { [0u8, 0, 255, 255] } else { [255u8, 0, 0, 255] };
                    px[i..i + 4].copy_from_slice(&c);
                }
            }
            let bmp = factory.CreateBitmapFromMemory(w, h, &GUID_WICPixelFormat32bppPBGRA, w * 4, &px).unwrap();
            let stream = SHCreateMemStream(None).unwrap();
            let enc = factory.CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null()).unwrap();
            enc.Initialize(&stream, WICBitmapEncoderNoCache).unwrap();
            let mut frame = None;
            enc.CreateNewFrame(&mut frame, std::ptr::null_mut()).unwrap();
            let frame = frame.unwrap();
            frame.Initialize(None).unwrap();
            frame.WriteSource(&bmp, std::ptr::null()).unwrap();
            frame.Commit().unwrap();
            enc.Commit().unwrap();
            let mut stat = windows::Win32::System::Com::STATSTG::default();
            stream.Stat(&mut stat, windows::Win32::System::Com::STATFLAG_NONAME).unwrap();
            let len = stat.cbSize as usize;
            let _ = stream.Seek(0, windows::Win32::System::Com::STREAM_SEEK_SET, None);
            let mut bytes = vec![0u8; len];
            let mut read = 0u32;
            let _ = stream.Read(bytes.as_mut_ptr() as *mut _, len as u32, Some(&mut read));
            let out = decode_square(&bytes[..read as usize], 64).unwrap();
            let at = |x: u32, y: u32| out.bgra[((y * 64 + x) * 4) as usize + 2];
            assert!(at(2, 32) > 200 && at(61, 32) > 200 && at(32, 32) > 200, "potongan tengah 90x90 seluruhnya merah, tidak ada biru dari tepi");
        }
    }
}

pub fn decode_file_fit(path: &str, px: u32) -> Option<Bitmap> {
    unsafe {
        let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()?;
        let decoder = factory
            .CreateDecoderFromFilename(&windows::core::HSTRING::from(path), None, windows::Win32::Foundation::GENERIC_READ, WICDecodeMetadataCacheOnDemand)
            .ok()?;
        let frame = decoder.GetFrame(0).ok()?;
        let (mut w, mut h) = (0u32, 0u32);
        frame.GetSize(&mut w, &mut h).ok()?;
        if w == 0 || h == 0 {
            return None;
        }
        let k = (px as f32 / w as f32).min(px as f32 / h as f32);
        let (sw, sh) = (((w as f32 * k).round() as u32).max(1), ((h as f32 * k).round() as u32).max(1));
        let scaler = factory.CreateBitmapScaler().ok()?;
        scaler.Initialize(&frame, sw, sh, WICBitmapInterpolationModeFant).ok()?;
        let conv = factory.CreateFormatConverter().ok()?;
        conv.Initialize(&scaler, &GUID_WICPixelFormat32bppPBGRA, WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeCustom)
            .ok()?;
        let mut small = vec![0u8; (sw * sh * 4) as usize];
        conv.CopyPixels(std::ptr::null(), sw * 4, &mut small).ok()?;
        let mut out = vec![0u8; (px * px * 4) as usize];
        let (ox, oy) = ((px - sw) / 2, (px - sh) / 2);
        for y in 0..sh {
            let d = (((oy + y) * px + ox) * 4) as usize;
            let s = (y * sw * 4) as usize;
            out[d..d + (sw * 4) as usize].copy_from_slice(&small[s..s + (sw * 4) as usize]);
        }
        Some(Bitmap { w: px, h: px, bgra: out })
    }
}
