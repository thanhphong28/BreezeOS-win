use nacre_core::quick::{self, Op};
use std::path::{Path, PathBuf};
use windows::core::HSTRING;
use windows::Win32::Foundation::GENERIC_WRITE;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_ContainerFormatBmp, GUID_ContainerFormatJpeg, GUID_ContainerFormatPng, GUID_ContainerFormatTiff,
    GUID_WICPixelFormat24bppBGR, GUID_WICPixelFormat32bppBGRA, IWICBitmapFrameEncode, IWICImagingFactory, WICBitmapDitherTypeNone,
    WICBitmapEncoderNoCache, WICBitmapInterpolationModeFant, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
};
use windows::Win32::System::Com::StructuredStorage::IPropertyBag2;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};

fn container(dst: &Path) -> windows_core::GUID {
    match dst.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).as_deref() {
        Some("jpg") | Some("jpeg") => GUID_ContainerFormatJpeg,
        Some("bmp") => GUID_ContainerFormatBmp,
        Some("tif") | Some("tiff") => GUID_ContainerFormatTiff,
        _ => GUID_ContainerFormatPng,
    }
}

fn read_scaled(factory: &IWICImagingFactory, src: &str, scale: f32) -> Option<(u32, u32, Vec<u8>)> {
    unsafe {
        let decoder = factory.CreateDecoderFromFilename(&HSTRING::from(src), None, windows::Win32::Foundation::GENERIC_READ, WICDecodeMetadataCacheOnDemand).ok()?;
        let frame = decoder.GetFrame(0).ok()?;
        let (mut w, mut h) = (0u32, 0u32);
        frame.GetSize(&mut w, &mut h).ok()?;
        if w == 0 || h == 0 {
            return None;
        }
        let (sw, sh) = (((w as f32 * scale).round() as u32).max(1), ((h as f32 * scale).round() as u32).max(1));
        let scaler = factory.CreateBitmapScaler().ok()?;
        scaler.Initialize(&frame, sw, sh, WICBitmapInterpolationModeFant).ok()?;
        let conv = factory.CreateFormatConverter().ok()?;
        conv.Initialize(&scaler, &GUID_WICPixelFormat32bppBGRA, WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeCustom).ok()?;
        let mut buf = vec![0u8; (sw * sh * 4) as usize];
        conv.CopyPixels(std::ptr::null(), sw * 4, &mut buf).ok()?;
        Some((sw, sh, buf))
    }
}

fn on_white(bgra: &[u8], w: u32, h: u32) -> Vec<u8> {
    let stride = (w * 3).div_ceil(4) * 4;
    let mut out = vec![0u8; (stride * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let s = ((y * w + x) * 4) as usize;
            let d = (y * stride + x * 3) as usize;
            let a = bgra[s + 3] as u32;
            for c in 0..3 {
                out[d + c] = ((bgra[s + c] as u32 * a + 255 * (255 - a)) / 255) as u8;
            }
        }
    }
    out
}

fn write(factory: &IWICImagingFactory, dst: &Path, w: u32, h: u32, bgra: &[u8]) -> Result<(), String> {
    let fmt = container(dst);
    let jpeg = fmt == GUID_ContainerFormatJpeg;
    unsafe {
        let bitmap = if jpeg {
            let px = on_white(bgra, w, h);
            factory.CreateBitmapFromMemory(w, h, &GUID_WICPixelFormat24bppBGR, (w * 3).div_ceil(4) * 4, &px)
        } else {
            factory.CreateBitmapFromMemory(w, h, &GUID_WICPixelFormat32bppBGRA, w * 4, bgra)
        }
        .map_err(|e| e.to_string())?;
        let stream = factory.CreateStream().map_err(|e| e.to_string())?;
        stream.InitializeFromFilename(&HSTRING::from(dst.as_os_str()), GENERIC_WRITE.0).map_err(|e| e.to_string())?;
        let encoder = factory.CreateEncoder(&fmt, std::ptr::null()).map_err(|e| e.to_string())?;
        encoder.Initialize(&stream, WICBitmapEncoderNoCache).map_err(|e| e.to_string())?;
        let mut frame: Option<IWICBitmapFrameEncode> = None;
        let mut props: Option<IPropertyBag2> = None;
        encoder.CreateNewFrame(&mut frame, &mut props).map_err(|e| e.to_string())?;
        let frame = frame.ok_or("frame")?;
        frame.Initialize(props.as_ref()).map_err(|e| e.to_string())?;
        frame.SetSize(w, h).map_err(|e| e.to_string())?;
        let mut pf = if jpeg { GUID_WICPixelFormat24bppBGR } else { GUID_WICPixelFormat32bppBGRA };
        frame.SetPixelFormat(&mut pf).map_err(|e| e.to_string())?;
        frame.WriteSource(&bitmap, std::ptr::null()).map_err(|e| e.to_string())?;
        frame.Commit().map_err(|e| e.to_string())?;
        encoder.Commit().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn convert(src: &str, dst: &Path, scale: f32) -> Result<(), String> {
    unsafe {
        let com = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let r = (|| {
            let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
            let (w, h, px) = read_scaled(&factory, src, scale).ok_or("gambar tidak bisa dibaca")?;
            write(&factory, dst, w, h, &px)
        })();
        if r.is_err() {
            let _ = std::fs::remove_file(dst);
        }
        if com {
            CoUninitialize();
        }
        r
    }
}

fn tar_exe() -> Option<PathBuf> {
    let root = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("C:\\Windows"));
    let p = root.join("System32").join("tar.exe");
    p.exists().then_some(p)
}

pub fn zip(paths: &[String], dst: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let tar = tar_exe().ok_or("tar.exe tidak ditemukan")?;
    let mut cmd = std::process::Command::new(tar);
    cmd.arg("-a").arg("-c").arg("-f").arg(dst);
    for p in paths {
        let path = Path::new(p);
        let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else { continue };
        cmd.arg("-C").arg(dir).arg(name);
    }
    let out = cmd.creation_flags(0x0800_0000).output().map_err(|e| e.to_string())?;
    if out.status.success() && dst.exists() {
        Ok(())
    } else {
        let _ = std::fs::remove_file(dst);
        Err(String::from_utf8_lossy(&out.stderr).trim().chars().take(200).collect())
    }
}

pub fn run(op: Op, paths: &[String], small: &str, archive: &str) -> Result<Vec<PathBuf>, String> {
    let exists = |p: &Path| p.exists();
    match op {
        Op::Zip => {
            let dst = quick::zip_target(paths, archive, &exists).ok_or("tidak ada berkas")?;
            zip(paths, &dst)?;
            Ok(vec![dst])
        }
        _ => {
            let mut done = Vec::new();
            let mut last_err = String::new();
            for p in paths.iter().filter(|p| quick::is_image(p)) {
                let dst = quick::output_for(p, op, small, &exists);
                let scale = if op == Op::Half { 0.5 } else { 1.0 };
                match convert(p, &dst, scale) {
                    Ok(()) => done.push(dst),
                    Err(e) => last_err = e,
                }
            }
            if done.is_empty() {
                Err(if last_err.is_empty() { "tidak ada gambar".into() } else { last_err })
            } else {
                Ok(done)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("breezeos-quick-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        d
    }

    #[test]
    fn white_background_blends_transparency() {
        let px = on_white(&[0, 0, 0, 0, 0, 0, 255, 255], 2, 1);
        assert_eq!(&px[..6], &[255, 255, 255, 0, 0, 255]);
    }

    #[test]
    fn image_round_trip_halves_and_converts() {
        let d = temp_dir("img");
        let src = d.join("asal.png");
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let f: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).unwrap();
            let px: Vec<u8> = (0..40 * 20).flat_map(|i| [(i % 255) as u8, 80, 160, 255]).collect();
            write(&f, &src, 40, 20, &px).unwrap();
        }
        let s = src.to_string_lossy().to_string();
        let out = run(Op::Half, std::slice::from_ref(&s), "kecil", "Arsip").unwrap();
        let b = crate::wic::decode_file_fit(&out[0].to_string_lossy(), 64).unwrap();
        assert_eq!(b.w, 64);
        assert!(out[0].to_string_lossy().ends_with("asal (kecil).png"));
        let jpg = run(Op::Jpg, std::slice::from_ref(&s), "kecil", "Arsip").unwrap();
        assert!(jpg[0].exists() && jpg[0].extension().unwrap() == "jpg");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn zip_packs_files_next_to_them() {
        let d = temp_dir("zip");
        let a = d.join("a.txt");
        let b = d.join("b.txt");
        std::fs::write(&a, "halo").unwrap();
        std::fs::write(&b, "dunia").unwrap();
        let list = vec![a.to_string_lossy().to_string(), b.to_string_lossy().to_string()];
        let out = run(Op::Zip, &list, "kecil", "Arsip").unwrap();
        assert!(out[0].ends_with("Arsip.zip"));
        assert!(std::fs::metadata(&out[0]).unwrap().len() > 0);
        assert!(run(Op::Half, &list, "kecil", "Arsip").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }
}
