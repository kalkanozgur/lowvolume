# HANDOFF — lowvolume
**Güncelleme:** 2026-11-22 19:30 · **Güncelleyen:** Claude Code

## Durum (1-2 cümle)
Proje başlatıldı, iskelet hazır; spec revize edildi (`docs/spec.md`). Kod yok (boş lib + bin).

## Son yapılanlar
- Proje iskeleti oluşturuldu
- Spec değerlendirildi: WASAPI/OSD/pipe düşürüldü, native flyout + süreç içi lowmon entegrasyonu (ADR-0001)

## Açık sorun / blokaj
- GitHub reposu henüz oluşturulmadı / push edilmedi (kullanıcı onayı bekleniyor)

## Sıradaki adımlar (sırayla)
1. M1: `src/lib.rs` + `src/hook.rs` — WH_MOUSE_LL, taskbar tespiti, SendInput, delta biriktirme testleri
2. M2: standalone tray + config + mutex + autostart, tag v0.1.0
3. M3: lowmonitor reposunda ADR + lowmon-tray entegrasyonu (ayrı PR)

## Dokunma
- (yok)

## Kritik dosyalar
- `CLAUDE.md` — kurallar
- `docs/spec.md` — tek doğruluk kaynağı
- `docs/decisions/ADR-0001-project-kickoff.md` — mimari kararlar

## Doğrulama
- `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`

## Kısıtlar / kararlar
- Rust 1.100.0 (rust-toolchain.toml, lowmonitor ile aynı), edition 2024
- Ses adımı Windows'un sabit %2'si (VK_VOLUME_UP/DOWN)
- Config `%APPDATA%\LowMonitor\lowvolume.toml`
- Çift çalışma: named mutex `Local\LowMonitor.Volume`

---
## Geri dönüş (Claude Code'a döndüğünde)
> HANDOFF.md'yi oku. Aşağıdaki çıktı <Gemini/OpenCode>'den geldi. Uygulamadan önce mevcut kodla çelişkileri kontrol et, sonra uygula, devlog'a ekle, HANDOFF.md'yi güncelle.
