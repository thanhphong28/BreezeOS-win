const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;

fn res_entry(out: &mut Vec<u8>, typ: u16, name: u16, flags: u16, data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&32u32.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&typ.to_le_bytes());
    out.extend_from_slice(&0xFFFFu16.to_le_bytes());
    out.extend_from_slice(&name.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&0x0409u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(data);
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

fn ico_to_res(ico: &[u8], group: u16) -> Vec<u8> {
    let u16_at = |o: usize| u16::from_le_bytes([ico[o], ico[o + 1]]);
    let u32_at = |o: usize| u32::from_le_bytes([ico[o], ico[o + 1], ico[o + 2], ico[o + 3]]);
    assert!(ico.len() > 6 && u16_at(2) == 1, "assets/breezeos.ico bukan berkas ikon");
    let count = u16_at(4) as usize;
    let mut res = Vec::new();
    res_entry(&mut res, 0, 0, 0, &[]);
    let mut dir = Vec::new();
    dir.extend_from_slice(&0u16.to_le_bytes());
    dir.extend_from_slice(&1u16.to_le_bytes());
    dir.extend_from_slice(&(count as u16).to_le_bytes());
    for i in 0..count {
        let e = 6 + 16 * i;
        let (size, offset) = (u32_at(e + 8) as usize, u32_at(e + 12) as usize);
        let id = (i + 1) as u16;
        res_entry(&mut res, RT_ICON, id, 0x1010, &ico[offset..offset + size]);
        let planes = u16_at(e + 4).max(1);
        let bits = if u16_at(e + 6) == 0 { 32 } else { u16_at(e + 6) };
        dir.extend_from_slice(&ico[e..e + 4]);
        dir.extend_from_slice(&planes.to_le_bytes());
        dir.extend_from_slice(&bits.to_le_bytes());
        dir.extend_from_slice(&(size as u32).to_le_bytes());
        dir.extend_from_slice(&id.to_le_bytes());
    }
    res_entry(&mut res, RT_GROUP_ICON, group, 0x1030, &dir);
    res
}

fn embed_icon() {
    let ico = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/breezeos.ico");
    println!("cargo:rerun-if-changed={}", ico.display());
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let data = std::fs::read(&ico).expect("assets/breezeos.ico tidak terbaca");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("breezeos_icon.res");
    std::fs::write(&out, ico_to_res(&data, 1)).expect("gagal menulis breezeos_icon.res");
    println!("cargo:rustc-link-arg-bins={}", out.display());
}
