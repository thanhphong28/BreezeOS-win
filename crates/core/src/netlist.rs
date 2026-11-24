#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Network {
    pub ssid: String,
    pub signal: u8,
    pub secured: bool,
    pub connected: bool,
    pub profile: Option<String>,
}

impl Network {
    pub fn known(&self) -> bool {
        self.profile.is_some()
    }
}

pub fn merge(raw: Vec<Network>) -> Vec<Network> {
    let mut out: Vec<Network> = Vec::new();
    for n in raw {
        if n.ssid.trim().is_empty() {
            continue;
        }
        match out.iter_mut().find(|o| o.ssid == n.ssid) {
            Some(o) => {
                o.signal = o.signal.max(n.signal);
                o.secured |= n.secured;
                o.connected |= n.connected;
                if o.profile.is_none() {
                    o.profile = n.profile;
                }
            }
            None => out.push(n),
        }
    }
    out.sort_by(|a, b| b.connected.cmp(&a.connected).then(b.known().cmp(&a.known())).then(b.signal.cmp(&a.signal)).then(a.ssid.cmp(&b.ssid)));
    out
}

pub fn bars(signal: u8) -> u8 {
    match signal {
        0..=39 => 1,
        40..=69 => 2,
        _ => 3,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DevKind {
    Headphones,
    Keyboard,
    Mouse,
    #[default]
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Device {
    pub name: String,
    pub connected: bool,
    pub kind: DevKind,
    pub addr: u64,
    pub audio: bool,
}

pub fn bt_addr_from_path(path: &str) -> Option<u64> {
    let l = path.to_ascii_lowercase();
    if !l.contains("bthenum") {
        return None;
    }
    let end = l.find("_c00000000")?;
    let hex = l.get(end.checked_sub(12)?..end)?;
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u64::from_str_radix(hex, 16).ok().filter(|a| *a != 0)
}

pub fn kind_from_cod(cod: u32) -> DevKind {
    let major = (cod >> 8) & 0x1F;
    let minor = (cod >> 2) & 0x3F;
    match major {
        4 => DevKind::Headphones,
        5 if minor & 0x10 != 0 => DevKind::Keyboard,
        5 if minor & 0x20 != 0 => DevKind::Mouse,
        _ => DevKind::Other,
    }
}

pub fn kind_from_appearance(raw: u16) -> DevKind {
    let (cat, sub) = (raw >> 6, raw & 0x3F);
    match cat {
        0x21 | 0x25 => DevKind::Headphones,
        0x0F if sub == 1 => DevKind::Keyboard,
        0x0F if sub == 2 => DevKind::Mouse,
        _ => DevKind::Other,
    }
}

pub fn merge_devices(raw: Vec<Device>) -> Vec<Device> {
    let mut out: Vec<Device> = Vec::new();
    for d in raw {
        if d.name.trim().is_empty() {
            continue;
        }
        match out.iter_mut().find(|o| o.name == d.name) {
            Some(o) => {
                o.connected |= d.connected;
                if o.kind == DevKind::Other {
                    o.kind = d.kind;
                }
                if o.addr == 0 || (d.audio && !o.audio) {
                    o.addr = d.addr;
                }
                o.audio |= d.audio;
            }
            None => out.push(d),
        }
    }
    out.sort_by(|a, b| b.connected.cmp(&a.connected).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net(ssid: &str, signal: u8, connected: bool, profile: Option<&str>) -> Network {
        Network { ssid: ssid.into(), signal, secured: true, connected, profile: profile.map(Into::into) }
    }

    #[test]
    fn merge_joins_duplicates_and_sorts() {
        let raw = vec![net("Cafe", 80, false, None), net("Rumah", 40, false, None), net("Rumah", 45, true, Some("Rumah")), net("", 99, false, None), net("Kantor", 60, false, Some("Kantor"))];
        let m = merge(raw);
        let names: Vec<&str> = m.iter().map(|n| n.ssid.as_str()).collect();
        assert_eq!(names, ["Rumah", "Kantor", "Cafe"]);
        assert_eq!(m[0].signal, 45);
        assert!(m[0].connected && m[0].known());
    }

    #[test]
    fn bars_thresholds() {
        assert_eq!(bars(0), 1);
        assert_eq!(bars(39), 1);
        assert_eq!(bars(40), 2);
        assert_eq!(bars(70), 3);
        assert_eq!(bars(100), 3);
    }

    #[test]
    fn device_kinds() {
        assert_eq!(kind_from_cod(0x240404), DevKind::Headphones);
        assert_eq!(kind_from_cod(0x002540), DevKind::Keyboard);
        assert_eq!(kind_from_cod(0x002580), DevKind::Mouse);
        assert_eq!(kind_from_cod(0x5A020C), DevKind::Other);
        assert_eq!(kind_from_appearance(0x0941), DevKind::Headphones);
        assert_eq!(kind_from_appearance(0x03C1), DevKind::Keyboard);
        assert_eq!(kind_from_appearance(0x03C2), DevKind::Mouse);
        assert_eq!(kind_from_appearance(0), DevKind::Other);
    }

    #[test]
    fn devices_merge_by_name() {
        let d = |n: &str, c: bool, k: DevKind| Device { name: n.into(), connected: c, kind: k, ..Default::default() };
        let m = merge_devices(vec![d("buds", false, DevKind::Other), d("Mouse", false, DevKind::Mouse), d("buds", true, DevKind::Headphones), d(" ", true, DevKind::Other)]);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0], d("buds", true, DevKind::Headphones));
        assert_eq!(m[1].name, "Mouse");
        let le = Device { addr: 0x1111, ..d("buds", false, DevKind::Headphones) };
        let classic = Device { addr: 0x2222, audio: true, ..d("buds", false, DevKind::Headphones) };
        let m = merge_devices(vec![le, classic]);
        assert_eq!((m[0].addr, m[0].audio), (0x2222, true));
    }

    #[test]
    fn address_from_filter_path() {
        let p = r"{2}.\\?\bthenum#{0000110b-0000-1000-8000-00805f9b34fb}_vid&0001004c_pid&200e_ver&0300#8&2f1c8c4d&0&9C3E53ABCDEF_C00000000#{6994ad04-93ef-11d0-a3cc-00a0c9223196}\wavespeaker";
        assert_eq!(bt_addr_from_path(p), Some(0x9C3E53ABCDEF));
        assert_eq!(bt_addr_from_path(r"{2}.\?\bthhfenum#bthhfpaudio#8&1234&0&97#{6994ad04}"), None, "hands-free tanpa alamat");
        assert_eq!(bt_addr_from_path(r"{2}.\?\hdaudio#func_01&ven_10ec#4&0&0001#{6994ad04}\speaker"), None);
        assert_eq!(bt_addr_from_path("bthenum#x_C00000000"), None, "terlalu pendek");
    }
}
