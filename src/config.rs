//! `%APPDATA%\LowMonitor\lowvolume.toml` (spec §6). Eksik alan → varsayılan, bilinmeyen alan yok sayılır.

use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: General,
    pub volume: Volume,
    pub taskbar: Taskbar,
    pub advanced: Advanced,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct General {
    pub enabled: bool,
    /// Yalnız standalone (HKCU Run); lowmon-tray içinde yok sayılır.
    pub autostart: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Volume {
    pub invert: bool,
    pub ignore_injected: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Taskbar {
    pub primary: bool,
    pub secondary: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Advanced {
    /// 0 = kapalı.
    pub rehook_interval_s: u32,
}

impl Default for General {
    fn default() -> Self {
        Self {
            enabled: true,
            autostart: false,
        }
    }
}
impl Default for Volume {
    fn default() -> Self {
        Self {
            invert: false,
            ignore_injected: true,
        }
    }
}
impl Default for Taskbar {
    fn default() -> Self {
        Self {
            primary: true,
            secondary: true,
        }
    }
}
impl Default for Advanced {
    fn default() -> Self {
        Self {
            rehook_interval_s: 60,
        }
    }
}

impl Config {
    /// `%APPDATA%\LowMonitor\lowvolume.toml`
    pub fn path() -> PathBuf {
        PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default())
            .join("LowMonitor")
            .join("lowvolume.toml")
    }

    /// Dosya yoksa varsayılan, hata yok. Bozuksa varsayılan + hata metni (çağıran loglar).
    pub fn load(path: &Path) -> (Self, Option<String>) {
        match std::fs::read_to_string(path) {
            Ok(s) => Self::parse(&s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Self::default(), None),
            Err(e) => (Self::default(), Some(format!("{}: {e}", path.display()))),
        }
    }

    pub fn parse(s: &str) -> (Self, Option<String>) {
        match toml::from_str(s) {
            Ok(c) => (c, None),
            Err(e) => (Self::default(), Some(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_default() {
        assert_eq!(Config::parse("").0, Config::default());
    }

    #[test]
    fn partial_keeps_other_defaults() {
        let (c, err) = Config::parse("[volume]\ninvert = true\n[unknown]\nx = 1\n");
        assert!(err.is_none());
        assert!(c.volume.invert);
        assert!(c.volume.ignore_injected);
        assert_eq!(c.advanced.rehook_interval_s, 60);
    }

    #[test]
    fn broken_falls_back_with_error() {
        let (c, err) = Config::parse("[volume\ninvert = ");
        assert_eq!(c, Config::default());
        assert!(err.is_some());
        let (c, err) = Config::parse("[volume]\ninvert = 3\n");
        assert_eq!(c, Config::default());
        assert!(err.is_some());
    }

    #[test]
    fn missing_file_is_silent_default() {
        let (c, err) = Config::load(Path::new("Z:\\definitely\\missing\\lowvolume.toml"));
        assert_eq!(c, Config::default());
        assert!(err.is_none());
    }
}
