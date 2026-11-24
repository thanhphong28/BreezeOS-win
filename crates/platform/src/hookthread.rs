use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{GetCurrentThreadId, SetThreadPriority, GetCurrentThread, THREAD_PRIORITY_ABOVE_NORMAL};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, HOOKPROC, MSG, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WM_QUIT,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Mouse,
    Keyboard,
}

pub struct HookThread {
    tid: u32,
    join: Option<JoinHandle<()>>,
}

impl HookThread {
    pub fn start(kind: Kind, proc: HOOKPROC) -> Option<HookThread> {
        let (tx, rx) = channel::<Option<u32>>();
        let join = std::thread::Builder::new()
            .name("breezeos-hook".into())
            .spawn(move || unsafe {
                let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL);
                let id = match kind {
                    Kind::Mouse => WH_MOUSE_LL,
                    Kind::Keyboard => WH_KEYBOARD_LL,
                };
                let hi = GetModuleHandleW(None).ok().map(|m| windows::Win32::Foundation::HINSTANCE(m.0));
                let hook = match SetWindowsHookExW(id, proc, hi, 0) {
                    Ok(h) => h,
                    Err(_) => {
                        let _ = tx.send(None);
                        return;
                    }
                };
                let _ = tx.send(Some(GetCurrentThreadId()));
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                let _ = UnhookWindowsHookEx(hook);
            })
            .ok()?;
        let tid = rx.recv().ok().flatten()?;
        Some(HookThread { tid, join: Some(join) })
    }
}

impl Drop for HookThread {
    fn drop(&mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.tid, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::UI::WindowsAndMessaging::CallNextHookEx;

    static SEEN_THREAD: AtomicU32 = AtomicU32::new(0);

    unsafe extern "system" fn probe(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        SEEN_THREAD.store(GetCurrentThreadId(), Ordering::Relaxed);
        CallNextHookEx(None, code, wp, lp)
    }

    #[test]
    fn hook_runs_on_its_own_thread_and_stops_cleanly() {
        let h = HookThread::start(Kind::Mouse, Some(probe)).expect("hook terpasang");
        assert_ne!(h.tid, unsafe { GetCurrentThreadId() });
        drop(h);
    }
}
