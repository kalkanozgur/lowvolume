#![windows_subsystem = "windows"]
//! Standalone `lowvolume.exe` (spec §5): lib + gizli pencere + tray. lowmon-tray çalışıyorsa sessiz çıkar.

use lowvolume::Config;
use std::cell::RefCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{ERROR_SUCCESS, HWND, POINT};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
use windows_sys::Win32::UI::Shell::{
    ExtractIconExW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW, Shell_NotifyIconW, ShellExecuteW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, GetMessageW, HICON, IDI_APPLICATION, LoadIconW, MF_CHECKED, MF_SEPARATOR,
    MF_STRING, MSG, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW,
    SW_SHOWNORMAL, SetForegroundWindow, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu,
    TranslateMessage, WM_APP, WM_CLOSE, WM_DESTROY, WM_LBUTTONDBLCLK, WM_RBUTTONUP, WNDCLASSW,
    WS_POPUP,
};

const WM_TRAY: u32 = WM_APP + 10;
const ID_ENABLED: usize = 1;
const ID_OPEN: usize = 2;
const ID_RELOAD: usize = 3;
const ID_AUTOSTART: usize = 4;
const ID_EXIT: usize = 5;
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "lowvolume";
const LOG_MAX: u64 = 64 * 1024;

/// "Config'i aç" dosya yoksa bunu yazar. Varsayılanlarla birebir (test).
const TEMPLATE: &str = r#"# lowvolume — taskbar üstünde tekerlek → sistem sesi. Değişince tray > "Config'i yeniden yükle".

[general]
enabled = true

[volume]
invert = false           # tekerlek yönünü ters çevir
ignore_injected = true   # yazılımın ürettiği tekerlek olaylarını yok say (Logitech/RDP sorununda false)

[taskbar]
primary = true           # ana monitör taskbar'ı
secondary = true         # diğer monitörlerin taskbar'ı

[advanced]
rehook_interval_s = 60   # hook'u periyodik yeniden kur (0 = kapalı)
"#;

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

/// `%LOCALAPPDATA%\LowMonitor\lowvolume.log`, LOG_MAX'ı aşınca baştan başlar.
fn log(msg: &str) {
    let dir =
        PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("LowMonitor");
    let path = dir.join("lowvolume.log");
    let _ = std::fs::create_dir_all(&dir);
    let truncate = std::fs::metadata(&path).is_ok_and(|m| m.len() > LOG_MAX);
    let f = std::fs::OpenOptions::new()
        .create(true)
        .append(!truncate)
        .write(true)
        .truncate(truncate)
        .open(&path);
    if let Ok(mut f) = f {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let _ = writeln!(f, "{t} {}", msg.replace(['\r', '\n'], " "));
    }
}

fn load_config(path: &Path) -> Config {
    let (cfg, err) = Config::load(path);
    if let Some(e) = err {
        log(&format!("config hatası, varsayılanlar kullanılıyor: {e}"));
    }
    cfg
}

fn autostart_enabled() -> bool {
    let (k, v) = (w(RUN_KEY), w(RUN_VALUE));
    // SAFETY: NUL sonlu dizgiler; veri istenmiyor, yalnız varlık.
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            null_mut(),
            null_mut(),
        ) == ERROR_SUCCESS
    }
}

fn set_autostart(on: bool) {
    let (k, v) = (w(RUN_KEY), w(RUN_VALUE));
    let rc = if on {
        let Ok(exe) = std::env::current_exe() else {
            return;
        };
        let data = w(&format!("\"{}\"", exe.display()));
        // SAFETY: data NUL dahil UTF-16, boyut bayt cinsinden.
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                k.as_ptr(),
                v.as_ptr(),
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            )
        }
    } else {
        // SAFETY: NUL sonlu dizgiler.
        unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, k.as_ptr(), v.as_ptr()) }
    };
    if rc != ERROR_SUCCESS {
        log(&format!("autostart {on}: kayıt defteri hatası {rc}"));
    }
}

fn open_config(path: &Path) {
    if !path.exists() {
        let ok = path
            .parent()
            .is_some_and(|d| std::fs::create_dir_all(d).is_ok())
            && std::fs::write(path, TEMPLATE).is_ok();
        if !ok {
            log(&format!("config yazılamadı: {}", path.display()));
            return;
        }
    }
    let (op, file) = (w("open"), w(&path.to_string_lossy()));
    // SAFETY: NUL sonlu dizgiler; dönüş > 32 başarı.
    let r = unsafe {
        ShellExecuteW(
            null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL,
        )
    };
    if (r as isize) <= 32 {
        // .toml ilişkilendirilmemiş olabilir.
        let np = w("notepad.exe");
        // SAFETY: aynı.
        unsafe {
            ShellExecuteW(
                null_mut(),
                op.as_ptr(),
                np.as_ptr(),
                file.as_ptr(),
                null(),
                SW_SHOWNORMAL,
            )
        };
    }
}

struct App {
    handle: lowvolume::Handle,
    cfg_path: PathBuf,
    nid: NOTIFYICONDATAW,
    taskbar_created: u32,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

impl App {
    fn set_tip(&mut self) {
        let tip = if self.handle.enabled() {
            "lowvolume"
        } else {
            "lowvolume (devre dışı)"
        };
        let t: Vec<u16> = tip.encode_utf16().collect();
        self.nid.szTip = [0; 128];
        self.nid.szTip[..t.len()].copy_from_slice(&t);
    }

    fn tray(&mut self, op: u32) {
        self.set_tip();
        // SAFETY: nid geçerli, hWnd canlı.
        unsafe { Shell_NotifyIconW(op, &self.nid) };
    }

    fn toggle(&mut self) {
        self.handle.set_enabled(!self.handle.enabled());
        self.tray(NIM_MODIFY);
    }

    fn menu(&mut self, hwnd: HWND) {
        let item = |checked: bool| MF_STRING | if checked { MF_CHECKED } else { 0 };
        let items: [(u32, usize, &str); 6] = [
            (item(self.handle.enabled()), ID_ENABLED, "Etkin"),
            (MF_STRING, ID_OPEN, "Config'i aç"),
            (MF_STRING, ID_RELOAD, "Config'i yeniden yükle"),
            (
                item(autostart_enabled()),
                ID_AUTOSTART,
                "Başlangıçta çalıştır",
            ),
            (MF_SEPARATOR, 0, ""),
            (MF_STRING, ID_EXIT, "Çıkış"),
        ];
        let mut pt = POINT { x: 0, y: 0 };
        // SAFETY: menü bu fonksiyonda oluşturulup yok edilir; SetForegroundWindow menünün dışarı tıklayınca kapanması için.
        let cmd = unsafe {
            let m = CreatePopupMenu();
            for (flags, id, text) in items {
                let t = w(text);
                AppendMenuW(m, flags, id, t.as_ptr());
            }
            GetCursorPos(&mut pt);
            SetForegroundWindow(hwnd);
            let cmd = TrackPopupMenu(
                m,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                pt.x,
                pt.y,
                0,
                hwnd,
                null(),
            );
            DestroyMenu(m);
            cmd as usize
        };
        match cmd {
            ID_ENABLED => self.toggle(),
            ID_OPEN => open_config(&self.cfg_path),
            ID_RELOAD => {
                self.handle.reload(&load_config(&self.cfg_path));
                self.tray(NIM_MODIFY);
            }
            ID_AUTOSTART => set_autostart(!autostart_enabled()),
            // Post: WM_DESTROY bu borrow'un dışında işlensin (aksi hâlde RefCell yeniden giriş panic'i).
            // SAFETY: kendi penceremiz.
            ID_EXIT => unsafe {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            },
            _ => {}
        }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: usize, lp: isize) -> isize {
    let handled = APP.with(|a| {
        // Menü açıkken gelen mesajlarda yeniden giriş olabilir → try_borrow.
        let Ok(mut a) = a.try_borrow_mut() else {
            return false;
        };
        let Some(app) = a.as_mut() else { return false };
        match msg {
            WM_TRAY => match lp as u32 {
                WM_RBUTTONUP => app.menu(hwnd),
                WM_LBUTTONDBLCLK => app.toggle(),
                _ => {}
            },
            // explorer yeniden başladı → ikon yeniden eklenir.
            m if m == app.taskbar_created => app.tray(NIM_ADD),
            _ => return false,
        }
        true
    });
    if handled {
        return 0;
    }
    if msg == WM_DESTROY {
        if let Some(mut app) = APP.with(|a| a.borrow_mut().take()) {
            app.tray(NIM_DELETE);
        } // Handle düşer → hook kaldırılır
        // SAFETY: argümansız.
        unsafe { PostQuitMessage(0) };
        return 0;
    }
    // SAFETY: işlenmeyen mesajlar varsayılana.
    unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
}

fn load_icon() -> HICON {
    let mut small: HICON = null_mut();
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| String::from(r"C:\Windows"));
    let path = w(&format!(r"{root}\System32\SndVol.exe"));
    // SAFETY: path NUL sonlu; tek küçük ikon istenir.
    unsafe {
        ExtractIconExW(path.as_ptr(), 0, null_mut(), &mut small, 1);
        if small.is_null() {
            small = LoadIconW(null_mut(), IDI_APPLICATION);
        }
    }
    small
}

fn main() {
    let cfg_path = Config::path();
    let cfg = load_config(&cfg_path);
    let handle = match lowvolume::start(&cfg) {
        Ok(h) => h,
        Err(lowvolume::Error::AlreadyRunning) => return, // lowmon-tray veya başka örnek
        Err(e) => {
            log(&format!("başlatılamadı: {e}"));
            std::process::exit(1);
        }
    };

    let class = w("LowVolumeTray");
    // SAFETY: sınıf/pencere bu thread'de; WS_POPUP, gösterilmez. HWND_MESSAGE değil: TaskbarCreated yayını gelsin.
    let hwnd = unsafe {
        let hinst = GetModuleHandleW(null());
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinst,
            lpszClassName: class.as_ptr(),
            ..std::mem::zeroed()
        };
        RegisterClassW(&wc);
        CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        )
    };
    if hwnd.is_null() {
        log("tray penceresi oluşturulamadı");
        std::process::exit(1);
    }

    // SAFETY: NOTIFYICONDATAW düz C yapısı, sıfır geçerli başlangıç.
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAY;
    nid.hIcon = load_icon();
    let tc = w("TaskbarCreated");
    // SAFETY: NUL sonlu dizgi.
    let taskbar_created = unsafe { RegisterWindowMessageW(tc.as_ptr()) };

    let mut app = App {
        handle,
        cfg_path,
        nid,
        taskbar_created,
    };
    app.tray(NIM_ADD);
    APP.with(|a| *a.borrow_mut() = Some(app));
    // Başlangıçta dokunulan (shell32/ikon yükleme) sayfaları bırak; idle'da geri gelmezler.
    // SAFETY: kendi süreç handle'ımız; (MAX, MAX) = çalışma kümesini kırp.
    unsafe { SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX) };

    // SAFETY: standart mesaj döngüsü.
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_equals_defaults() {
        let (c, err) = Config::parse(TEMPLATE);
        assert!(err.is_none(), "{err:?}");
        assert_eq!(c, Config::default());
    }
}
