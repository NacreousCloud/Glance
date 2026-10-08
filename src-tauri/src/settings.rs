use parking_lot::Mutex;
use serde::{Deserialize, Deserializer, Serialize};
use std::io::Write;
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

fn default_tags() -> Vec<String> {
    vec!["launcher".into()]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IconSource {
    Emoji { value: String },
    AppIconPng { base64: String, source_path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    LaunchApp {
        path: String,
    },
    OpenUrl {
        url: String,
    },
    RunShell {
        command: String,
        args: Vec<String>,
        #[serde(default = "default_true")]
        confirm: bool,
    },
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub id: String,
    pub trigger: HotkeyTrigger,
    #[serde(default = "default_mode")]
    pub menu_mode: String,
}

fn default_mode() -> String {
    "all".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HotkeyTrigger {
    Keyboard {
        accelerator: String,
    },
    Mouse {
        button: u8,
        modifiers: u8,
    },
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

fn default_tap_max_ms() -> u32 {
    200
}

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

fn default_backdrop_color() -> String {
    "#000000".into()
}
fn default_backdrop_opacity() -> f32 {
    0.0
}
fn default_sector_color() -> String {
    "#1f2937".into()
}
fn default_sector_opacity() -> f32 {
    0.85
}
fn default_hover_color() -> String {
    "#3b82f6".into()
}
fn default_center_color() -> String {
    "#111827".into()
}

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
    #[serde(default)]
    pub onboarding_completed: bool,
}

/// Preferences deliberately exclude menus and bindings: a stale settings
/// screen must never replace data managed by their dedicated commands.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferencesPatch {
    pub indicator_style: Option<IndicatorStyle>,
    pub indicator_enabled: Option<bool>,
    pub radial_close_on_leave: Option<bool>,
    pub radial_theme: Option<RadialTheme>,
    pub onboarding_completed: Option<bool>,
}

impl PreferencesPatch {
    pub fn apply(self, settings: &mut Settings) {
        if let Some(v) = self.indicator_style {
            settings.indicator_style = v;
        }
        if let Some(v) = self.indicator_enabled {
            settings.indicator_enabled = v;
        }
        if let Some(v) = self.radial_close_on_leave {
            settings.radial_close_on_leave = v;
        }
        if let Some(v) = self.radial_theme {
            settings.radial_theme = v;
        }
        if let Some(v) = self.onboarding_completed {
            settings.onboarding_completed = v;
        }
    }
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
            onboarding_completed: false,
        }
    }
}

pub struct SettingsStore {
    path: PathBuf,
    legacy: Vec<PathBuf>,
    lock: Mutex<()>,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            legacy: legacy_config_paths(),
            lock: Mutex::new(()),
        }
    }

    /// Constructor for tests: skips legacy fallback so a developer's real
    /// mouse-noti config on disk does not bleed into unit tests.
    #[cfg(test)]
    pub fn new_isolated(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            legacy: Vec::new(),
            lock: Mutex::new(()),
        }
    }

    pub fn load(&self) -> Settings {
        let _guard = self.lock.lock();
        self.read().unwrap_or_else(|e| {
            tracing::error!(error = %e, "unable to load settings");
            Settings::default()
        })
    }

    fn read(&self) -> std::io::Result<Settings> {
        let backup = self.path.with_extension("toml.bak");
        let mut candidates: Vec<PathBuf> = vec![self.path.clone(), backup];
        candidates.extend(self.legacy.iter().cloned());
        let mut error = None;
        for candidate in &candidates {
            match std::fs::read_to_string(candidate) {
                Ok(text) => match toml::from_str::<Settings>(&text) {
                    Ok(settings) => return Ok(settings),
                    Err(e) => error = Some(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => error = Some(e),
            }
        }
        match error {
            Some(e) => Err(e),
            None => Ok(Settings::default()),
        }
    }

    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> std::io::Result<Settings> {
        let _guard = self.lock.lock();
        let mut settings = self.read()?;
        change(&mut settings);
        self.save_unlocked(&settings)?;
        Ok(settings)
    }

    #[cfg(test)]
    pub fn save(&self, settings: &Settings) -> std::io::Result<()> {
        let _guard = self.lock.lock();
        self.save_unlocked(settings)
    }

    fn save_unlocked(&self, settings: &Settings) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        atomic_write(&self.path, &text)?;
        // Never copy a potentially corrupt primary over the valid backup.
        // Keep the previous backup if refreshing it fails.
        if let Err(e) = atomic_write(&self.path.with_extension("toml.bak"), &text) {
            tracing::warn!(error = %e, "unable to refresh settings backup");
        }
        Ok(())
    }
}

fn atomic_write(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(text.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| e.error)?;
    Ok(())
}

pub fn default_config_path() -> PathBuf {
    let proj = directories::ProjectDirs::from("dev", "glance", "glance").expect("project dirs");
    proj.config_dir().join("config.toml")
}

/// Pre-rebrand config locations. SettingsStore::load() tries these as a
/// fallback so existing users keep their settings after the mouse-noti →
/// Glance rename. The first save to the new location supersedes them.
fn legacy_config_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(proj) = directories::ProjectDirs::from("dev", "mouse-noti", "mouse-noti") {
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
    fn preference_patch_preserves_menu_and_hotkey_edits() {
        let dir = tempdir().unwrap();
        let store = SettingsStore::new_isolated(dir.path().join("config.toml"));
        store
            .update(|s| {
                s.menu_items.push(MenuItem {
                    id: "app".into(),
                    label: "App".into(),
                    icon: IconSource::Emoji { value: "A".into() },
                    action: Action::OpenUrl {
                        url: "https://example.com".into(),
                    },
                    tags: vec![],
                });
                s.hotkey_bindings.push(HotkeyBinding {
                    id: "key".into(),
                    menu_mode: "all".into(),
                    trigger: HotkeyTrigger::Keyboard {
                        accelerator: "F13".into(),
                    },
                });
            })
            .unwrap();
        store
            .update(|s| {
                PreferencesPatch {
                    indicator_enabled: Some(false),
                    ..Default::default()
                }
                .apply(s)
            })
            .unwrap();
        let saved = store.load();
        assert!(!saved.indicator_enabled);
        assert_eq!(saved.menu_items.len(), 1);
        assert_eq!(saved.hotkey_bindings.len(), 1);
    }

    #[test]
    fn concurrent_edits_preserve_every_binding() {
        let dir = tempdir().unwrap();
        let store =
            std::sync::Arc::new(SettingsStore::new_isolated(dir.path().join("config.toml")));
        let threads: Vec<_> = (0..8)
            .map(|i| {
                let store = store.clone();
                std::thread::spawn(move || {
                    store
                        .update(|s| {
                            s.hotkey_bindings.push(HotkeyBinding {
                                id: i.to_string(),
                                menu_mode: "all".into(),
                                trigger: HotkeyTrigger::Keyboard {
                                    accelerator: "F13".into(),
                                },
                            });
                        })
                        .unwrap()
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(store.load().hotkey_bindings.len(), 8);
    }

    #[test]
    fn corrupt_config_without_backup_is_not_overwritten_by_update() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "corrupt data").unwrap();
        let store = SettingsStore::new_isolated(&path);
        assert!(store.update(|s| s.indicator_enabled = false).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "corrupt data");
    }

    #[test]
    fn failed_replacement_preserves_valid_backup() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let store = SettingsStore::new_isolated(&path);
        store.save(&Settings::default()).unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(store.update(|s| s.indicator_enabled = false).is_err());
        assert!(store.load().indicator_enabled);
    }

    #[test]
    fn onboarding_completion_survives_reload() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let store = SettingsStore::new_isolated(&path);
        assert!(!store.load().onboarding_completed);
        store
            .update(|s| {
                PreferencesPatch {
                    onboarding_completed: Some(true),
                    ..Default::default()
                }
                .apply(s)
            })
            .unwrap();
        assert!(
            SettingsStore::new_isolated(&path)
                .load()
                .onboarding_completed
        );
    }

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
            onboarding_completed: false,
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
            onboarding_completed: false,
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
                icon: IconSource::Emoji {
                    value: "💬".into()
                },
                action: Action::LaunchApp {
                    path: "/Applications/Slack.app".into(),
                },
                tags: vec!["launcher".into()],
            }],
            hotkey_bindings: vec![HotkeyBinding {
                id: "hk-1".into(),
                trigger: HotkeyTrigger::Keyboard {
                    accelerator: "CommandOrControl+Shift+M".into(),
                },
                menu_mode: "all".into(),
            }],
            radial_close_on_leave: false,
            radial_theme: RadialTheme::default(),
            indicator_enabled: true,
            onboarding_completed: false,
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
