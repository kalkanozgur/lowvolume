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
Output: `target/release/lowvolume.exe` (to be completed in M2).

## AI agent handoff
See `HANDOFF.md` for current state and next steps. Rules live in `CLAUDE.md`.

## License
MIT OR Apache-2.0
