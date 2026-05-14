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
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub icon: IconSource,
    pub action: Action,
    #[serde(default = "default_tags")]
    pub tags: Vec<String>,
}

fn default_tags() -> Vec<String> { vec!["launcher".into()] }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IconSource {
    Emoji { value: String },
    AppIconPng { base64: String, source_path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    LaunchApp { path: String },
    OpenUrl { url: String },
    RunShell {
        command: String,
        args: Vec<String>,
        #[serde(default = "default_true")]
        confirm: bool,
    },
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub id: String,
    pub trigger: HotkeyTrigger,
    #[serde(default = "default_mode")]
    pub menu_mode: String,
}

fn default_mode() -> String { "all".into() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HotkeyTrigger {
    Keyboard { accelerator: String },
    Mouse { button: u8, modifiers: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub indicator_style: IndicatorStyle,
    pub autostart: bool,
    #[serde(default)]
    pub menu_items: Vec<MenuItem>,
    #[serde(default)]
    pub hotkey_bindings: Vec<HotkeyBinding>,
    /// When true, the radial menu auto-closes the moment the cursor leaves
    /// the menu window. Default false (close requires explicit click /
    /// ESC / focus-loss).
    #[serde(default)]
    pub radial_close_on_leave: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            indicator_style: IndicatorStyle::RingPulse,
            autostart: false,
            menu_items: Vec::new(),
            hotkey_bindings: Vec::new(),
            radial_close_on_leave: false,
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
            menu_items: Vec::new(),
            hotkey_bindings: Vec::new(),
            radial_close_on_leave: false,
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
            menu_items: Vec::new(),
            hotkey_bindings: Vec::new(),
            radial_close_on_leave: false,
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

    #[test]
    fn menu_item_emoji_roundtrip() {
        let s = Settings {
            indicator_style: IndicatorStyle::RingPulse,
            autostart: false,
            menu_items: vec![MenuItem {
                id: "fixed-id".into(),
                label: "Open Slack".into(),
                icon: IconSource::Emoji { value: "💬".into() },
                action: Action::LaunchApp { path: "/Applications/Slack.app".into() },
                tags: vec!["launcher".into()],
            }],
            hotkey_bindings: vec![HotkeyBinding {
                id: "hk-1".into(),
                trigger: HotkeyTrigger::Keyboard { accelerator: "CommandOrControl+Shift+M".into() },
                menu_mode: "all".into(),
            }],
            radial_close_on_leave: false,
        };
        let toml = toml::to_string_pretty(&s).unwrap();
        let parsed: Settings = toml::from_str(&toml).unwrap();
        assert_eq!(parsed, s);
    }

    #[test]
    fn legacy_v0_2_settings_loads_with_defaults() {
        let legacy = r#"
indicator_style = "ring_pulse"
autostart = false
"#;
        let parsed: Settings = toml::from_str(legacy).unwrap();
        assert_eq!(parsed.indicator_style, IndicatorStyle::RingPulse);
        assert_eq!(parsed.autostart, false);
        assert!(parsed.menu_items.is_empty());
        assert!(parsed.hotkey_bindings.is_empty());
    }
}
