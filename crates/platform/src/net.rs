use std::path::Path;
use std::time::{Duration, Instant};
use windows::Foundation::Uri;
use windows::Storage::Streams::DataReader;
use windows::Web::Http::HttpClient;
use windows_core::HSTRING;

pub fn finish(status: impl Fn() -> Option<i32>, cancel: impl Fn(), limit: Duration) -> bool {
    let start = Instant::now();
    loop {
        match status() {
            Some(1) => return true,
            Some(0) => {}
            _ => return false,
        }
        if start.elapsed() > limit {
            cancel();
            return false;
        }
        std::thread::sleep(Duration::from_millis(40));
    }
}

fn client(accept: Option<&str>) -> Option<HttpClient> {
    let client = HttpClient::new().ok()?;
    if let Ok(h) = client.DefaultRequestHeaders() {
        if let Ok(ua) = h.UserAgent() {
            let _ = ua.TryParseAdd(&HSTRING::from(concat!("BreezeOS/", env!("CARGO_PKG_VERSION"))));
        }
        if let (Some(a), Ok(acc)) = (accept, h.Accept()) {
            let _ = acc.TryParseAdd(&HSTRING::from(a));
        }
    }
    Some(client)
}

pub fn http_get(url: &str, limit: Duration) -> Option<(i32, String)> {
    http_get_accept(url, None, limit)
}

pub fn http_get_accept(url: &str, accept: Option<&str>, limit: Duration) -> Option<(i32, String)> {
    let client = client(accept)?;
    let op = client.GetAsync(&Uri::CreateUri(&HSTRING::from(url)).ok()?).ok()?;
    if !finish(|| op.Status().ok().map(|s| s.0), || drop(op.Cancel()), limit) {
        return None;
    }
    let resp = op.GetResults().ok()?;
    let status = resp.StatusCode().ok()?.0;
    let read = resp.Content().ok()?.ReadAsStringAsync().ok()?;
    if !finish(|| read.Status().ok().map(|s| s.0), || drop(read.Cancel()), limit) {
        return None;
    }
    Some((status, read.GetResults().ok()?.to_string()))
}

pub fn download(url: &str, to: &Path, limit: Duration, max_bytes: u64) -> Result<u64, String> {
    let client = client(None).ok_or("klien HTTP tidak tersedia")?;
    let uri = Uri::CreateUri(&HSTRING::from(url)).map_err(|e| e.to_string())?;
    let op = client.GetAsync(&uri).map_err(|e| e.to_string())?;
    if !finish(|| op.Status().ok().map(|s| s.0), || drop(op.Cancel()), limit) {
        return Err("waktu habis saat menghubungi server".into());
    }
    let resp = op.GetResults().map_err(|e| e.to_string())?;
    let status = resp.StatusCode().map_err(|e| e.to_string())?.0;
    if status != 200 {
        return Err(format!("HTTP {status}"));
    }
    let read = resp.Content().and_then(|c| c.ReadAsBufferAsync()).map_err(|e| e.to_string())?;
    if !finish(|| read.Status().ok().map(|s| s.0), || drop(read.Cancel()), limit) {
        return Err("waktu habis saat mengunduh".into());
    }
    let buf = read.GetResults().map_err(|e| e.to_string())?;
    let len = buf.Length().map_err(|e| e.to_string())? as u64;
    if len == 0 || len > max_bytes {
        return Err(format!("ukuran berkas tidak wajar ({len} byte)"));
    }
    let reader = DataReader::FromBuffer(&buf).map_err(|e| e.to_string())?;
    let mut bytes = vec![0u8; len as usize];
    reader.ReadBytes(&mut bytes).map_err(|e| e.to_string())?;
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = to.with_extension("part");
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, to).map_err(|e| e.to_string())?;
    Ok(len)
}

pub fn sha256(bytes: &[u8]) -> Option<String> {
    use windows::Win32::Security::Cryptography::{BCryptCloseAlgorithmProvider, BCryptHash, BCryptOpenAlgorithmProvider, BCRYPT_ALG_HANDLE, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS, BCRYPT_SHA256_ALGORITHM};
    unsafe {
        let mut alg = BCRYPT_ALG_HANDLE::default();
        if BCryptOpenAlgorithmProvider(&mut alg, BCRYPT_SHA256_ALGORITHM, None, BCRYPT_OPEN_ALGORITHM_PROVIDER_FLAGS(0)).is_err() {
            return None;
        }
        let mut out = [0u8; 32];
        let ok = BCryptHash(alg, None, bytes, &mut out).is_ok();
        let _ = BCryptCloseAlgorithmProvider(alg, 0);
        ok.then(|| out.iter().map(|b| format!("{b:02x}")).collect())
    }
}

pub fn sha256_file(path: &Path) -> Option<String> {
    sha256(&std::fs::read(path).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vectors() {
        assert_eq!(sha256(b"").unwrap(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(sha256(b"abc").unwrap(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
