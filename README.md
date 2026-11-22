# lowvolume
Scroll the mouse wheel over the Windows taskbar to change system volume. Tiny Rust tray app from the LowMonitor family; runs standalone or inside the LowMonitor tray.

Türkçe: [README.tr.md](README.tr.md)

| Field | Value |
|---|---|
| Status | Design |
| Modules | software (Rust: lib + bin) |
| Started | 2026-11-22 |

## Folder structure
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

## Quick start
```
cargo build --release
```
Run `target/release/lowvolume.exe`: a speaker icon appears in the tray; scroll over the taskbar to change volume. Right-click the icon for Enabled / Open config / Reload config / Run at startup / Exit.

- Config: `%APPDATA%\LowMonitor\lowvolume.toml` (created with defaults by "Open config")
- Log: `%LOCALAPPDATA%\LowMonitor\lowvolume.log`
- If the LowMonitor tray already runs lowvolume, the standalone exe exits silently.

## AI agent handoff
See `HANDOFF.md` for current state and next steps. Rules live in `CLAUDE.md`.

## License
MIT OR Apache-2.0
