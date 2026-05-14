# Phase 2: Radial Menu Implementation Plan

> **For agentic workers:** Implement task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Add a cursor-centered radial menu invoked by a user-configured hotkey (mouse button OR keyboard chord). The menu contains user-defined action items (launch app / open URL / run shell command). Item count is dynamic — sectors are computed at render time from the user's item list (1–12 items supported). Item icons can be emojis or, for `launch_app` actions, the target app's system icon auto-extracted at registration time. Shell actions confirm before execution by default; users can opt out per item.

**Architecture:**

```
┌─────────────────────────────────────────────────────────────┐
│  Tauri App                                                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐    ┌──────────────────┐              │
│  │  Keyboard hotkey │    │   Mouse hotkey   │              │
│  │  (global-shortcut)│   │   (rdev thread)  │              │
│  └────────┬─────────┘    └────────┬─────────┘              │
│           │                       │                         │
│           ▼                       ▼                         │
│  ┌───────────────────────────────────────────┐             │
│  │   HotkeyTrigger event                     │             │
│  │   (cursor_pos, source, recent_noti?)      │             │
│  └────────────┬──────────────────────────────┘             │
│               │                                            │
│               ▼                                            │
│  ┌───────────────────────────────────────────┐             │
│  │  RadialMenu window                        │             │
│  │  - center at cursor                       │             │
│  │  - N sectors from menu items              │             │
│  │  - hover highlight, click → action_id     │             │
│  │  - ESC / outside click → cancel           │             │
│  └────────────┬──────────────────────────────┘             │
│               │ action_id                                  │
│               ▼                                            │
│  ┌───────────────────────────────────────────┐             │
│  │  ActionRunner                             │             │
│  │  - launch_app (NSWorkspace::openApp)      │             │
│  │  - open_url (tauri_plugin_opener)         │             │
│  │  - run_shell (Command::new) + confirm     │             │
│  └───────────────────────────────────────────┘             │
│                                                            │
│  ┌───────────────────────────────────────────┐             │
│  │  Settings store + Editor GUI              │             │
│  │  - menu_items: Vec<MenuItem>              │             │
│  │  - hotkey_bindings: Vec<HotkeyBinding>    │             │
│  │  - shell_confirm default: true            │             │
│  └───────────────────────────────────────────┘             │
└─────────────────────────────────────────────────────────────┘
```

**Tech Stack:**
- Rust core: extends existing `event_bus`, `settings`, `noti` modules. Adds `hotkey`, `action`, `radial` modules.
- New crate: `rdev = "0.5"` (cross-platform low-level mouse + keyboard hook).
- Reuses: `tauri-plugin-global-shortcut` (already in deps), `tauri-plugin-opener` (for URLs), `tauri-plugin-shell` (for `run_shell`), `objc2-app-kit` (for `NSWorkspace.icon(forFile:)`).
- Frontend: React. New components in `src/radial/` and `src/settings/menu/`.

**Related context:**
- Phase 1 MVP (notification indicator) shipped as v0.2.0. Build is stable on macOS.
- `EventBus::recent_within(Duration)` already exists (Task 6 of Phase 1) — used for context-aware menu mode.
- Settings schema is TOML, already has backup recovery in `SettingsStore::save`.

**User decisions (recorded during brainstorming, 2026-05-14):**
- Sector count: dynamic, derived from item count (1–12).
- Icon source: emoji input + auto-extracted system app icon for `launch_app` actions.
- Hotkey source: user-configurable. Both mouse buttons and keyboard chords (Function keys, modifier combos). UI lets user pick either category per binding.
- Shell action confirm: per-item boolean, default `true`.

---

## File Structure

```
mouse-noti/
├── src-tauri/
│   ├── Cargo.toml                         # +rdev, +base64 (icon encoding)
│   └── src/
│       ├── action/
│       │   ├── mod.rs                     # Action enum, ActionRunner, ActionId
│       │   ├── launch_app.rs
│       │   ├── open_url.rs
│       │   └── run_shell.rs
│       ├── hotkey/
│       │   ├── mod.rs                     # HotkeyBinding, HotkeyManager trait
│       │   ├── keyboard.rs                # tauri-plugin-global-shortcut adapter
│       │   └── mouse.rs                   # rdev adapter (separate thread)
│       ├── radial/
│       │   ├── mod.rs                     # window controller, trigger handler
│       │   └── geometry.rs                # sector math (atan2, dynamic count)
│       ├── app_icon.rs                    # NSWorkspace.icon(forFile) → base64 PNG
│       ├── settings.rs                    # extend MenuItem, HotkeyBinding fields
│       ├── commands.rs                    # +menu_item CRUD, +exec_action, +bind_hotkey
│       └── lib.rs                         # wire RadialController, HotkeyManager
└── src/
    ├── radial/
    │   ├── RadialMenu.tsx                 # main radial component
    │   ├── Sector.tsx                     # per-sector rendering
    │   ├── geometry.ts                    # mirror Rust sector math for hover
    │   └── api.ts
    └── settings/
        ├── MenuEditor.tsx                 # list + add/remove + reorder
        ├── MenuItemForm.tsx               # per-item edit form
        ├── IconPicker.tsx                 # emoji input + app picker
        ├── HotkeyEditor.tsx               # capture mouse/keyboard chord
        └── ConfirmDialog.tsx              # shell confirm dialog
```

`hotkey/mouse.rs` runs `rdev::listen` on a dedicated OS thread (rdev is blocking). It posts an event into a `tokio::sync::mpsc` channel that the Tauri runtime drains.

---

## Settings Schema Extension

Extend `Settings` struct in `src-tauri/src/settings.rs` (TOML-compatible).

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Settings {
    pub indicator_style: IndicatorStyle,
    pub autostart: bool,
    #[serde(default)]
    pub menu_items: Vec<MenuItem>,
    #[serde(default)]
    pub hotkey_bindings: Vec<HotkeyBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MenuItem {
    pub id: String,                // uuid v4 string, generated on create
    pub label: String,             // human-readable name shown in menu
    pub icon: IconSource,
    pub action: Action,
    /// Tag set; used by context-aware menu mode (Phase 2.4).
    /// E.g. ["launcher"], ["notification"], or both.
    #[serde(default = "default_tags")]
    pub tags: Vec<String>,
}

fn default_tags() -> Vec<String> { vec!["launcher".into()] }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IconSource {
    Emoji { value: String },
    /// Base64-encoded PNG, extracted at MenuItem registration time.
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
    pub id: String,                // uuid v4
    /// Either a keyboard chord or a mouse-button combo.
    pub trigger: HotkeyTrigger,
    /// When this hotkey fires, which menu mode to show.
    /// "launcher" / "notification" / "all" — filters menu_items by tag.
    #[serde(default = "default_mode")]
    pub menu_mode: String,
}

fn default_mode() -> String { "all".into() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HotkeyTrigger {
    Keyboard {
        // Tauri-plugin-global-shortcut accelerator string,
        // e.g. "CommandOrControl+Shift+M" or "F13".
        accelerator: String,
    },
    Mouse {
        // rdev::Button values mapped to integers.
        // 1=Left, 2=Right, 3=Middle, 4=Side1 (back), 5=Side2 (forward),
        // 6..=Extra. Always with modifiers so we don't intercept normal clicks.
        button: u8,
        // Bitfield of modifiers required: 1=Cmd/Win, 2=Ctrl, 4=Shift, 8=Alt
        modifiers: u8,
    },
}
```

Default `Settings::default()` returns empty `menu_items` and `hotkey_bindings` — first launch has no radial menu, user opens Settings to configure.

A schema migration helper reads any existing v0.2.x config (which has only `indicator_style` + `autostart`) and produces this extended shape on next save via `#[serde(default)]`.

---

## Sub-phase 2.1: Action core + Settings schema + manual exec

### Task 1: Extend settings schema

**Files:**
- Modify: `src-tauri/src/settings.rs`
- Modify: `src-tauri/Cargo.toml` (+ `uuid = { version = "1", features = ["v4"] }`)

- [ ] **Step 1.1: Add uuid dep**

In `[dependencies]`:
```toml
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 1.2: Add the new types to `settings.rs`**

Insert the structs/enums from the "Settings Schema Extension" section above. Replace the existing `Settings` struct in-place (preserve `IndicatorStyle` enum, `SettingsStore` impl, and `default_config_path` fn).

- [ ] **Step 1.3: Add tests for the new schema**

Append to the existing `#[cfg(test)] mod tests` block in `settings.rs`:

```rust
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
    };
    let toml = toml::to_string_pretty(&s).unwrap();
    let parsed: Settings = toml::from_str(&toml).unwrap();
    assert_eq!(parsed, s);
}

#[test]
fn legacy_v0_2_settings_loads_with_defaults() {
    // v0.2.x config: only indicator_style + autostart.
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
```

- [ ] **Step 1.4: Build + test**

```bash
cd /Users/kanghyeon/DEV/mouse-noti
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Expected: clean, 12 tests pass (was 10 + 2 new).

- [ ] **Step 1.5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/settings.rs
git commit -m "feat(settings): add MenuItem, HotkeyBinding, IconSource, Action schema"
```

### Task 2: ActionRunner core (Rust)

**Files:**
- Create: `src-tauri/src/action/mod.rs`
- Create: `src-tauri/src/action/launch_app.rs`
- Create: `src-tauri/src/action/open_url.rs`
- Create: `src-tauri/src/action/run_shell.rs`
- Modify: `src-tauri/src/lib.rs` (add `pub mod action;`)

- [ ] **Step 2.1: action/mod.rs scaffold**

```rust
pub mod launch_app;
pub mod open_url;
pub mod run_shell;

use crate::settings::Action;
use anyhow::Result;

pub struct ActionRunner;

impl ActionRunner {
    /// Execute an action. For shell actions that require confirmation, the
    /// confirm step must be handled BEFORE calling this (frontend
    /// ConfirmDialog gates the call). This method does not prompt.
    pub fn execute(action: &Action) -> Result<()> {
        match action {
            Action::LaunchApp { path } => launch_app::run(path),
            Action::OpenUrl { url } => open_url::run(url),
            Action::RunShell { command, args, .. } => run_shell::run(command, args),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_branches_compile() {
        // Cheap smoke: enum variants reach their handlers.
        // Real execution covered by per-module tests.
        let _ = ActionRunner::execute as fn(&Action) -> Result<()>;
    }
}
```

- [ ] **Step 2.2: action/launch_app.rs (macOS)**

```rust
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn run(path: &str) -> Result<()> {
    let p = PathBuf::from(path);
    if !p.exists() {
        anyhow::bail!("app path does not exist: {}", path);
    }
    // `open` handles .app bundles on macOS without spawning a child shell.
    Command::new("open")
        .arg(&p)
        .status()
        .with_context(|| format!("failed to spawn `open` for {}", path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_path_errors() {
        let err = run("/nonexistent/app.app").unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }
}
```

- [ ] **Step 2.3: action/open_url.rs**

```rust
use anyhow::{Context, Result};
use std::process::Command;

pub fn run(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.contains("://") {
        anyhow::bail!("url must have a scheme: {}", url);
    }
    Command::new("open")
        .arg(url)
        .status()
        .with_context(|| format!("failed to spawn `open` for {}", url))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_schemeless() {
        let err = run("example.com").unwrap_err();
        assert!(err.to_string().contains("scheme"));
    }
}
```

- [ ] **Step 2.4: action/run_shell.rs**

```rust
use anyhow::{Context, Result};
use std::process::Command;

pub fn run(command: &str, args: &[String]) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("shell command cannot be empty");
    }
    Command::new(command)
        .args(args)
        .status()
        .with_context(|| format!("failed to spawn `{}`", command))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_command_errors() {
        let err = run("", &[]).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn echo_runs() {
        // /bin/true is always present on macOS + Linux.
        run("/usr/bin/true", &[]).unwrap();
    }
}
```

- [ ] **Step 2.5: Wire into lib.rs**

Add `pub mod action;` near the other top-level module declarations in `src-tauri/src/lib.rs`.

- [ ] **Step 2.6: Build + test**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Expected: clean, 17 tests pass (12 + 5 new: dispatch_branches_compile, missing_path_errors, rejects_schemeless, empty_command_errors, echo_runs).

- [ ] **Step 2.7: Commit**

```bash
git add src-tauri/src/action src-tauri/src/lib.rs
git commit -m "feat(action): ActionRunner with launch_app/open_url/run_shell"
```

### Task 3: Tauri commands for menu CRUD + action exec

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (register new commands)

- [ ] **Step 3.1: Add CRUD + exec commands**

Append to `src-tauri/src/commands.rs`:

```rust
use crate::action::ActionRunner;
use crate::settings::{MenuItem, HotkeyBinding};

#[tauri::command]
pub fn list_menu_items(store: tauri::State<Arc<SettingsStore>>) -> Vec<MenuItem> {
    store.load().menu_items
}

#[tauri::command]
pub fn upsert_menu_item(
    item: MenuItem,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    let mut settings = store.load();
    if let Some(existing) = settings.menu_items.iter_mut().find(|i| i.id == item.id) {
        *existing = item;
    } else {
        settings.menu_items.push(item);
    }
    store.save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_menu_item(
    item_id: String,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    let mut settings = store.load();
    settings.menu_items.retain(|i| i.id != item_id);
    store.save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_menu_items(
    ids: Vec<String>,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    let mut settings = store.load();
    let mut by_id: std::collections::HashMap<String, MenuItem> = settings
        .menu_items
        .drain(..)
        .map(|i| (i.id.clone(), i))
        .collect();
    let mut reordered = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(item) = by_id.remove(&id) {
            reordered.push(item);
        }
    }
    // Any items not in `ids` are preserved at the tail.
    reordered.extend(by_id.into_values());
    settings.menu_items = reordered;
    store.save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_hotkey_bindings(store: tauri::State<Arc<SettingsStore>>) -> Vec<HotkeyBinding> {
    store.load().hotkey_bindings
}

#[tauri::command]
pub fn upsert_hotkey_binding(
    binding: HotkeyBinding,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    let mut settings = store.load();
    if let Some(existing) = settings.hotkey_bindings.iter_mut().find(|b| b.id == binding.id) {
        *existing = binding;
    } else {
        settings.hotkey_bindings.push(binding);
    }
    store.save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_hotkey_binding(
    binding_id: String,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    let mut settings = store.load();
    settings.hotkey_bindings.retain(|b| b.id != binding_id);
    store.save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn exec_menu_item(
    item_id: String,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    let settings = store.load();
    let item = settings
        .menu_items
        .iter()
        .find(|i| i.id == item_id)
        .ok_or_else(|| format!("menu item not found: {item_id}"))?;
    ActionRunner::execute(&item.action).map_err(|e| e.to_string())
}
```

- [ ] **Step 3.2: Register commands in `invoke_handler`**

Edit `src-tauri/src/lib.rs` and add the seven new commands to BOTH branches of `tauri::generate_handler![...]` (the `mock-os` and non-`mock-os` branches).

- [ ] **Step 3.3: Build + test**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Expected: clean.

- [ ] **Step 3.4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(commands): menu/hotkey CRUD + exec_menu_item"
```

### Task 4: App icon extraction helper

**Files:**
- Create: `src-tauri/src/app_icon.rs`
- Modify: `src-tauri/Cargo.toml` (+ `base64 = "0.22"`)
- Modify: `src-tauri/src/lib.rs` (add `pub mod app_icon;`)
- Modify: `src-tauri/src/commands.rs` (add `extract_app_icon` command)

- [ ] **Step 4.1: Cargo dep**

In `[dependencies]`:
```toml
base64 = "0.22"
```

- [ ] **Step 4.2: app_icon.rs (macOS only)**

```rust
#![cfg(target_os = "macos")]

use anyhow::Result;

/// Load the application icon at `app_path` and return a base64-encoded PNG.
/// Uses NSWorkspace.icon(forFile:) → resize to 64×64 → PNG encode.
pub fn load_app_icon_base64(app_path: &str) -> Result<String> {
    use objc2::{msg_send, runtime::AnyObject, ClassType};
    use objc2_app_kit::{NSBitmapImageRep, NSImage, NSWorkspace};
    use objc2_foundation::{NSData, NSDictionary, NSString};

    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let path_ns = NSString::from_str(app_path);
        let image: objc2::rc::Retained<NSImage> = msg_send![&*workspace, iconForFile: &*path_ns];
        // Get TIFF representation -> NSBitmapImageRep -> PNG.
        let tiff: objc2::rc::Retained<NSData> = msg_send![&*image, TIFFRepresentation];
        let rep: objc2::rc::Retained<NSBitmapImageRep> =
            msg_send![NSBitmapImageRep::class(), imageRepWithData: &*tiff];
        let props = NSDictionary::<NSString, AnyObject>::dictionary();
        // NSBitmapImageFileTypePNG = 4 (deprecated name) / NSBitmapImageFileType::PNG = 4.
        let png: objc2::rc::Retained<NSData> =
            msg_send![&*rep, representationUsingType: 4u64, properties: &*props];
        let bytes = std::slice::from_raw_parts(png.bytes().as_ptr() as *const u8, png.length());
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    }
}
```

If `objc2-app-kit` features need extending (`NSBitmapImageRep`, `NSImage`), add them to the macOS deps block.

For non-macOS:
```rust
#![cfg(not(target_os = "macos"))]

use anyhow::Result;

pub fn load_app_icon_base64(_app_path: &str) -> Result<String> {
    anyhow::bail!("app icon extraction unsupported on this platform");
}
```

(Split into two files via `cfg` or use a single file with both branches.)

- [ ] **Step 4.3: Command**

In `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn extract_app_icon(path: String) -> Result<String, String> {
    crate::app_icon::load_app_icon_base64(&path).map_err(|e| e.to_string())
}
```

Register in `invoke_handler` (both branches).

- [ ] **Step 4.4: Build + test**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Expected: clean.

If `objc2-app-kit` feature names differ from what we used, adapt. If `NSBitmapImageRep::class()` is not in scope, use `Class::get` or `objc2::class!`.

- [ ] **Step 4.5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/app_icon.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(app_icon): NSWorkspace-based icon extraction → base64 PNG"
```

---

## Sub-phase 2.2: Radial menu window + frontend

### Task 5: Radial Tauri window declaration

**Files:**
- Modify: `src-tauri/tauri.conf.json` (add third window: `radial`)
- Modify: `src-tauri/capabilities/default.json` (add `radial` to windows)

- [ ] **Step 5.1: Add window**

In `tauri.conf.json` `app.windows` array, append:
```json
{
  "label": "radial",
  "url": "index.html#/radial",
  "transparent": true,
  "decorations": false,
  "alwaysOnTop": true,
  "visibleOnAllWorkspaces": true,
  "skipTaskbar": true,
  "resizable": false,
  "fullscreen": false,
  "visible": false,
  "shadow": false,
  "focus": true,
  "width": 400,
  "height": 400
}
```

`focus: true` — radial menu MUST take keyboard focus (for ESC handling). Different from overlay window (focus: false).

- [ ] **Step 5.2: Update capabilities**

In `src-tauri/capabilities/default.json` `windows` array, add `"radial"`.

- [ ] **Step 5.3: Verify**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: clean.

- [ ] **Step 5.4: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/capabilities/default.json
git commit -m "feat(radial): declare radial Tauri window"
```

### Task 6: Radial sector geometry (Rust + frontend, mirrored)

**Files:**
- Create: `src-tauri/src/radial/mod.rs`
- Create: `src-tauri/src/radial/geometry.rs`
- Create: `src/radial/geometry.ts`
- Modify: `src-tauri/src/lib.rs` (add `pub mod radial;`)

- [ ] **Step 6.1: Rust geometry**

`src-tauri/src/radial/geometry.rs`:

```rust
//! Map (cursor_dx, cursor_dy) relative to menu center → which sector index
//! is hovered, given `n` sectors arranged clockwise from 12 o'clock.

pub fn sector_at(dx: f64, dy: f64, n: usize, inner_radius: f64, outer_radius: f64) -> Option<usize> {
    if n == 0 { return None; }
    let r = (dx * dx + dy * dy).sqrt();
    if r < inner_radius || r > outer_radius {
        return None;
    }
    // atan2 returns (-PI, PI]. We want 0..2PI starting from 12 o'clock,
    // increasing clockwise. Screen Y is flipped (down is positive), so
    // negate dy for math-coordinate.
    let theta_math = (-dy).atan2(dx); // 0 = right, increases CCW
    // Rotate so 12 o'clock = 0 and CW positive.
    let theta_cw_from_top = (std::f64::consts::FRAC_PI_2 - theta_math)
        .rem_euclid(std::f64::consts::TAU);
    let sector_size = std::f64::consts::TAU / n as f64;
    Some(((theta_cw_from_top / sector_size).floor() as usize) % n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_0_is_top() {
        // Cursor directly above center.
        assert_eq!(sector_at(0.0, -100.0, 4, 20.0, 200.0), Some(0));
    }

    #[test]
    fn sector_1_is_right_for_4() {
        assert_eq!(sector_at(100.0, 0.0, 4, 20.0, 200.0), Some(1));
    }

    #[test]
    fn inner_circle_no_sector() {
        assert_eq!(sector_at(5.0, 5.0, 6, 20.0, 200.0), None);
    }

    #[test]
    fn outer_ring_no_sector() {
        assert_eq!(sector_at(300.0, 0.0, 6, 20.0, 200.0), None);
    }

    #[test]
    fn single_item_always_zero() {
        assert_eq!(sector_at(50.0, 50.0, 1, 20.0, 200.0), Some(0));
    }
}
```

`src-tauri/src/radial/mod.rs` (stub for now, filled later):

```rust
pub mod geometry;
```

- [ ] **Step 6.2: TS mirror**

`src/radial/geometry.ts`:

```ts
export function sectorAt(
  dx: number,
  dy: number,
  n: number,
  innerRadius: number,
  outerRadius: number
): number | null {
  if (n === 0) return null;
  const r = Math.sqrt(dx * dx + dy * dy);
  if (r < innerRadius || r > outerRadius) return null;
  const thetaMath = Math.atan2(-dy, dx);
  const TAU = Math.PI * 2;
  let thetaCw = Math.PI / 2 - thetaMath;
  thetaCw = ((thetaCw % TAU) + TAU) % TAU;
  const sectorSize = TAU / n;
  return Math.floor(thetaCw / sectorSize) % n;
}
```

Add a corresponding vitest case in `src/radial/geometry.test.ts`:

```ts
import { describe, it, expect } from 'vitest';
import { sectorAt } from './geometry';

describe('sectorAt', () => {
  it('top is sector 0 for n=4', () => {
    expect(sectorAt(0, -100, 4, 20, 200)).toBe(0);
  });
  it('right is sector 1 for n=4', () => {
    expect(sectorAt(100, 0, 4, 20, 200)).toBe(1);
  });
  it('inner circle returns null', () => {
    expect(sectorAt(5, 5, 6, 20, 200)).toBeNull();
  });
});
```

- [ ] **Step 6.3: Build + test**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
pnpm test:run
```

Expected: cargo +5 tests (22 total), vitest +3 (10 total).

- [ ] **Step 6.4: Commit**

```bash
git add src-tauri/src/radial src-tauri/src/lib.rs src/radial
git commit -m "feat(radial): sector geometry math (Rust + TS mirror)"
```

### Task 7: Radial frontend UI (manual trigger via DevTools first)

**Files:**
- Create: `src/radial/RadialMenu.tsx`
- Create: `src/radial/Sector.tsx`
- Create: `src/radial/api.ts`
- Modify: `src/App.tsx` (route `#/radial` → `<RadialMenu />`)

- [ ] **Step 7.1: api.ts**

```ts
import { invoke } from '@tauri-apps/api/core';
import type { MenuItem } from '../settings/api';  // extend with MenuItem type

export const listMenuItems = () => invoke<MenuItem[]>('list_menu_items');
export const execMenuItem = (itemId: string) =>
  invoke<void>('exec_menu_item', { itemId });
```

Also create `src/types.ts` with the shared types (mirror of the Rust schema):

```ts
export type IndicatorStyle = 'ring_pulse' | 'icon_badge' | 'persistent_badge';

export type IconSource =
  | { kind: 'emoji'; value: string }
  | { kind: 'app_icon_png'; base64: string; source_path: string };

export type Action =
  | { kind: 'launch_app'; path: string }
  | { kind: 'open_url'; url: string }
  | { kind: 'run_shell'; command: string; args: string[]; confirm: boolean };

export type MenuItem = {
  id: string;
  label: string;
  icon: IconSource;
  action: Action;
  tags: string[];
};

export type HotkeyTrigger =
  | { kind: 'keyboard'; accelerator: string }
  | { kind: 'mouse'; button: number; modifiers: number };

export type HotkeyBinding = {
  id: string;
  trigger: HotkeyTrigger;
  menu_mode: string; // "all" | "launcher" | "notification"
};
```

Re-export from both `settings/api.ts` and `radial/api.ts` for DRY.

- [ ] **Step 7.2: Sector.tsx**

```tsx
import type { MenuItem } from '../types';

type Props = {
  item: MenuItem;
  index: number;
  total: number;
  hovered: boolean;
  innerRadius: number;
  outerRadius: number;
};

export default function Sector({ item, index, total, hovered, innerRadius, outerRadius }: Props) {
  const sectorAngle = (2 * Math.PI) / total;
  const startAngle = -Math.PI / 2 + index * sectorAngle;
  const endAngle = startAngle + sectorAngle;

  // SVG arc path (donut sector).
  const x1 = Math.cos(startAngle) * outerRadius;
  const y1 = Math.sin(startAngle) * outerRadius;
  const x2 = Math.cos(endAngle) * outerRadius;
  const y2 = Math.sin(endAngle) * outerRadius;
  const x3 = Math.cos(endAngle) * innerRadius;
  const y3 = Math.sin(endAngle) * innerRadius;
  const x4 = Math.cos(startAngle) * innerRadius;
  const y4 = Math.sin(startAngle) * innerRadius;
  const largeArc = sectorAngle > Math.PI ? 1 : 0;

  const path = [
    `M ${x1} ${y1}`,
    `A ${outerRadius} ${outerRadius} 0 ${largeArc} 1 ${x2} ${y2}`,
    `L ${x3} ${y3}`,
    `A ${innerRadius} ${innerRadius} 0 ${largeArc} 0 ${x4} ${y4}`,
    'Z',
  ].join(' ');

  // Icon center (mid-radius, mid-angle).
  const iconAngle = (startAngle + endAngle) / 2;
  const iconR = (innerRadius + outerRadius) / 2;
  const iconX = Math.cos(iconAngle) * iconR;
  const iconY = Math.sin(iconAngle) * iconR;

  return (
    <g>
      <path
        d={path}
        fill={hovered ? 'rgba(96,165,250,0.85)' : 'rgba(31,41,55,0.85)'}
        stroke="rgba(255,255,255,0.2)"
        strokeWidth={1}
      />
      <foreignObject x={iconX - 20} y={iconY - 20} width={40} height={40} pointerEvents="none">
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', width: 40, height: 40, fontSize: 24 }}>
          {item.icon.kind === 'emoji' ? (
            <span>{item.icon.value}</span>
          ) : (
            <img src={`data:image/png;base64,${item.icon.base64}`} width={32} height={32} alt="" />
          )}
        </div>
      </foreignObject>
      <text
        x={iconX}
        y={iconY + 28}
        textAnchor="middle"
        fill="white"
        fontSize={10}
        pointerEvents="none"
      >
        {item.label}
      </text>
    </g>
  );
}
```

- [ ] **Step 7.3: RadialMenu.tsx**

```tsx
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { sectorAt } from './geometry';
import Sector from './Sector';
import { listMenuItems, execMenuItem } from './api';
import type { MenuItem } from '../types';

const CENTER = 200;
const INNER = 50;
const OUTER = 180;

type Trigger = { cursor_x: number; cursor_y: number };

export default function RadialMenu() {
  const [items, setItems] = useState<MenuItem[]>([]);
  const [hovered, setHovered] = useState<number | null>(null);

  useEffect(() => {
    listMenuItems().then(setItems);
    const un = listen<Trigger>('radial:show', () => {
      // window has just been shown by Rust; mouse events arrive here.
    });
    const escListener = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        getCurrentWebviewWindow().hide();
      }
    };
    window.addEventListener('keydown', escListener);
    return () => {
      un.then((f) => f());
      window.removeEventListener('keydown', escListener);
    };
  }, []);

  const onMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = (e.currentTarget as SVGSVGElement).getBoundingClientRect();
    const dx = e.clientX - rect.left - CENTER;
    const dy = e.clientY - rect.top - CENTER;
    setHovered(sectorAt(dx, dy, items.length, INNER, OUTER));
  };

  const onClick = () => {
    if (hovered === null) return;
    const item = items[hovered];
    if (!item) return;
    execMenuItem(item.id);
    getCurrentWebviewWindow().hide();
  };

  return (
    <svg
      viewBox={`0 0 ${CENTER * 2} ${CENTER * 2}`}
      width={CENTER * 2}
      height={CENTER * 2}
      onMouseMove={onMouseMove}
      onClick={onClick}
      style={{ position: 'fixed', top: 0, left: 0 }}
    >
      <g transform={`translate(${CENTER} ${CENTER})`}>
        {items.map((item, i) => (
          <Sector
            key={item.id}
            item={item}
            index={i}
            total={items.length}
            hovered={hovered === i}
            innerRadius={INNER}
            outerRadius={OUTER}
          />
        ))}
      </g>
    </svg>
  );
}
```

- [ ] **Step 7.4: Route**

In `src/App.tsx`, add a branch:
```tsx
if (route.startsWith('#/radial')) return <RadialMenu />;
```

- [ ] **Step 7.5: Rust window controller**

Replace `src-tauri/src/radial/mod.rs`:

```rust
pub mod geometry;

use crate::event_bus::EventBus;
use crate::overlay::cursor;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

#[derive(serde::Serialize, Clone)]
pub struct ShowPayload {
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub menu_mode: String,
    pub recent_app_name: Option<String>,
}

pub async fn show<R: Runtime>(app: &AppHandle<R>, bus: &EventBus, menu_mode: &str) {
    let Some(pos) = cursor::current_position() else { return; };
    let Some(win) = app.get_webview_window("radial") else { return; };

    // Center the radial window on cursor. Window is 400x400 logical.
    let scale = win.scale_factor().unwrap_or(1.0);
    let half_w = 200.0 * scale;
    let half_h = 200.0 * scale;
    let _ = win.set_position(tauri::PhysicalPosition::new(
        (pos.0 - half_w) as i32,
        (pos.1 - half_h) as i32,
    ));
    let _ = win.set_size(tauri::PhysicalSize::new((400.0 * scale) as u32, (400.0 * scale) as u32));
    let _ = win.set_visible_on_all_workspaces(true);
    let _ = win.set_always_on_top(true);
    let _ = win.show();
    let _ = win.set_focus();

    tokio::time::sleep(Duration::from_millis(30)).await;

    let recent = bus.recent_within(Duration::from_secs(5));
    let recent_app_name = recent.last().map(|e| e.app_name.clone());

    let payload = ShowPayload {
        cursor_x: pos.0,
        cursor_y: pos.1,
        menu_mode: menu_mode.to_string(),
        recent_app_name,
    };
    let _ = app.emit("radial:show", payload);
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("radial") {
        let _ = win.hide();
    }
}
```

- [ ] **Step 7.6: Manual trigger command (temporary, removed in Task 9)**

Add to `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub async fn debug_show_radial(
    app: tauri::AppHandle,
    bus: tauri::State<'_, crate::event_bus::EventBus>,
) -> Result<(), String> {
    crate::radial::show(&app, &bus, "all").await;
    Ok(())
}

#[tauri::command]
pub fn debug_hide_radial(app: tauri::AppHandle) {
    crate::radial::hide(&app);
}
```

Register in both branches of `invoke_handler`. Tracking: these two commands are removed in Task 9 once the hotkey listener wires the real trigger.

- [ ] **Step 7.7: Build + test**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
pnpm test:run
```

Expected: clean.

- [ ] **Step 7.8: Manual smoke**

Run `pnpm tauri dev`. Open settings DevTools console (right-click settings window). Run:
```js
await window.__TAURI__.core.invoke('debug_show_radial');
```

Expected: radial menu appears at cursor. Hovering over sectors highlights them. Clicking a sector either executes (if items exist) or just closes (empty state).

Add a few menu items by hand via:
```js
await window.__TAURI__.core.invoke('upsert_menu_item', {
  item: {
    id: '1',
    label: 'Slack',
    icon: { kind: 'emoji', value: '💬' },
    action: { kind: 'launch_app', path: '/Applications/Slack.app' },
    tags: ['launcher'],
  },
});
```

Then re-trigger the radial menu and confirm Slack appears.

ESC closes the menu.

- [ ] **Step 7.9: Commit**

```bash
git add src-tauri/src/radial src-tauri/src/commands.rs src-tauri/src/lib.rs src/radial src/App.tsx src/types.ts src/settings/api.ts
git commit -m "feat(radial): cursor-centered menu with sector geometry + manual debug trigger"
```

---

## Sub-phase 2.3: Hotkey listener

### Task 8: Keyboard hotkey listener

**Files:**
- Create: `src-tauri/src/hotkey/mod.rs`
- Create: `src-tauri/src/hotkey/keyboard.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 8.1: hotkey/mod.rs**

```rust
pub mod keyboard;

use crate::settings::{HotkeyBinding, HotkeyTrigger};

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub binding_id: String,
    pub menu_mode: String,
}
```

- [ ] **Step 8.2: hotkey/keyboard.rs**

```rust
use crate::hotkey::TriggerEvent;
use crate::settings::{HotkeyBinding, HotkeyTrigger};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Register all keyboard hotkey bindings from settings. Returns a list of
/// (binding_id, error) for any that failed to register (already taken, etc.).
pub fn register_all<R: Runtime>(
    app: &AppHandle<R>,
    bindings: &[HotkeyBinding],
    tx: tokio::sync::mpsc::UnboundedSender<TriggerEvent>,
) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    let shortcut_ext = app.global_shortcut();
    for binding in bindings {
        if let HotkeyTrigger::Keyboard { accelerator } = &binding.trigger {
            match accelerator.parse::<Shortcut>() {
                Ok(sc) => {
                    let tx = tx.clone();
                    let binding_id = binding.id.clone();
                    let menu_mode = binding.menu_mode.clone();
                    let handler = move |_app: &AppHandle<R>, event: &Shortcut, state: ShortcutState| {
                        if state == ShortcutState::Pressed {
                            let _ = tx.send(TriggerEvent {
                                binding_id: binding_id.clone(),
                                menu_mode: menu_mode.clone(),
                            });
                        }
                    };
                    if let Err(e) = shortcut_ext.on_shortcut(sc, handler) {
                        failures.push((binding.id.clone(), e.to_string()));
                    }
                }
                Err(e) => failures.push((binding.id.clone(), e.to_string())),
            }
        }
    }
    failures
}

pub fn unregister_all<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.global_shortcut().unregister_all();
}
```

(Tauri's `on_shortcut` API signature may differ across point releases — adapt as needed. The intent: register each parsed `Shortcut` and route presses to `tx`.)

- [ ] **Step 8.3: Wire in lib.rs**

In `pub fn run()`, before/around `start_os_source`:

```rust
let (hotkey_tx, mut hotkey_rx) = tokio::sync::mpsc::unbounded_channel::<hotkey::TriggerEvent>();
// Inside .setup(|app| { ... })
let bindings = store.load().hotkey_bindings.clone();
let failures = hotkey::keyboard::register_all(app.handle(), &bindings, hotkey_tx.clone());
for (id, err) in &failures {
    tracing::warn!(binding_id = %id, error = %err, "failed to register keyboard hotkey");
}

// Drain trigger events; on each, show the radial menu.
let app_handle = app.handle().clone();
let bus_clone = bus.clone();
tauri::async_runtime::spawn(async move {
    while let Some(event) = hotkey_rx.recv().await {
        tracing::info!(?event, "hotkey fired");
        crate::radial::show(&app_handle, &bus_clone, &event.menu_mode).await;
    }
});
```

(The existing `let debug_shortcut = Shortcut::new(...)` block in lib.rs — replace its handler logic with sending to the same channel, OR remove the block entirely now that bindings come from settings.)

- [ ] **Step 8.4: Build + test**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Expected: clean.

- [ ] **Step 8.5: Manual smoke**

Register a binding via DevTools:
```js
await window.__TAURI__.core.invoke('upsert_hotkey_binding', {
  binding: {
    id: 'kb-1',
    trigger: { kind: 'keyboard', accelerator: 'CommandOrControl+Shift+M' },
    menu_mode: 'all',
  },
});
```

Restart app. Press the accelerator. Radial menu should open at cursor.

- [ ] **Step 8.6: Commit**

```bash
git add src-tauri/src/hotkey src-tauri/src/lib.rs
git commit -m "feat(hotkey): keyboard binding via global-shortcut"
```

### Task 9: Mouse hotkey listener (rdev)

**Files:**
- Modify: `src-tauri/Cargo.toml` (+ `rdev = "0.5"`)
- Create: `src-tauri/src/hotkey/mouse.rs`
- Modify: `src-tauri/src/hotkey/mod.rs` (re-export)
- Modify: `src-tauri/src/lib.rs` (start mouse listener thread)

- [ ] **Step 9.1: rdev dep**

In `[dependencies]`:
```toml
rdev = "0.5"
```

- [ ] **Step 9.2: hotkey/mouse.rs**

```rust
use crate::hotkey::TriggerEvent;
use crate::settings::{HotkeyBinding, HotkeyTrigger};
use parking_lot::Mutex;
use rdev::{listen, Button, Event, EventType};
use std::sync::Arc;

/// Modifier bits: 1=Cmd/Win, 2=Ctrl, 4=Shift, 8=Alt.
#[derive(Default, Clone, Copy)]
struct ModState {
    cmd: bool,
    ctrl: bool,
    shift: bool,
    alt: bool,
}

impl ModState {
    fn bits(&self) -> u8 {
        (self.cmd as u8) | ((self.ctrl as u8) << 1) | ((self.shift as u8) << 2) | ((self.alt as u8) << 3)
    }
}

fn button_to_u8(b: Button) -> u8 {
    match b {
        Button::Left => 1,
        Button::Right => 2,
        Button::Middle => 3,
        Button::Unknown(n) => 3 + n,
    }
}

/// Spawn a dedicated OS thread that listens for global mouse events.
/// When a button matching any binding fires with the right modifier set,
/// send a TriggerEvent.
pub fn spawn_listener(
    bindings: Vec<HotkeyBinding>,
    tx: tokio::sync::mpsc::UnboundedSender<TriggerEvent>,
) {
    std::thread::spawn(move || {
        let mods: Arc<Mutex<ModState>> = Arc::new(Mutex::new(ModState::default()));
        let mods_for_cb = mods.clone();
        let bindings = bindings.clone();
        if let Err(e) = listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(k) => {
                    let mut m = mods_for_cb.lock();
                    update_mod(&mut m, k, true);
                }
                EventType::KeyRelease(k) => {
                    let mut m = mods_for_cb.lock();
                    update_mod(&mut m, k, false);
                }
                EventType::ButtonPress(b) => {
                    let pressed_button = button_to_u8(b);
                    let current_mods = mods_for_cb.lock().bits();
                    for binding in &bindings {
                        if let HotkeyTrigger::Mouse { button, modifiers } = &binding.trigger {
                            if *button == pressed_button && *modifiers == current_mods {
                                let _ = tx.send(TriggerEvent {
                                    binding_id: binding.id.clone(),
                                    menu_mode: binding.menu_mode.clone(),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }) {
            tracing::error!(error = ?e, "rdev::listen failed");
        }
    });
}

fn update_mod(m: &mut ModState, k: rdev::Key, pressed: bool) {
    use rdev::Key;
    match k {
        Key::MetaLeft | Key::MetaRight => m.cmd = pressed,
        Key::ControlLeft | Key::ControlRight => m.ctrl = pressed,
        Key::ShiftLeft | Key::ShiftRight => m.shift = pressed,
        Key::Alt | Key::AltGr => m.alt = pressed,
        _ => {}
    }
}
```

- [ ] **Step 9.3: Wire in lib.rs**

After keyboard register_all:
```rust
let mouse_bindings = bindings.clone();
hotkey::mouse::spawn_listener(mouse_bindings, hotkey_tx.clone());
```

- [ ] **Step 9.4: Build**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Adapt if rdev API names differ.

- [ ] **Step 9.5: Manual smoke**

Bind side-button (button=4) + Cmd:
```js
await window.__TAURI__.core.invoke('upsert_hotkey_binding', {
  binding: {
    id: 'm-1',
    trigger: { kind: 'mouse', button: 4, modifiers: 1 },
    menu_mode: 'all',
  },
});
```

Restart app. Hold Cmd, press mouse side button → radial menu opens.

- [ ] **Step 9.6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/hotkey
git commit -m "feat(hotkey): mouse button bindings via rdev"
```

### Task 10: Remove debug commands

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 10.1: Remove `debug_show_radial` + `debug_hide_radial`**

Delete the two `#[tauri::command]` blocks from `commands.rs`. Remove from `invoke_handler` lists in both branches.

- [ ] **Step 10.2: Build + test**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

- [ ] **Step 10.3: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "chore(commands): drop debug radial trigger commands"
```

---

## Sub-phase 2.4: Editor GUI + Polish

### Task 11: Menu editor (Settings GUI)

**Files:**
- Create: `src/settings/menu/MenuEditor.tsx`
- Create: `src/settings/menu/MenuItemForm.tsx`
- Create: `src/settings/menu/IconPicker.tsx`
- Create: `src/settings/menu/MenuPreview.tsx`
- Modify: `src/settings/Settings.tsx` (add a "Menu Items" section)
- Extend: `src/settings/api.ts` with menu CRUD wrappers + types

- [ ] **Step 11.1: api.ts CRUD wrappers**

```ts
import { invoke } from '@tauri-apps/api/core';
import type { MenuItem, HotkeyBinding } from '../types';

export const listMenuItems = () => invoke<MenuItem[]>('list_menu_items');
export const upsertMenuItem = (item: MenuItem) =>
  invoke<void>('upsert_menu_item', { item });
export const deleteMenuItem = (itemId: string) =>
  invoke<void>('delete_menu_item', { itemId });
export const reorderMenuItems = (ids: string[]) =>
  invoke<void>('reorder_menu_items', { ids });
export const extractAppIcon = (path: string) =>
  invoke<string>('extract_app_icon', { path });
```

- [ ] **Step 11.2: IconPicker.tsx**

```tsx
import { useState } from 'react';
import type { IconSource } from '../../types';
import { extractAppIcon } from '../api';

type Props = {
  value: IconSource;
  appPath?: string;
  onChange: (icon: IconSource) => void;
};

export default function IconPicker({ value, appPath, onChange }: Props) {
  const [busy, setBusy] = useState(false);

  const tryExtractAppIcon = async () => {
    if (!appPath) return;
    setBusy(true);
    try {
      const base64 = await extractAppIcon(appPath);
      onChange({ kind: 'app_icon_png', base64, source_path: appPath });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <label className="text-sm font-medium">Icon (emoji)</label>
        <input
          type="text"
          maxLength={4}
          className="border rounded px-2 py-1 w-16 text-center"
          value={value.kind === 'emoji' ? value.value : ''}
          onChange={(e) => onChange({ kind: 'emoji', value: e.target.value })}
        />
      </div>
      {appPath && (
        <button
          type="button"
          className="px-2 py-1 bg-gray-100 rounded text-sm"
          onClick={tryExtractAppIcon}
          disabled={busy}
        >
          {busy ? 'Extracting…' : 'Use app icon'}
        </button>
      )}
      {value.kind === 'app_icon_png' && (
        <div className="text-xs text-gray-500">
          App icon from <code>{value.source_path}</code>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 11.3: MenuItemForm.tsx**

```tsx
import { useState } from 'react';
import type { MenuItem, Action } from '../../types';
import IconPicker from './IconPicker';

type Props = {
  initial: MenuItem;
  onSave: (item: MenuItem) => void;
  onCancel: () => void;
};

export default function MenuItemForm({ initial, onSave, onCancel }: Props) {
  const [item, setItem] = useState<MenuItem>(initial);
  const appPath =
    item.action.kind === 'launch_app' ? (item.action as { path: string }).path : undefined;

  const updateAction = (a: Action) => setItem({ ...item, action: a });

  return (
    <div className="space-y-3 border p-4 rounded">
      <div>
        <label className="text-sm font-medium">Label</label>
        <input
          type="text"
          className="border rounded px-2 py-1 w-full"
          value={item.label}
          onChange={(e) => setItem({ ...item, label: e.target.value })}
        />
      </div>
      <div>
        <label className="text-sm font-medium">Action type</label>
        <select
          className="border rounded px-2 py-1"
          value={item.action.kind}
          onChange={(e) => {
            const kind = e.target.value as Action['kind'];
            if (kind === 'launch_app') updateAction({ kind: 'launch_app', path: '' });
            else if (kind === 'open_url') updateAction({ kind: 'open_url', url: '' });
            else updateAction({ kind: 'run_shell', command: '', args: [], confirm: true });
          }}
        >
          <option value="launch_app">Launch app</option>
          <option value="open_url">Open URL</option>
          <option value="run_shell">Run shell command</option>
        </select>
      </div>
      {item.action.kind === 'launch_app' && (
        <input
          type="text"
          placeholder="/Applications/Slack.app"
          className="border rounded px-2 py-1 w-full"
          value={item.action.path}
          onChange={(e) => updateAction({ kind: 'launch_app', path: e.target.value })}
        />
      )}
      {item.action.kind === 'open_url' && (
        <input
          type="text"
          placeholder="https://example.com"
          className="border rounded px-2 py-1 w-full"
          value={item.action.url}
          onChange={(e) => updateAction({ kind: 'open_url', url: e.target.value })}
        />
      )}
      {item.action.kind === 'run_shell' && (
        <div className="space-y-2">
          <input
            type="text"
            placeholder="/usr/local/bin/foo"
            className="border rounded px-2 py-1 w-full"
            value={item.action.command}
            onChange={(e) =>
              updateAction({ ...item.action, command: e.target.value } as Action)
            }
          />
          <input
            type="text"
            placeholder="args (space-separated)"
            className="border rounded px-2 py-1 w-full"
            value={item.action.args.join(' ')}
            onChange={(e) =>
              updateAction({
                ...item.action,
                args: e.target.value.split(/\s+/).filter((a) => a.length > 0),
              } as Action)
            }
          />
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={item.action.confirm}
              onChange={(e) =>
                updateAction({ ...item.action, confirm: e.target.checked } as Action)
              }
            />
            Confirm before running
          </label>
        </div>
      )}
      <IconPicker
        value={item.icon}
        appPath={appPath}
        onChange={(icon) => setItem({ ...item, icon })}
      />
      <div className="flex gap-2">
        <button
          type="button"
          className="px-3 py-1 bg-blue-600 text-white rounded"
          onClick={() => onSave(item)}
        >
          Save
        </button>
        <button
          type="button"
          className="px-3 py-1 bg-gray-200 rounded"
          onClick={onCancel}
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 11.4: MenuEditor.tsx**

```tsx
import { useEffect, useState } from 'react';
import {
  listMenuItems,
  upsertMenuItem,
  deleteMenuItem,
  reorderMenuItems,
} from '../api';
import type { MenuItem } from '../../types';
import MenuItemForm from './MenuItemForm';

const newItem = (): MenuItem => ({
  id: crypto.randomUUID(),
  label: 'New item',
  icon: { kind: 'emoji', value: '⚡' },
  action: { kind: 'launch_app', path: '' },
  tags: ['launcher'],
});

export default function MenuEditor() {
  const [items, setItems] = useState<MenuItem[]>([]);
  const [editing, setEditing] = useState<MenuItem | null>(null);

  const refresh = () => listMenuItems().then(setItems);
  useEffect(() => {
    refresh();
  }, []);

  const onSave = async (item: MenuItem) => {
    await upsertMenuItem(item);
    setEditing(null);
    refresh();
  };

  const onDelete = async (id: string) => {
    await deleteMenuItem(id);
    refresh();
  };

  const moveUp = async (i: number) => {
    if (i === 0) return;
    const next = items.slice();
    [next[i - 1], next[i]] = [next[i], next[i - 1]];
    await reorderMenuItems(next.map((n) => n.id));
    refresh();
  };

  const moveDown = async (i: number) => {
    if (i === items.length - 1) return;
    const next = items.slice();
    [next[i + 1], next[i]] = [next[i], next[i + 1]];
    await reorderMenuItems(next.map((n) => n.id));
    refresh();
  };

  return (
    <section className="space-y-3">
      <h2 className="text-sm font-semibold">Menu items</h2>
      {editing ? (
        <MenuItemForm
          initial={editing}
          onSave={onSave}
          onCancel={() => setEditing(null)}
        />
      ) : (
        <button
          type="button"
          className="px-3 py-1 bg-gray-200 rounded text-sm"
          onClick={() => setEditing(newItem())}
        >
          + Add item
        </button>
      )}
      <ul className="space-y-1">
        {items.map((it, i) => (
          <li
            key={it.id}
            className="flex items-center justify-between border rounded p-2"
          >
            <div className="flex items-center gap-2">
              <span className="text-xl">
                {it.icon.kind === 'emoji' ? (
                  it.icon.value
                ) : (
                  <img src={`data:image/png;base64,${it.icon.base64}`} width={24} height={24} alt="" />
                )}
              </span>
              <span>{it.label}</span>
            </div>
            <div className="flex gap-1">
              <button type="button" className="px-2 text-sm" onClick={() => moveUp(i)}>↑</button>
              <button type="button" className="px-2 text-sm" onClick={() => moveDown(i)}>↓</button>
              <button type="button" className="px-2 text-sm" onClick={() => setEditing(it)}>Edit</button>
              <button type="button" className="px-2 text-sm text-red-600" onClick={() => onDelete(it.id)}>Delete</button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
```

- [ ] **Step 11.5: Settings.tsx — mount editor**

In `src/settings/Settings.tsx`, import `MenuEditor` and add it to the rendered tree alongside `StylePicker`, `PermissionPanel`, `AutoStartToggle`.

- [ ] **Step 11.6: Run tests**

```bash
pnpm test:run
```

Expected: all green. No new tests required for these UI components in this task (Task 13 covers regression test for save/reorder integration).

- [ ] **Step 11.7: Commit**

```bash
git add src/settings src/types.ts
git commit -m "feat(settings): MenuEditor + MenuItemForm + IconPicker UI"
```

### Task 12: Hotkey editor (Settings GUI)

**Files:**
- Create: `src/settings/hotkey/HotkeyEditor.tsx`
- Create: `src/settings/hotkey/HotkeyCapture.tsx`
- Modify: `src/settings/Settings.tsx`

- [ ] **Step 12.1: HotkeyCapture.tsx**

Component that, on click, listens for the next mouse press OR keyboard chord and returns a `HotkeyTrigger`. Uses native browser events; chord recording cycles through pressed modifiers + main key on press.

```tsx
import { useEffect, useState } from 'react';
import type { HotkeyTrigger } from '../../types';

type Props = {
  value: HotkeyTrigger;
  onChange: (v: HotkeyTrigger) => void;
};

const MOD_CMD = 1, MOD_CTRL = 2, MOD_SHIFT = 4, MOD_ALT = 8;

export default function HotkeyCapture({ value, onChange }: Props) {
  const [capturing, setCapturing] = useState(false);
  const [mode, setMode] = useState<'keyboard' | 'mouse'>(
    value.kind === 'mouse' ? 'mouse' : 'keyboard'
  );

  useEffect(() => {
    if (!capturing) return;
    const onKey = (e: KeyboardEvent) => {
      if (mode !== 'keyboard') return;
      if (['Meta', 'Control', 'Shift', 'Alt'].includes(e.key)) return; // wait for non-modifier
      e.preventDefault();
      const parts: string[] = [];
      if (e.metaKey) parts.push('CommandOrControl');
      if (e.ctrlKey && !e.metaKey) parts.push('CommandOrControl');
      if (e.shiftKey) parts.push('Shift');
      if (e.altKey) parts.push('Alt');
      parts.push(e.key.toUpperCase());
      onChange({ kind: 'keyboard', accelerator: parts.join('+') });
      setCapturing(false);
    };
    const onMouseDown = (e: MouseEvent) => {
      if (mode !== 'mouse') return;
      // Map browser button: 0=left, 1=middle, 2=right, 3=back, 4=forward.
      // Translate to rdev convention: 1=left, 2=right, 3=middle, 4=back, 5=forward.
      const map = [1, 3, 2, 4, 5];
      const button = map[e.button] ?? 0;
      if (button === 0) return;
      e.preventDefault();
      let modifiers = 0;
      if (e.metaKey) modifiers |= MOD_CMD;
      if (e.ctrlKey) modifiers |= MOD_CTRL;
      if (e.shiftKey) modifiers |= MOD_SHIFT;
      if (e.altKey) modifiers |= MOD_ALT;
      onChange({ kind: 'mouse', button, modifiers });
      setCapturing(false);
    };
    window.addEventListener('keydown', onKey);
    window.addEventListener('mousedown', onMouseDown);
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('mousedown', onMouseDown);
    };
  }, [capturing, mode, onChange]);

  return (
    <div className="space-y-2">
      <div className="flex gap-2">
        <button
          type="button"
          className={`px-2 py-1 rounded ${mode === 'keyboard' ? 'bg-blue-100' : 'bg-gray-100'}`}
          onClick={() => setMode('keyboard')}
        >
          Keyboard
        </button>
        <button
          type="button"
          className={`px-2 py-1 rounded ${mode === 'mouse' ? 'bg-blue-100' : 'bg-gray-100'}`}
          onClick={() => setMode('mouse')}
        >
          Mouse
        </button>
      </div>
      <div className="border rounded p-2 flex items-center justify-between">
        <code>
          {value.kind === 'keyboard'
            ? value.accelerator || '(none)'
            : `Mouse ${value.button} + mods=${value.modifiers}`}
        </code>
        <button
          type="button"
          className="px-2 py-1 bg-blue-600 text-white rounded text-sm"
          onClick={() => setCapturing(true)}
        >
          {capturing ? 'Press combo…' : 'Capture'}
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 12.2: api.ts CRUD wrappers (extend)**

Add to `src/settings/api.ts`:
```ts
import type { HotkeyBinding } from '../types';

export const listHotkeyBindings = () => invoke<HotkeyBinding[]>('list_hotkey_bindings');
export const upsertHotkeyBinding = (binding: HotkeyBinding) =>
  invoke<void>('upsert_hotkey_binding', { binding });
export const deleteHotkeyBinding = (bindingId: string) =>
  invoke<void>('delete_hotkey_binding', { bindingId });
```

- [ ] **Step 12.3: HotkeyEditor.tsx**

```tsx
import { useEffect, useState } from 'react';
import type { HotkeyBinding, HotkeyTrigger } from '../../types';
import {
  listHotkeyBindings,
  upsertHotkeyBinding,
  deleteHotkeyBinding,
} from '../api';
import HotkeyCapture from './HotkeyCapture';

const newBinding = (): HotkeyBinding => ({
  id: crypto.randomUUID(),
  trigger: { kind: 'keyboard', accelerator: '' },
  menu_mode: 'all',
});

export default function HotkeyEditor() {
  const [bindings, setBindings] = useState<HotkeyBinding[]>([]);
  const [editing, setEditing] = useState<HotkeyBinding | null>(null);

  const refresh = () => listHotkeyBindings().then(setBindings);
  useEffect(() => {
    refresh();
  }, []);

  const onSave = async (b: HotkeyBinding) => {
    await upsertHotkeyBinding(b);
    setEditing(null);
    refresh();
  };

  const onDelete = async (id: string) => {
    await deleteHotkeyBinding(id);
    refresh();
  };

  return (
    <section className="space-y-3">
      <h2 className="text-sm font-semibold">Hotkey bindings</h2>
      {editing ? (
        <div className="border p-3 rounded space-y-2">
          <HotkeyCapture
            value={editing.trigger}
            onChange={(t) => setEditing({ ...editing, trigger: t })}
          />
          <div>
            <label className="text-sm font-medium">Menu mode</label>
            <select
              className="border rounded px-2 py-1 ml-2"
              value={editing.menu_mode}
              onChange={(e) => setEditing({ ...editing, menu_mode: e.target.value })}
            >
              <option value="all">All items</option>
              <option value="launcher">Launcher only</option>
              <option value="notification">Notification actions only</option>
            </select>
          </div>
          <div className="flex gap-2">
            <button
              type="button"
              className="px-3 py-1 bg-blue-600 text-white rounded"
              onClick={() => onSave(editing)}
            >
              Save
            </button>
            <button
              type="button"
              className="px-3 py-1 bg-gray-200 rounded"
              onClick={() => setEditing(null)}
            >
              Cancel
            </button>
          </div>
        </div>
      ) : (
        <button
          type="button"
          className="px-3 py-1 bg-gray-200 rounded text-sm"
          onClick={() => setEditing(newBinding())}
        >
          + Add binding
        </button>
      )}
      <ul className="space-y-1">
        {bindings.map((b) => (
          <li
            key={b.id}
            className="flex items-center justify-between border rounded p-2"
          >
            <code className="text-sm">
              {b.trigger.kind === 'keyboard'
                ? b.trigger.accelerator
                : `Mouse ${b.trigger.button} + mods=${b.trigger.modifiers}`}
              {' → '}
              {b.menu_mode}
            </code>
            <div className="flex gap-1">
              <button type="button" className="px-2 text-sm" onClick={() => setEditing(b)}>Edit</button>
              <button type="button" className="px-2 text-sm text-red-600" onClick={() => onDelete(b.id)}>Delete</button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
```

- [ ] **Step 12.4: Mount in Settings.tsx**

Add `<HotkeyEditor />` below `<MenuEditor />`.

- [ ] **Step 12.5: Run tests + commit**

```bash
pnpm test:run
git add src/settings/hotkey src/settings/Settings.tsx
git commit -m "feat(settings): HotkeyEditor with capture UI"
```

### Task 13: Live re-registration after binding changes

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 13.1: Shared bindings state**

Create a new type and managed state. Add to `src-tauri/src/hotkey/mod.rs`:

```rust
use crate::settings::HotkeyBinding;
use parking_lot::Mutex;
use std::sync::Arc;

pub type SharedBindings = Arc<Mutex<Vec<HotkeyBinding>>>;
```

In `src-tauri/src/lib.rs`, construct it from initial settings BEFORE registering hotkeys:

```rust
let shared_bindings: hotkey::SharedBindings =
    Arc::new(Mutex::new(store.load().hotkey_bindings.clone()));
```

`.manage(shared_bindings.clone())` so commands can mutate it.

- [ ] **Step 13.2: Refactor mouse listener to read from SharedBindings**

In `hotkey/mouse.rs`, change `spawn_listener` signature:
```rust
pub fn spawn_listener(
    bindings: super::SharedBindings,
    tx: tokio::sync::mpsc::UnboundedSender<TriggerEvent>,
)
```

Inside the rdev `ButtonPress` arm, lock `bindings` and iterate. (Lock is held briefly; rdev callback can tolerate it.)

- [ ] **Step 13.3: Re-register notifier for keyboard**

Add a notifier channel:
```rust
let (rebind_tx, mut rebind_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
```

`.manage(rebind_tx.clone())` so commands can signal.

In `setup`, spawn a task that re-registers keyboard hotkeys on each rebind signal:
```rust
let app_handle = app.handle().clone();
let bindings_for_rebind = shared_bindings.clone();
let trigger_tx = hotkey_tx.clone();
tauri::async_runtime::spawn(async move {
    while rebind_rx.recv().await.is_some() {
        hotkey::keyboard::unregister_all(&app_handle);
        let current = bindings_for_rebind.lock().clone();
        let failures = hotkey::keyboard::register_all(
            &app_handle,
            &current,
            trigger_tx.clone(),
        );
        for (id, err) in &failures {
            tracing::warn!(binding_id = %id, error = %err, "rebind failed");
        }
    }
});
```

- [ ] **Step 13.4: Update CRUD commands to refresh shared state and signal**

In `commands.rs`, modify `upsert_hotkey_binding` and `delete_hotkey_binding`:

```rust
#[tauri::command]
pub fn upsert_hotkey_binding(
    binding: HotkeyBinding,
    store: tauri::State<Arc<SettingsStore>>,
    shared: tauri::State<crate::hotkey::SharedBindings>,
    rebind_tx: tauri::State<tokio::sync::mpsc::UnboundedSender<()>>,
) -> Result<(), String> {
    let mut settings = store.load();
    if let Some(existing) = settings.hotkey_bindings.iter_mut().find(|b| b.id == binding.id) {
        *existing = binding;
    } else {
        settings.hotkey_bindings.push(binding);
    }
    store.save(&settings).map_err(|e| e.to_string())?;
    *shared.lock() = settings.hotkey_bindings.clone();
    let _ = rebind_tx.send(());
    Ok(())
}

#[tauri::command]
pub fn delete_hotkey_binding(
    binding_id: String,
    store: tauri::State<Arc<SettingsStore>>,
    shared: tauri::State<crate::hotkey::SharedBindings>,
    rebind_tx: tauri::State<tokio::sync::mpsc::UnboundedSender<()>>,
) -> Result<(), String> {
    let mut settings = store.load();
    settings.hotkey_bindings.retain(|b| b.id != binding_id);
    store.save(&settings).map_err(|e| e.to_string())?;
    *shared.lock() = settings.hotkey_bindings.clone();
    let _ = rebind_tx.send(());
    Ok(())
}
```

- [ ] **Step 13.5: Build + manual smoke**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Manual: in running app, add a new keyboard binding via Settings → confirm hotkey fires without restart. Edit it (different accelerator) → old one stops, new one fires. Delete it → no fire.

- [ ] **Step 13.6: Test the mouse path too**

Add a mouse binding → confirm it fires immediately. Delete → no fire. (rdev listener reads from shared state every press, no rebind signal needed.)

- [ ] **Step 13.2: Test**

After upserting a binding in Settings GUI, the new hotkey should fire without restart. Manual smoke.

- [ ] **Step 13.3: Commit**

```bash
git add src-tauri/src
git commit -m "feat(hotkey): live re-registration on binding CRUD"
```

### Task 14: Context-aware menu mode

**Files:**
- Modify: `src-tauri/src/radial/mod.rs` (already filters by `menu_mode`, but pass through to frontend)
- Modify: `src/radial/RadialMenu.tsx` (filter `items` by tag matching `menu_mode`)

- [ ] **Step 14.1: Frontend filter**

In `RadialMenu.tsx`, listen for `radial:show` payload and use `menu_mode`:

```ts
listen<ShowPayload>('radial:show', (e) => {
  const mode = e.payload.menu_mode;
  setMenuMode(mode);
});

// When rendering:
const filtered = items.filter((it) =>
  menuMode === 'all'
    ? true
    : it.tags.includes(menuMode)
);
```

If `recent_app_name` is present and `menu_mode === 'notification'`, optionally display the sender app name in the menu center: `<text>{recentAppName}</text>`.

- [ ] **Step 14.2: Settings — add `menu_mode` to HotkeyEditor**

Add a dropdown `<select>` in HotkeyEditor row: `all` / `launcher` / `notification`.

- [ ] **Step 14.3: Commit**

```bash
git add src src-tauri/src/radial
git commit -m "feat(radial): context-aware menu mode via tags"
```

---

## Task 15: Docs + manual QA + tag

### Step 15.1: README

Add a "Radial menu" section describing:
- Default state: empty, user opens Settings to add items
- How to add a hotkey binding (capture button)
- How to add a menu item (label + icon + action)
- Action types and shell confirm semantics
- Known limits: shell command runs with mouse-noti's privileges; user is responsible

### Step 15.2: Manual QA checklist

Add to `docs/superpowers/checklists/mvp-manual-qa.md`:

```markdown
## Radial menu

- [ ] Settings → Menu items: add 3 items (Slack launch_app, GitHub open_url, ls run_shell with confirm=true)
- [ ] Settings → Hotkey bindings: bind keyboard (Cmd+Shift+M) and mouse (button 4 + Cmd)
- [ ] Press keyboard binding → radial appears at cursor with 3 sectors
- [ ] Hover each sector → highlight follows
- [ ] Click Slack sector → Slack opens, menu closes
- [ ] Re-trigger → click GitHub sector → browser opens
- [ ] Re-trigger → click ls sector → confirm dialog appears → confirm → command runs
- [ ] ESC during menu → menu closes without executing
- [ ] Add a 4th item → re-trigger → menu now has 4 sectors
- [ ] Press mouse binding (button 4 + Cmd) → radial appears
- [ ] Trigger a real macOS notification → press hotkey configured as menu_mode=notification → menu shows only notification-tagged items
- [ ] Reorder menu items via ↑/↓ → re-trigger → sector order reflects new order
- [ ] Restart app → bindings + items persist
```

### Step 15.3: Version bump + tag

- [ ] Bump `0.2.0` → `0.3.0` in `Cargo.toml`, `package.json`, `tauri.conf.json`
- [ ] Commit: `chore: bump version to 0.3.0`
- [ ] Tag: `git tag -a v0.3.0 -m "v0.3.0 — radial menu (Phase 2)"`

---

## Risks and Open Questions

| Risk | Mitigation |
|---|---|
| `rdev::listen` requires Accessibility permission on macOS | Already granted in v0.2.0 testing; if not, surface in PermissionPanel + log warning. |
| Mouse hotkey conflicts with normal click semantics | Default bindings REQUIRE a modifier (no bare side-button). Frontend HotkeyCapture warns if modifiers=0. |
| `rdev` listener thread cannot be live-updated | Mouse listener reads bindings from `Arc<Mutex<Vec<>>>` shared with the rest of the app; mutations to the vec take effect on next button event. |
| Keyboard accelerator parsing differences between macOS/Windows | Use `CommandOrControl` modifier name (tauri-plugin-global-shortcut normalizes to platform-correct key). |
| App icon extraction returns no image for sandboxed apps | `load_app_icon_base64` returns Err; UI falls back to emoji. |
| Radial window steals focus → conflicts with text input | Window auto-hides on ESC and on outside click (Task 7 adds outside-click handler via window blur event). |
| Shell command run with mouse-noti's privileges | Documented limitation; confirm dialog is the user's safeguard. Future: sandbox profile. |
| Vec<MenuItem> can grow unbounded | Editor enforces max 12 items (matches geometry cap); 13th item save returns error from `upsert_menu_item`. |
| HotkeyCapture in browser cannot capture mouse buttons 4/5 reliably | Most browsers expose 3/4 via `MouseEvent.button`. If a user's browser doesn't fire mousedown for those, fall back to a manual entry input. Document the limit. |
| Outside-click should hide menu but radial window is 400x400; cursor outside the visible disc inside the window | Click handler: if `sectorAt` returns `null`, hide the window without execution. |

---

## Definition of Done

- [ ] All Task 1–14 commits land cleanly.
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml`: clean (zero warnings).
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --all-targets`: zero warnings.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --lib`: ≥22 tests passing.
- [ ] `pnpm test:run`: ≥10 tests passing.
- [ ] Manual QA checklist (Task 15.2) passes end-to-end on macOS.
- [ ] Tagged `v0.3.0`.

---

## Out of Scope (deferred to Phase 3)

- `noti_reply` action (responding to a notification via macOS AX simulation).
- Brand-color presets for indicator (current hash-based color stays).
- Drag-reorder for menu items (↑/↓ buttons sufficient for MVP).
- Per-item global hotkey (item itself fires hotkey without radial). Phase 3.
- Usage statistics / auto-sort by recency.
- Custom radial menu themes / animation customization.
