# Proje kuralları — lowvolume

## Genel
- Dil: doküman Türkçe; README İngilizce + Türkçe (README.md / README.tr.md, senkron). Kodda İngilizce tanımlayıcı + Türkçe yorum serbest.
- Klasör ve dosya adları İngilizce, küçük harf, kebab-case.
- Tarih formatı ISO: YYYY-MM-DD.
- Tek doğruluk kaynağı: `docs/spec.md`. Spekten sapan karar → önce ADR, sonra spek güncellemesi.
- Her anlamlı değişiklikten sonra: devlog girdisi + gerekiyorsa CHANGELOG.
- Geri dönüşü pahalı kararlar → `docs/decisions/ADR-XXXX-<konu>.md`.

## Rust
- Cargo standardı: kökte `Cargo.toml`, `src/lib.rs` (lowmon-tray de kullanır), `src/main.rs` (standalone bin).
- Bağımlılık sürümleri lowmonitor `[workspace.dependencies]` ile aynı (ör. `windows-sys` 0.61, `serde`, `toml`); yeni bağımlılık ADR ile.
- Yasak: tokio/async runtime, WMI, .NET, WebView, `tray-icon`/`winit`.
- `cargo fmt` + `cargo clippy -- -D warnings` temiz olmadan push yok.
- `unsafe` blokları minimum; her birinin üstünde `// SAFETY:` yorumu.
- Hook callback'i blok işlem yapmaz (WindowFromPoint + sınıf kontrolü + kanala gönderim dışında iş yok).
- Performans bütçesi: idle CPU %0, çalışma kümesi < 3 MB (standalone). Aşan değişiklik ölçümle gelir.

## Lib API kararlılığı
- `lowmon-tray` bu repoyu `tag` ile git bağımlılığı olarak çeker. Public API değişikliği → MINOR (0.x) bump + lowmonitor tarafında PR.

## Handoff
- Oturum sonunda veya "token azalıyor" denince `HANDOFF.md`'yi güncelle (durum, açık sorun, sıradaki 3 adım, dokunma listesi).
- Başka bir araçtan (Gemini/OpenCode) gelen çıktıyı uygulamadan önce mevcut kodla ve `docs/spec.md` ile çelişkileri kontrol et.

## Sürümleme
- SemVer, `vMAJOR.MINOR.PATCH` git tag; tag öncesi CHANGELOG güncellenir.

## Commit mesajları
Conventional Commits: `feat(hook): …`, `fix(tray): …`, `docs: …`, `chore: …`
