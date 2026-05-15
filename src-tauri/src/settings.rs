use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

/// Drop hotkey bindings that fail to parse (e.g. legacy `hot_corner`
/// triggers from v0.5.0/0.5.1) instead of aborting the whole settings
/// load and resetting the user's other preferences.
fn deserialize_lenient_bindings<'de, D>(deserializer: D) -> Result<Vec<HotkeyBinding>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Vec<toml::Value> = Vec::deserialize(deserializer)?;
    let mut out = Vec::with_capacity(raw.len());
    for v in raw {
        match v.try_into::<HotkeyBinding>() {
            Ok(b) => out.push(b),
            Err(e) => tracing::warn!(error = %e, "dropping unparseable hotkey binding"),
        }
    }
    Ok(out)
}

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
    /// Force-click on a Force Touch trackpad (macOS only). Fires when
    /// NSEvent stage transitions from 1 (normal click) to 2 (force).
    ForceTouch,
    /// N-finger tap on the trackpad (macOS only, via private
    /// MultitouchSupport framework). Fires when `fingers` simultaneous
    /// contacts are released within `max_duration_ms`.
    TrackpadTap {
        fingers: u8,
        #[serde(default = "default_tap_max_ms")]
        max_duration_ms: u32,
    },
}

fn default_tap_max_ms() -> u32 { 200 }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RadialTheme {
    #[serde(default = "default_backdrop_color")]
    pub backdrop_color: String,
    #[serde(default = "default_backdrop_opacity")]
    pub backdrop_opacity: f32,
    #[serde(default = "default_sector_color")]
    pub sector_color: String,
    #[serde(default = "default_sector_opacity")]
    pub sector_opacity: f32,
    #[serde(default = "default_hover_color")]
    pub hover_color: String,
    #[serde(default = "default_center_color")]
    pub center_color: String,
}

fn default_backdrop_color() -> String { "#000000".into() }
fn default_backdrop_opacity() -> f32 { 0.0 }
fn default_sector_color() -> String { "#1f2937".into() }
fn default_sector_opacity() -> f32 { 0.85 }
fn default_hover_color() -> String { "#3b82f6".into() }
fn default_center_color() -> String { "#111827".into() }

impl Default for RadialTheme {
    fn default() -> Self {
        Self {
            backdrop_color: default_backdrop_color(),
            backdrop_opacity: default_backdrop_opacity(),
            sector_color: default_sector_color(),
            sector_opacity: default_sector_opacity(),
            hover_color: default_hover_color(),
            center_color: default_center_color(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub indicator_style: IndicatorStyle,
    pub autostart: bool,
    /// Master switch for the OS notification → cursor indicator pipeline.
    /// When false the overlay subscriber drops incoming events (the OS
    /// source still runs so toggling back on is instant).
    #[serde(default = "default_true")]
    pub indicator_enabled: bool,
    #[serde(default)]
    pub menu_items: Vec<MenuItem>,
    #[serde(default, deserialize_with = "deserialize_lenient_bindings")]
    pub hotkey_bindings: Vec<HotkeyBinding>,
    /// When true, the radial menu auto-closes the moment the cursor leaves
    /// the menu window. Default false (close requires explicit click /
    /// ESC / focus-loss).
    #[serde(default)]
    pub radial_close_on_leave: bool,
    #[serde(default)]
    pub radial_theme: RadialTheme,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            indicator_style: IndicatorStyle::RingPulse,
            autostart: false,
            indicator_enabled: true,
            menu_items: Vec::new(),
            hotkey_bindings: Vec::new(),
            radial_close_on_leave: false,
            radial_theme: RadialTheme::default(),
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
    legacy: Vec<PathBuf>,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            legacy: legacy_config_paths(),
        }
    }

    /// Constructor for tests: skips legacy fallback so a developer's real
    /// mouse-noti config on disk does not bleed into unit tests.
    #[cfg(test)]
    pub fn new_isolated(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            legacy: Vec::new(),
        }
    }

    pub fn load(&self) -> Settings {
        let backup = self.path.with_extension("toml.bak");
        let mut candidates: Vec<PathBuf> = vec![self.path.clone(), backup];
        candidates.extend(self.legacy.iter().cloned());
        for candidate in &candidates {
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
        directories::ProjectDirs::from("dev", "glance", "glance").expect("project dirs");
    proj.config_dir().join("config.toml")
}

/// Pre-rebrand config locations. SettingsStore::load() tries these as a
/// fallback so existing users keep their settings after the mouse-noti →
/// Glance rename. The first save to the new location supersedes them.
fn legacy_config_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(proj) =
        directories::ProjectDirs::from("dev", "mouse-noti", "mouse-noti")
    {
        out.push(proj.config_dir().join("config.toml"));
        out.push(proj.config_dir().join("config.toml.bak"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn save_then_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let store = SettingsStore::new_isolated(&path);
        let s = Settings {
            indicator_style: IndicatorStyle::IconBadge,
            autostart: true,
            indicator_enabled: true,
            menu_items: Vec::new(),
            hotkey_bindings: Vec::new(),
            radial_close_on_leave: false,
            radial_theme: RadialTheme::default(),
        };
        store.save(&s).unwrap();
        let loaded = store.load();
        assert_eq!(loaded, s);
    }

    #[test]
    fn corrupt_file_falls_back_to_backup() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let store = SettingsStore::new_isolated(&path);
        let good = Settings {
            indicator_style: IndicatorStyle::PersistentBadge,
            autostart: false,
            indicator_enabled: true,
            menu_items: Vec::new(),
            hotkey_bindings: Vec::new(),
            radial_close_on_leave: false,
            radial_theme: RadialTheme::default(),
        };
        store.save(&good).unwrap();
        std::fs::write(&path, "GARBAGE").unwrap();
        let loaded = store.load();
        assert_eq!(loaded, good);
    }

    #[test]
    fn missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new_isolated(dir.path().join("nope.toml"));
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
            radial_theme: RadialTheme::default(),
            indicator_enabled: true,
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
