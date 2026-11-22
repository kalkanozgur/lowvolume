//! lowvolume: taskbar üstünde mouse tekerleği → sistem sesi (docs/spec.md).
//! Standalone bin ve lowmon-tray (süreç içi, git bağımlılığı) aynı API'yi kullanır:
//!
//! ```no_run
//! let (cfg, _err) = lowvolume::Config::load(&lowvolume::Config::path());
//! let handle = lowvolume::start(&cfg)?; // Drop → hook kaldırılır, thread'ler biter
//! # Ok::<(), lowvolume::Error>(())
//! ```

mod config;
mod hook;
mod wheel;

pub use config::{Advanced, Config, General, Taskbar, Volume};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

#[derive(Debug)]
pub enum Error {
    /// Başka bir süreç (standalone veya lowmon-tray) hook'u zaten kurmuş.
    AlreadyRunning,
    /// Win32 hata kodu (mutex veya SetWindowsHookExW).
    Os(u32),
    Spawn(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyRunning => f.write_str("lowvolume zaten çalışıyor"),
            Error::Os(c) => write!(f, "Win32 hatası {c}"),
            Error::Spawn(e) => write!(f, "thread başlatılamadı: {e}"),
        }
    }
}
impl std::error::Error for Error {}

/// Hook ve worker thread'lerinin okuduğu canlı ayarlar.
pub(crate) struct Shared {
    enabled: AtomicBool,
    invert: AtomicBool,
    ignore_injected: AtomicBool,
    primary: AtomicBool,
    secondary: AtomicBool,
    rehook_ms: AtomicU32,
}

impl Shared {
    fn new(c: &Config) -> Self {
        let s = Self {
            enabled: AtomicBool::new(false),
            invert: AtomicBool::new(false),
            ignore_injected: AtomicBool::new(false),
            primary: AtomicBool::new(false),
            secondary: AtomicBool::new(false),
            rehook_ms: AtomicU32::new(0),
        };
        s.apply(c);
        s
    }

    fn apply(&self, c: &Config) {
        self.enabled.store(c.general.enabled, Relaxed);
        self.invert.store(c.volume.invert, Relaxed);
        self.ignore_injected
            .store(c.volume.ignore_injected, Relaxed);
        self.primary.store(c.taskbar.primary, Relaxed);
        self.secondary.store(c.taskbar.secondary, Relaxed);
        self.rehook_ms
            .store(c.advanced.rehook_interval_s.saturating_mul(1000), Relaxed);
    }
}

/// Çalışan örnek. Drop → hook kaldırılır, thread'ler biter, mutex bırakılır.
pub struct Handle {
    shared: Arc<Shared>,
    hook_tid: u32,
    hook: Option<JoinHandle<()>>,
    worker: Option<JoinHandle<()>>,
    _mutex: hook::NamedMutex,
}

impl Handle {
    pub fn enabled(&self) -> bool {
        self.shared.enabled.load(Relaxed)
    }

    pub fn set_enabled(&self, on: bool) {
        self.shared.enabled.store(on, Relaxed);
    }

    /// Yeni config'i uygular (ör. tray "Yeniden yükle").
    pub fn reload(&self, c: &Config) {
        self.shared.apply(c);
        hook::post(self.hook_tid, hook::WM_APP_RELOAD);
    }

    pub fn stop(self) {}
}

impl Drop for Handle {
    fn drop(&mut self) {
        hook::post(self.hook_tid, hook::QUIT);
        // Hook thread çıkınca Sender düşer → worker'ın recv'i biter.
        for t in [self.hook.take(), self.worker.take()].into_iter().flatten() {
            let _ = t.join();
        }
    }
}

/// Hook'u kurar. Süreçler arası tek örnek: ikinci çağrı `Error::AlreadyRunning`.
pub fn start(cfg: &Config) -> Result<Handle, Error> {
    let mutex = hook::acquire_mutex().map_err(|e| match e {
        hook::MutexError::AlreadyRunning => Error::AlreadyRunning,
        hook::MutexError::Os(c) => Error::Os(c),
    })?;
    let shared = Arc::new(Shared::new(cfg));
    let (tx, rx) = mpsc::channel::<i16>();

    let s = shared.clone();
    let worker = thread::Builder::new()
        .name("lowvolume-worker".into())
        .spawn(move || worker(rx, &s))
        .map_err(Error::Spawn)?;

    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let s = shared.clone();
    let hook = thread::Builder::new()
        .name("lowvolume-hook".into())
        .spawn(move || hook::run(tx, s, ready_tx))
        .map_err(Error::Spawn)?; // başarısızsa tx düşer → worker kendiliğinden biter

    match ready_rx.recv() {
        Ok(Ok(hook_tid)) => Ok(Handle {
            shared,
            hook_tid,
            hook: Some(hook),
            worker: Some(worker),
            _mutex: mutex,
        }),
        Ok(Err(code)) => {
            let _ = hook.join();
            let _ = worker.join();
            Err(Error::Os(code))
        }
        Err(_) => {
            let _ = worker.join();
            Err(Error::Os(0))
        }
    }
}

/// Kanalı boşaltır (doğal coalesce), biriktirir, tek SendInput.
fn worker(rx: mpsc::Receiver<i16>, shared: &Shared) {
    let mut acc = wheel::Acc::default();
    while let Ok(d) = rx.recv() {
        let mut ticks = acc.push(d.into());
        while let Ok(d) = rx.try_recv() {
            ticks += acc.push(d.into());
        }
        if shared.invert.load(Relaxed) {
            ticks = -ticks;
        }
        hook::send_volume(ticks);
    }
}
