# lowvolume
Windows görev çubuğu üstünde mouse tekerleğiyle sistem sesini değiştirir. LowMonitor ailesinden küçük bir Rust tray uygulaması; tek başına veya LowMonitor tray'i içinde çalışır.

English: [README.md](README.md)

| Alan | Değer |
|---|---|
| Durum | Tasarım |
| Modüller | software (Rust: lib + bin) |
| Başlangıç | 2026-11-22 |

## Klasör yapısı
```
lowvolume/
├─ Cargo.toml
├─ rust-toolchain.toml
├─ src/
│  ├─ lib.rs        # hook + volume (used by lowmon-tray too)
│  └─ main.rs       # standalone lowvolume.exe
└─ docs/
   ├─ spec.md
   ├─ requirements.md
   ├─ devlog/
   ├─ decisions/
   └─ reference/
```

## Hızlı başlangıç
```
cargo build --release
```
`target/release/lowvolume.exe`'yi çalıştır: tray'de hoparlör ikonu çıkar; taskbar üstünde tekerlekle ses değişir. İkona sağ tık: Etkin / Config'i aç / Config'i yeniden yükle / Başlangıçta çalıştır / Çıkış.

- Config: `%APPDATA%\LowMonitor\lowvolume.toml` ("Config'i aç" varsayılanlarla oluşturur)
- Log: `%LOCALAPPDATA%\LowMonitor\lowvolume.log`
- LowMonitor tray'i lowvolume'u zaten çalıştırıyorsa standalone exe sessizce çıkar.

## Yapay zeka ajanı devri
Son durum ve sıradaki adımlar `HANDOFF.md`'de. Kurallar `CLAUDE.md`'de.

## Lisans
MIT OR Apache-2.0
