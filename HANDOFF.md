# HANDOFF — lowvolume
**Güncelleme:** 2026-11-22 · **Güncelleyen:** Claude Code

## Durum (1-2 cümle)
v0.1.0 yayında (M1 + M2). M3 tamam: lowmonitor `lowmon-tray` lowvolume'u süreç içinde çalıştırıyor (kalkanozgur/lowmonitor PR #82, ADR-0039). Sıradaki iş M4 sağlamlaştırma.

## Son yapılanlar
- Repo: https://github.com/kalkanozgur/lowvolume (public), tag `v0.1.0`
- M1: `src/lib.rs`, `src/hook.rs`, `src/config.rs`, `src/wheel.rs`
- M2: `src/main.rs` (tray, menü, HKCU Run, log, WS kırpma); `general.autostart` config'ten çıktı
- M3 (lowmonitor): `crates/lowmon-tray/src/volume.rs`, "Ses (taskbar tekerleği) ▸" menüsü; Handle `WM_DESTROY`'da bırakılır

## Açık sorun / blokaj
- Menü öğeleri (standalone + lowmon tray), explorer yeniden başlatma, ikincil monitör elle denenmedi
- Standalone kapandıktan sonra lowmon tray yalnız "Ayarı yeniden yükle" ile devralır
- İlk lowmon canlı testinde tekerlek bir kez tepkisiz kaldı, tekrarlanamadı

## Sıradaki adımlar (sırayla)
1. Elle test: menü tıklamaları, explorer restart, ikincil monitör
2. M4: UAC/yükseltilmiş pencere, Win10, DPI, taskbar konumları
3. Lib API dersi: `Handle` süreç çıkışında TLS'te düşürülmemeli (join abort) — README/lib doc'a not

## Dokunma
- (yok)

## Kritik dosyalar
- `CLAUDE.md` — kurallar
- `docs/spec.md` — tek doğruluk kaynağı
- `src/hook.rs` — hook callback (blok iş yasak)

## Doğrulama
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo run --example console`

## Kısıtlar / kararlar
- Rust 1.100.0, edition 2024; `windows-sys` 0.61.2, `serde` 1.0.229, `toml` 1.2.1 (lowmonitor ile aynı)
- Ses adımı Windows'un sabit %2'si (VK_VOLUME_UP/DOWN)
- Config `%APPDATA%\LowMonitor\lowvolume.toml`; log `%LOCALAPPDATA%\LowMonitor\lowvolume.log`
- Autostart: yalnız kayıt defteri `HKCU\...\Run\lowvolume`
- Tek örnek: named mutex `Local\LowMonitor.Volume`
- DPI: hook thread'inde `SetThreadDpiAwarenessContext(PMv2)`

---
## Geri dönüş (Claude Code'a döndüğünde)
> HANDOFF.md'yi oku. Aşağıdaki çıktı <Gemini/OpenCode>'den geldi. Uygulamadan önce mevcut kodla çelişkileri kontrol et, sonra uygula, devlog'a ekle, HANDOFF.md'yi güncelle.
