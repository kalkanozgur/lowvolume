# Gereksinimler — lowvolume

## Amaç
İmleç Windows taskbar'ı üstündeyken mouse tekerleği sistem sesini değiştirsin; normal scroll yutulsun. Tek başına veya LowMonitor tray'i içinde.

## Fonksiyonel gereksinimler
- Birincil (`Shell_TrayWnd`) ve ikincil (`Shell_SecondaryTrayWnd`) taskbar üstünde tekerlek → VK_VOLUME_UP/DOWN.
- Taskbar dışında scroll'a dokunulmaz; yatay tekerlek (`WM_MOUSEHWHEEL`) geçer.
- Yüksek çözünürlüklü tekerlek: delta 120'nin katı değilse biriktirilir.
- `invert`, `ignore_injected`, primary/secondary seçimi config'ten.
- Standalone: tray (Etkin · Config'i aç · Yeniden yükle · Başlangıçta çalıştır · Çıkış).
- lowmon-tray içinde: "Ses ▸" menüsü; aynı anda yalnız bir hook (mutex).
- Bozuk config → varsayılanlar + log.

## Performans
- Idle CPU %0.0; çalışma kümesi < 3 MB (standalone); exe < 500 KB.
- 1 dk sürekli hızlı scroll: hook düşmez, gecikme hissedilmez.

## Ortam
- Windows 10/11 x64; PerMonitorV2 DPI; çoklu monitör; taskbar otomatik gizle / üst / yan.

## Kabul kriterleri
- [ ] Taskbar üstünde tekerlek ses değiştirir + yerel flyout çıkar; pencere scroll olmaz.
- [ ] Taskbar dışında normal scroll bozulmaz.
- [ ] İkincil monitör taskbar'ı çalışır.
- [ ] Varsayılan çıkış cihazı değişince (kulaklık) çalışmaya devam eder.
- [ ] Standalone tray; explorer yeniden başlayınca ikon geri gelir.
- [ ] Standalone + lowmon-tray aynı anda → tek hook.
- [ ] Performans maddeleri ölçümle doğrulanır.
- [ ] Yükseltilmiş pencere ön plandayken davranış: TBD (ölçülecek).
