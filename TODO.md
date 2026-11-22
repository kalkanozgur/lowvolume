# TODO
## Şimdi
- [x] GitHub reposu (public) oluştur + push
- [x] M1: hook + taskbar tespiti + SendInput + delta testleri
- [ ] M1 elle test: `cargo run --example console` → taskbar scroll, ikincil monitör, taskbar dışı scroll
## Sonra
- [x] M2: standalone tray, config, mutex, autostart
- [ ] M2 elle test: tray menü öğeleri, explorer yeniden başlatma
- [x] v0.1.0 tag
- [x] M3: lowmon-tray entegrasyonu (lowmonitor ADR-0039, PR #82)
- [x] M3 elle test: lowmon tray "Ses ▸" menüsü + fiziksel tekerlek (kullanıcı, LowMonitor 1.2.0 kurulumu)
- [ ] M4: sağlamlaştırma (explorer restart, UAC, Win10/11, DPI, ikincil monitör)
## Fikirler
- Orta tık = mute
- Başlat menüsü / tray taşma alanı üstünde de çalışma
- Kod imzası (AV yanlış pozitifleri)
