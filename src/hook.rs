//! Win32 tarafı (spec §3–4): WH_MOUSE_LL hook thread'i, taskbar tespiti, SendInput, named mutex.

use crate::Shared;
use std::cell::RefCell;
use std::ptr::{null, null_mut};
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::{Sender, SyncSender};
use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{CreateMutexW, GetCurrentThreadId};
use windows_sys::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_VOLUME_DOWN,
    VK_VOLUME_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GA_ROOT, GetAncestor, GetClassNameW, GetMessageW, HC_ACTION, HHOOK, KillTimer,
    LLMHF_INJECTED, MSG, MSLLHOOKSTRUCT, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, SetTimer,
    SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL, WM_APP, WM_MOUSEWHEEL, WM_QUIT, WM_TIMER,
    WindowFromPoint,
};

/// Hook thread'ine: config değişti, rehook zamanlayıcısını yeniden kur.
pub(crate) const WM_APP_RELOAD: u32 = WM_APP + 1;
const MUTEX_NAME: &str = "Local\\LowMonitor.Volume\0";
/// Tek olayda gönderilecek en fazla çentik (aşırı hızlı tekerlekte SendInput dizisini sınırlar).
const MAX_TICKS: usize = 50;

struct Ctx {
    tx: Sender<i16>,
    shared: Arc<Shared>,
}

thread_local! {
    // Hook callback'i yalnız hook'u kuran thread'de çalışır → kilitsiz erişim.
    static CTX: RefCell<Option<Ctx>> = const { RefCell::new(None) };
}

pub(crate) fn is_taskbar(class: &[u16], primary: bool, secondary: bool) -> bool {
    let eq = |s: &str| s.encode_utf16().eq(class.iter().copied());
    (primary && eq("Shell_TrayWnd")) || (secondary && eq("Shell_SecondaryTrayWnd"))
}

impl Ctx {
    /// true → olay yutulur. Yalnız WindowFromPoint + sınıf adı + kanal; blok iş yok.
    fn handle(&self, ms: &MSLLHOOKSTRUCT) -> bool {
        let s = &self.shared;
        if !s.enabled.load(Relaxed) {
            return false;
        }
        if s.ignore_injected.load(Relaxed) && ms.flags & LLMHF_INJECTED != 0 {
            return false;
        }
        // SAFETY: saf Win32 sorguları; geçersiz/NULL HWND'de 0 döner.
        let (root, mut buf) = unsafe { (GetAncestor(WindowFromPoint(ms.pt), GA_ROOT), [0u16; 32]) };
        if root.is_null() {
            return false;
        }
        // SAFETY: buf 32 eleman, uzunluk doğru verildi.
        let n = unsafe { GetClassNameW(root, buf.as_mut_ptr(), buf.len() as i32) };
        let class = &buf[..n.max(0) as usize];
        if !is_taskbar(class, s.primary.load(Relaxed), s.secondary.load(Relaxed)) {
            return false;
        }
        // Yüksek WORD = işaretli delta. Worker ölmüşse olayı geçir.
        self.tx.send((ms.mouseData >> 16) as u16 as i16).is_ok()
    }
}

unsafe extern "system" fn proc(code: i32, wp: usize, lp: isize) -> isize {
    if code == HC_ACTION as i32 && wp == WM_MOUSEWHEEL as usize {
        // SAFETY: WH_MOUSE_LL + HC_ACTION → lp geçerli bir MSLLHOOKSTRUCT'a işaret eder.
        let ms = unsafe { &*(lp as *const MSLLHOOKSTRUCT) };
        let eaten = CTX.with(|c| c.borrow().as_ref().is_some_and(|ctx| ctx.handle(ms)));
        if eaten {
            return 1;
        }
    }
    // SAFETY: zincirdeki sonraki hook'a aynı argümanlar.
    unsafe { CallNextHookEx(null_mut(), code, wp, lp) }
}

fn install() -> HHOOK {
    // SAFETY: proc 'static; LL hook'ta hmod olarak kendi modülümüz yeterli.
    unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(proc), GetModuleHandleW(null()), 0) }
}

/// Eski zamanlayıcıyı durdurur, config'teki aralıkla yenisini kurar (0 = yok).
fn arm_timer(old: usize, shared: &Shared) -> usize {
    // SAFETY: thread zamanlayıcısı (HWND NULL); old=0 ise KillTimer zararsız başarısız olur.
    unsafe {
        if old != 0 {
            KillTimer(null_mut(), old);
        }
        match shared.rehook_ms.load(Relaxed) {
            0 => 0,
            ms => SetTimer(null_mut(), 0, ms, None),
        }
    }
}

/// Hook thread gövdesi. Hazır olunca `ready`'ye thread id (veya kurulum hatası) yazar.
pub(crate) fn run(tx: Sender<i16>, shared: Arc<Shared>, ready: SyncSender<Result<u32, u32>>) {
    // SAFETY: thread-yerel DPI bağlamı; MSLLHOOKSTRUCT.pt fiziksel koordinat → WindowFromPoint uyumlu.
    // PeekMessage thread mesaj kuyruğunu oluşturur (PostThreadMessage için).
    let mut msg: MSG = unsafe { std::mem::zeroed() };
    unsafe {
        SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
    }
    CTX.with(|c| {
        *c.borrow_mut() = Some(Ctx {
            tx,
            shared: shared.clone(),
        })
    });

    let mut hook = install();
    if hook.is_null() {
        // SAFETY: hemen ardından son hata kodu.
        let _ = ready.send(Err(unsafe { GetLastError() }));
        CTX.with(|c| c.borrow_mut().take());
        return;
    }
    // SAFETY: argümansız.
    let _ = ready.send(Ok(unsafe { GetCurrentThreadId() }));
    let mut timer = arm_timer(0, &shared);

    // SAFETY: msg bu thread'in yığınında; GetMessage 0 (WM_QUIT) veya -1 (hata) → çık.
    while unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) } > 0 {
        match msg.message {
            WM_TIMER if msg.wParam == timer => {
                // Windows zaman aşımında hook'u sessizce düşürebilir, tespit API'si yok → periyodik yeniden kur.
                // SAFETY: hook bu thread'de kuruldu; NULL ise Unhook zararsız başarısız olur.
                unsafe { UnhookWindowsHookEx(hook) };
                hook = install(); // başarısızsa bir sonraki tikte tekrar denenir
            }
            WM_APP_RELOAD => timer = arm_timer(timer, &shared),
            _ => {}
        }
    }
    if timer != 0 {
        // SAFETY: bu thread'in zamanlayıcısı.
        unsafe { KillTimer(null_mut(), timer) };
    }
    if !hook.is_null() {
        // SAFETY: bu thread'de kurulan hook.
        unsafe { UnhookWindowsHookEx(hook) };
    }
    CTX.with(|c| c.borrow_mut().take()); // tx düşer → worker çıkar
}

/// Hook thread'ine mesaj (WM_QUIT / WM_APP_RELOAD).
pub(crate) fn post(tid: u32, msg: u32) {
    // SAFETY: tid bu kütüphanenin hook thread'i; thread bittiyse çağrı başarısız olur, zararsız.
    unsafe { PostThreadMessageW(tid, msg, 0, 0) };
}

pub(crate) const QUIT: u32 = WM_QUIT;

/// `ticks` > 0 ses aç, < 0 kıs. Windows yerel flyout'u gösterir; adım %2 (Windows).
pub(crate) fn send_volume(ticks: i32) {
    let n = (ticks.unsigned_abs() as usize).min(MAX_TICKS);
    if n == 0 {
        return;
    }
    let vk = if ticks > 0 {
        VK_VOLUME_UP
    } else {
        VK_VOLUME_DOWN
    };
    let key = |flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let inputs: Vec<INPUT> = (0..n)
        .flat_map(|_| [key(0), key(KEYEVENTF_KEYUP)])
        .collect();
    // SAFETY: inputs geçerli dilim, eleman boyutu doğru.
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
}

/// Süreçler arası tek hook: `Local\LowMonitor.Volume`. Drop → handle kapanır.
pub(crate) struct NamedMutex(HANDLE);
// SAFETY: çekirdek nesne handle'ı thread'e bağlı değil; yalnız Drop'ta kapatılır.
unsafe impl Send for NamedMutex {}
unsafe impl Sync for NamedMutex {}

impl Drop for NamedMutex {
    fn drop(&mut self) {
        // SAFETY: CreateMutexW'den gelen geçerli handle, bir kez kapatılır.
        unsafe { CloseHandle(self.0) };
    }
}

pub(crate) enum MutexError {
    AlreadyRunning,
    Os(u32),
}

pub(crate) fn acquire_mutex() -> Result<NamedMutex, MutexError> {
    let name: Vec<u16> = MUTEX_NAME.encode_utf16().collect();
    // SAFETY: name NUL sonlu; GetLastError hemen ardından okunur.
    unsafe {
        let h = CreateMutexW(null(), 0, name.as_ptr());
        let err = GetLastError();
        if h.is_null() {
            return Err(MutexError::Os(err));
        }
        if err == ERROR_ALREADY_EXISTS {
            CloseHandle(h);
            return Err(MutexError::AlreadyRunning);
        }
        Ok(NamedMutex(h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn taskbar_classes() {
        assert!(is_taskbar(&w("Shell_TrayWnd"), true, false));
        assert!(!is_taskbar(&w("Shell_TrayWnd"), false, true));
        assert!(is_taskbar(&w("Shell_SecondaryTrayWnd"), false, true));
        assert!(!is_taskbar(&w("Shell_SecondaryTrayWnd"), true, false));
        assert!(!is_taskbar(&w("Shell_TrayWndX"), true, true));
        assert!(!is_taskbar(&w("Chrome_WidgetWin_1"), true, true));
        assert!(!is_taskbar(&[], true, true));
    }

    #[test]
    fn mutex_is_exclusive() {
        let a = acquire_mutex().ok().expect("ilk alım");
        assert!(matches!(acquire_mutex(), Err(MutexError::AlreadyRunning)));
        drop(a);
        assert!(acquire_mutex().is_ok());
    }
}
