use std::sync::mpsc::{channel, Receiver, Sender};
use windows::ApplicationModel::Appointments::{AppointmentManager, AppointmentProperties, AppointmentStore, AppointmentStoreAccessType, FindAppointmentsOptions};
use windows::Foundation::{DateTime, TimeSpan};
use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::System::Time::{FileTimeToSystemTime, SystemTimeToFileTime, SystemTimeToTzSpecificLocalTime, TzSpecificLocalTimeToSystemTime};

pub type Date = (i32, u32, u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub title: String,
    pub minute: Option<u32>,
    pub minutes: u32,
    pub link: Option<String>,
}

pub struct Agenda {
    pub requests: Sender<Date>,
    pub answers: Receiver<(Date, Option<Vec<Event>>)>,
}

fn local_midnight(d: Date) -> Option<DateTime> {
    unsafe {
        let local = SYSTEMTIME { wYear: d.0 as u16, wMonth: d.1 as u16, wDay: d.2 as u16, ..Default::default() };
        let mut utc = SYSTEMTIME::default();
        TzSpecificLocalTimeToSystemTime(None, &local, &mut utc).ok()?;
        let mut ft = FILETIME::default();
        SystemTimeToFileTime(&utc, &mut ft).ok()?;
        Some(DateTime { UniversalTime: ((ft.dwHighDateTime as i64) << 32) | ft.dwLowDateTime as i64 })
    }
}

fn to_local(t: DateTime) -> Option<(Date, u32)> {
    unsafe {
        let v = t.UniversalTime as u64;
        let ft = FILETIME { dwLowDateTime: v as u32, dwHighDateTime: (v >> 32) as u32 };
        let mut utc = SYSTEMTIME::default();
        FileTimeToSystemTime(&ft, &mut utc).ok()?;
        let mut local = SYSTEMTIME::default();
        SystemTimeToTzSpecificLocalTime(None, &utc, &mut local).ok()?;
        Some(((local.wYear as i32, local.wMonth as u32, local.wDay as u32), local.wHour as u32 * 60 + local.wMinute as u32))
    }
}

fn read(store: &AppointmentStore, d: Date) -> Option<Vec<Event>> {
    let start = local_midnight(d)?;
    let opts = FindAppointmentsOptions::new().ok()?;
    if let Ok(props) = opts.FetchProperties() {
        for p in [
            AppointmentProperties::Subject(),
            AppointmentProperties::StartTime(),
            AppointmentProperties::AllDay(),
            AppointmentProperties::Duration(),
            AppointmentProperties::Location(),
            AppointmentProperties::Details(),
            AppointmentProperties::OnlineMeetingLink(),
        ]
        .into_iter()
        .flatten()
        {
            let _ = props.Append(&p);
        }
    }
    let _ = opts.SetMaxCount(20);
    let day = TimeSpan { Duration: 24 * 3600 * 10_000_000 };
    let list = store.FindAppointmentsAsyncWithOptions(start, day, &opts).ok()?.join().ok()?;
    let mut out = Vec::new();
    for a in list {
        let title = a.Subject().map(|s| s.to_string()).unwrap_or_default();
        let all_day = a.AllDay().unwrap_or(false);
        let minute = match a.StartTime().ok().and_then(to_local) {
            Some((date, m)) if date == d && !all_day => Some(m),
            _ => None,
        };
        let minutes = a.Duration().map(|d| (d.Duration / 600_000_000) as u32).unwrap_or(0);
        let link = [a.OnlineMeetingLink().ok(), a.Location().ok(), a.Details().ok()]
            .into_iter()
            .flatten()
            .find_map(|t| nacre_core::quick::find_link(&t.to_string()));
        out.push(Event { title, minute, minutes, link });
    }
    out.sort_by_key(|e| e.minute.map_or(0, |m| m + 1));
    Some(out)
}

fn fake(d: Date, today: Date) -> Option<Vec<Event>> {
    let soon = std::env::var("NACRE_FAKE_MEETING").ok().and_then(|v| v.parse::<i32>().ok()).map(|m| {
        let t = crate::time::local_now();
        ((t.hour as i32 * 60 + t.minute as i32 + m).clamp(0, 24 * 60 - 1)) as u32
    });
    let mut list = vec![
        Event { title: "Rapat Uji".into(), minute: Some(14 * 60 + 30), minutes: 30, link: None },
        Event { title: "Olahraga Uji".into(), minute: Some(18 * 60), minutes: 60, link: None },
    ];
    if let Some(m) = soon {
        list.push(Event { title: "Standup tim".into(), minute: Some(m), minutes: 30, link: Some("https://meet.example.com/abc-defg-hij".into()) });
    }
    Some(if d == today { list } else { vec![] })
}

pub fn spawn(notify: impl Fn() + Send + 'static) -> Agenda {
    let (req_tx, req_rx) = channel::<Date>();
    let (ans_tx, ans_rx) = channel::<(Date, Option<Vec<Event>>)>();
    let fake_mode = std::env::var_os("NACRE_FAKE_CAL").is_some();
    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let store = if fake_mode { None } else { AppointmentManager::RequestStoreAsync(AppointmentStoreAccessType::AllCalendarsReadOnly).ok().and_then(|o| o.join().ok()) };
        if !fake_mode {
            crate::log::line("kalender", if store.is_some() { "kalender Windows terbaca" } else { "kalender Windows tidak tersedia" });
        }
        while let Ok(mut d) = req_rx.recv() {
            while let Ok(n) = req_rx.try_recv() {
                d = n;
            }
            let ans = if fake_mode {
                let t = crate::time::local_now();
                fake(d, (t.year, t.month, t.day))
            } else {
                store.as_ref().and_then(|s| read(s, d))
            };
            if ans_tx.send((d, ans)).is_err() {
                return;
            }
            notify();
        }
    });
    Agenda { requests: req_tx, answers: ans_rx }
}
