# lowvolume — Spec (rev 2, 2026-11-22)

Taskbar üzerinde mouse tekerleği ile sistem ses seviyesi kontrolü. Volumouse (NirSoft) çekirdek davranışının LowMonitor ailesine uygun, çok düşük kaynaklı Rust karşılığı.

- **Durum:** Tasarım
- **Aile:** LowMonitor — ayrı repo; lowmon-tray içinde süreç içi veya standalone exe
- **Hedef OS:** Windows 10/11 x64
- rev 1 (ilk taslak) → rev 2 farkları: [ADR-0001](decisions/ADR-0001-project-kickoff.md)

## 1. Hedefler / Hedef dışı
**Hedefler**
- İmleç taskbar üstündeyken tekerlek → ses; normal scroll yutulur.
- Idle %0 CPU, çalışma kümesi < 3 MB, exe < 500 KB.
- Ses göstergesi: Windows'un yerel flyout'u.

**Hedef dışı (v1)**
- Ayarlanabilir adım (Windows %2 sabit), modifier tuşlar, uygulama bazlı ses, parlaklık.
- Özel OSD, WASAPI, named pipe, GUI ayar penceresi, servis modu.

## 2. Mimari
```
lowvolume (lib)
├─ hook thread   : WH_MOUSE_LL + mesaj döngüsü + rehook timer (yalnız bu)
└─ worker thread : kanaldan delta → biriktir → SendInput(VK_VOLUME_*)

lowvolume.exe (bin, standalone) : lib + gizli pencere + tray
lowmon-tray (lowmonitor)        : lib'i git bağımlılığıyla süreç içi başlatır
```
- **Servis değil:** Session 0'da masaüstü girdisi yok; `WH_MOUSE_LL` interaktif masaüstü gerektirir.
- **Çift çalışma:** `start()` named mutex `Local\LowMonitor.Volume` alır; alamazsa `Err(AlreadyRunning)`.
  - Standalone başlarken dolu → sessiz çıkış.
  - lowmon-tray başlarken dolu → modülü başlatmaz, menüde "standalone çalışıyor".
- API: `pub fn start(cfg: Config) -> Result<Handle, Error>`; `Handle::{set_enabled, reload, stop}`; Drop → unhook + thread join.

## 3. Hook
- `SetWindowsHookExW(WH_MOUSE_LL, proc, hmod, 0)` ayrı thread'de.
- Yalnız `WM_MOUSEWHEEL` (`WM_MOUSEHWHEEL` geçer).
- `MSLLHOOKSTRUCT`: `pt`; delta = `(mouseData >> 16) as i16`; `ignore_injected` ise `LLMHF_INJECTED` atlanır (Logitech/Synergy/RDP için kapatılabilir).
- Taskbar değil → `CallNextHookEx`. Taskbar → delta kanala, `LRESULT(1)`.
- Callback'te yalnız: `WindowFromPoint`, `GetAncestor(GA_ROOT)`, `GetClassNameW`, kanal gönderimi. Panic yok.
- Zaman aşımı: `LowLevelHooksTimeout` (~300 ms) aşılırsa Windows hook'u sessizce düşürür; tespit API'si yok → `rehook_interval_s` (varsayılan 60) ile periyodik unhook + yeniden kurulum.
- DPI: süreç PerMonitorV2 (manifest); `pt` fiziksel koordinat, `WindowFromPoint` ile tutarlı. lowmon-tray manifest'i de PMv2 olmalı.

### 3.1 Taskbar tespiti
`GetAncestor(WindowFromPoint(pt), GA_ROOT)` sınıfı ∈ {`Shell_TrayWnd` (primary), `Shell_SecondaryTrayWnd` (secondary)}. Win11 XAML alt penceresi (`DesktopWindowContentBridge`) GA_ROOT ile köke çıkar. Başlat/arama/taşma pencereleri v1'de dahil değil.

## 4. Ses
- Worker: `recv()` sonrası `try_recv()` ile kuyruğu boşalt (doğal coalesce).
- `acc += delta; n = acc / 120; acc %= 120`; `invert` ise işaret ters; yön değişince `acc` sıfırlanır.
- `n > 0` → `SendInput` |n| × (VK_VOLUME_UP|DOWN down+up), tek çağrı.
- Mute, cihaz değişimi, flyout: Windows.

## 5. Tray (yalnız standalone)
- `Shell_NotifyIconW` + gizli mesaj penceresi; `TaskbarCreated` → ikonu geri ekle.
- Menü: Etkin (✓) · Config'i aç · Config'i yeniden yükle · Başlangıçta çalıştır (✓, HKCU Run) · Çıkış. Çift tık: etkin/devre dışı.

## 6. Config — `%APPDATA%\LowMonitor\lowvolume.toml`
```toml
[general]
enabled = true
autostart = false        # yalnız standalone
[volume]
invert = false
ignore_injected = true
[taskbar]
primary = true
secondary = true
[advanced]
rehook_interval_s = 60   # 0 = kapalı
```
Eksik alan → varsayılan; bozuk dosya → tümü varsayılan + log. Yeniden yükleme manuel (tray/lowmon menüsü).

## 7. Riskler
| Konu | Durum / Çözüm |
|---|---|
| Yükseltilmiş pencere ön planda (UIPI) | Ölçülecek (M4) |
| Hook'un sessizce düşmesi | Minimal callback + rehook 60 s |
| Win11 taskbar yapısı | GA_ROOT kök sınıf; Windows güncellemelerinde regresyon testi |
| AV / anti-cheat | İmzasız global hook yanlış pozitif olabilir; yalnız tekerlek, tuş kaydı yok |
| lowmon-tray süreci içinde hata | panic=abort tray'i düşürür → callback/worker panic'siz, hatalar `Result` |

## 8. Bağımlılıklar
- `windows-sys` 0.61, `serde`, `toml` (lowmonitor sürümleri). `tray-icon`/`winit` yok.
- Release: `opt-level="z"`, lto, codegen-units=1, panic=abort, strip. `windows_subsystem = "windows"`.

## 9. Yol haritası
1. **M1 çekirdek:** lib, hook, taskbar tespiti, SendInput, mutex; delta/config testleri.
2. **M2 standalone:** tray, config, autostart, manifest; tag v0.1.0.
3. **M3 lowmon entegrasyonu:** lowmonitor ADR + lowmon-tray PR.
4. **M4 sağlamlaştırma:** explorer restart, UAC, Win10/11, DPI, ikincil monitör, otomatik gizle, taskbar konumu.

## 10. Açık sorular (v2)
- Orta tık = mute? Başlat/taşma alanı? Kod imzası?
