use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IndicatorStyle {
    RingPulse,
    IconBadge,
    PersistentBadge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub indicator_style: IndicatorStyle,
    pub autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            indicator_style: IndicatorStyle::RingPulse,
            autostart: false,
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Settings {
        let backup = self.path.with_extension("toml.bak");
        for candidate in [&self.path, &backup] {
            if let Ok(text) = std::fs::read_to_string(candidate) {
                if let Ok(s) = toml::from_str::<Settings>(&text) {
                    return s;
                }
            }
        }
        Settings::default()
    }

    pub fn save(&self, settings: &Settings) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if self.path.exists() {
            let backup = self.path.with_extension("toml.bak");
            let _ = std::fs::copy(&self.path, backup);
        }
        let text = toml::to_string_pretty(settings).expect("serializable");
        std::fs::write(&self.path, &text)?;
        // Also mirror the latest good content into the backup so a single
        // successful save always leaves a recoverable copy on disk.
        let backup = self.path.with_extension("toml.bak");
        let _ = std::fs::write(&backup, text);
        Ok(())
    }
}

pub fn default_config_path() -> PathBuf {
    let proj =
        directories::ProjectDirs::from("dev", "mouse-noti", "mouse-noti").expect("project dirs");
    proj.config_dir().join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn save_then_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let store = SettingsStore::new(&path);
        let s = Settings {
            indicator_style: IndicatorStyle::IconBadge,
            autostart: true,
        };
        store.save(&s).unwrap();
        let loaded = store.load();
        assert_eq!(loaded, s);
    }

    #[test]
    fn corrupt_file_falls_back_to_backup() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let store = SettingsStore::new(&path);
        let good = Settings {
            indicator_style: IndicatorStyle::PersistentBadge,
            autostart: false,
        };
        store.save(&good).unwrap();
        std::fs::write(&path, "GARBAGE").unwrap();
        let loaded = store.load();
        assert_eq!(loaded, good);
    }

    #[test]
    fn missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("nope.toml"));
        assert_eq!(store.load(), Settings::default());
    }
}
