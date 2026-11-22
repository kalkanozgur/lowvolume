# HANDOFF — lowvolume
**Güncelleme:** 2026-11-22 · **Güncelleyen:** Claude Code

## Durum (1-2 cümle)
M1 main'de (PR #1). M2 standalone tray dal `feat/m2-standalone`'da: `src/main.rs` tray + autostart + log; canlı test geçti.

## Son yapılanlar
- Repo: https://github.com/kalkanozgur/lowvolume (public)
- M1: `src/lib.rs`, `src/hook.rs`, `src/config.rs`, `src/wheel.rs`
- M2: `src/main.rs` (tray, menü, HKCU Run, log, WS kırpma); `general.autostart` config'ten çıktı

## Açık sorun / blokaj
- Tray menü öğeleri, explorer yeniden başlatma ve ikincil monitör elle denenmedi

## Sıradaki adımlar (sırayla)
1. M2 PR + merge, tag v0.1.0
2. M3: lowmonitor reposunda ADR + lowmon-tray entegrasyonu (ayrı PR)
3. M4: sağlamlaştırma (UAC, Win10, DPI, taskbar konumları)

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
