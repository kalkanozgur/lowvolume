# Changelog
Format: Keep a Changelog. Sürümleme: CLAUDE.md > Sürümleme.

## [Unreleased]

## [0.1.1] - 2026-11-22
### Düzeltildi
- %100 dışı ölçekli monitörün taskbar'ında (ör. %125 ikincil) tekerlek çalışmıyordu: Windows LL hook callback'ini DPI-unaware çağırıyor, `WindowFromPoint` fiziksel koordinatta NULL dönüyordu. DPI bağlamı callback içinde kurulup geri alınıyor

## [0.1.0] - 2026-11-22
### Eklendi
- Proje iskeleti
- Çekirdek kütüphane: taskbar üstünde tekerlek → sistem sesi (Windows flyout), `lowvolume.toml` config, tek örnek mutex
- Standalone `lowvolume.exe`: tray menüsü (Etkin, Config'i aç/yeniden yükle, Başlangıçta çalıştır, Çıkış), log
