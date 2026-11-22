//! M1 elle doğrulama: `cargo run --example console`, taskbar üstünde tekerleği çevir → Windows ses flyout'u.

fn main() {
    let (mut cfg, err) = lowvolume::Config::load(&lowvolume::Config::path());
    if let Some(e) = err {
        eprintln!("config hatası, varsayılanlar: {e}");
    }
    // Otomatik test: SendInput ile üretilen tekerlek olaylarını da işle.
    if std::env::args().any(|a| a == "--allow-injected") {
        cfg.volume.ignore_injected = false;
    }
    let h = match lowvolume::start(&cfg) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("başlatılamadı: {e}");
            std::process::exit(1);
        }
    };
    println!("lowvolume çalışıyor ({cfg:?}). Taskbar üstünde tekerleği çevir. Çıkış: Enter");
    let _ = std::io::stdin().read_line(&mut String::new());
    h.stop();
    println!("hook kaldırıldı");
}
