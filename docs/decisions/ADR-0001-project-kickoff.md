# ADR-0001: Proje başlangıcı ve mimari
- Tarih: 2026-11-22
- Durum: Kabul

## Bağlam
Taskbar üstünde tekerlekle sistem sesi (Volumouse çekirdeği), LowMonitor ailesine uygun düşük kaynak. İlk taslak spec: ayrı exe + named pipe + lowmon'un süreci izlemesi + WASAPI + özel GDI OSD. lowmonitor'da modüller tray sürecinde barındırılıyor (lowmon-deck, lowmonitor ADR-0031).

## Karar
| Konu | Karar | Neden |
|---|---|---|
| Repo | Ayrı public repo `kalkanozgur/lowvolume`, MIT OR Apache-2.0 | Bağımsız dağıtım; lowmon'dan token'sız git bağımlılığı |
| Yapı | Tek crate: `lib` + `bin` (Cargo standardı; proje-baslat'ın `software/src` yerine) | lowmon-tray lib'i süreç içinde çalıştırır, standalone bin aynı lib'i kullanır |
| Ses | `SendInput(VK_VOLUME_UP/DOWN)` | Windows yerel flyout'u gelir; WASAPI/COM, cihaz takibi, OSD penceresi gereksiz. Bedel: adım %2 sabit |
| lowmon entegrasyonu | `lowmon-tray` → `lowvolume = { git, tag }`, süreç içi | Pipe, `--managed`, süreç izleme/yeniden başlatma gereksiz |
| Çift çalışma | Named mutex `Local\LowMonitor.Volume` | Standalone + tray aynı anda iki hook kurmasın |
| Config | `%APPDATA%\LowMonitor\lowvolume.toml` | Aile ile aynı dizin |
| Toolchain | Rust 1.100.0, edition 2024 (lowmonitor ile aynı) | Git bağımlılığında derleyici uyumu |

## Sonuçlar
- Artı: ~4 milestone'luk iş (WASAPI, OSD, pipe, süreç yönetimi) düştü; exe küçük, bağımlılık yalnız `windows-sys`/`serde`/`toml`.
- Eksi: adım boyutu ayarlanamaz; hook sorunu (panic=abort) lowmon-tray sürecini etkileyebilir → callback minimal, panic'siz.
- Risk: Win11 taskbar yapısı değişimi, yükseltilmiş pencere ön plandayken davranış (ölçülecek), AV yanlış pozitifi (imzasız global hook).
