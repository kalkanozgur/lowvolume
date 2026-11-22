# HANDOFF — lowvolume
**Güncelleme:** 2026-11-22 · **Güncelleyen:** Claude Code

## Durum (1-2 cümle)
M1 çekirdek kütüphane yazıldı (dal `feat/m1-core`): hook + taskbar tespiti + SendInput + mutex + config. Bin (`src/main.rs`) hâlâ boş; elle doğrulama `examples/console.rs` ile.

## Son yapılanlar
- Repo: https://github.com/kalkanozgur/lowvolume (public)
- `src/lib.rs` (API, worker), `src/hook.rs` (Win32), `src/config.rs`, `src/wheel.rs`; 10 test

## Açık sorun / blokaj
- Gerçek taskbar scroll testi elle yapılmadı

## Sıradaki adımlar (sırayla)
1. Elle test: `cargo run --example console` (birincil/ikincil taskbar, taskbar dışı scroll, hızlı scroll)
2. M2: standalone tray + autostart + log, tag v0.1.0
3. M3: lowmonitor reposunda ADR + lowmon-tray entegrasyonu (ayrı PR)

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
- Config `%APPDATA%\LowMonitor\lowvolume.toml`
- Tek örnek: named mutex `Local\LowMonitor.Volume`
- DPI: hook thread'inde `SetThreadDpiAwarenessContext(PMv2)`

---
## Geri dönüş (Claude Code'a döndüğünde)
> HANDOFF.md'yi oku. Aşağıdaki çıktı <Gemini/OpenCode>'den geldi. Uygulamadan önce mevcut kodla çelişkileri kontrol et, sonra uygula, devlog'a ekle, HANDOFF.md'yi güncelle.
