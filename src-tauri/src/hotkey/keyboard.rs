use crate::settings::{HotkeyBinding, HotkeyTrigger};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// Map from a registered `Shortcut` (formatted as a stable key derived from
/// its modifiers + key code) to the originating binding's id + menu_mode.
/// Used by the plugin handler in lib.rs to route a press event to a
/// TriggerEvent.
pub type RegistryMap = Arc<Mutex<HashMap<String, (String, String)>>>;

/// Register all keyboard bindings from `bindings`. Returns failures as
/// (binding_id, error_message).
pub fn register_all<R: Runtime>(
    app: &AppHandle<R>,
    bindings: &[HotkeyBinding],
    registry: &RegistryMap,
) -> Vec<(String, String)> {
    let shortcut_ext = app.global_shortcut();
    let mut failures = Vec::new();
    let mut reg = registry.lock();
    reg.clear();
    for binding in bindings {
        if let HotkeyTrigger::Keyboard { accelerator } = &binding.trigger {
            tracing::info!(binding_id = %binding.id, accelerator = %accelerator, "parsing keyboard binding");
            match accelerator.parse::<Shortcut>() {
                Ok(sc) => {
                    let key = shortcut_key(&sc);
                    tracing::info!(binding_id = %binding.id, ?sc, key = %key, "registering shortcut");
                    if let Err(e) = shortcut_ext.register(sc) {
                        tracing::warn!(binding_id = %binding.id, error = %e, "register failed");
                        failures.push((binding.id.clone(), e.to_string()));
                    } else {
                        reg.insert(
                            key,
                            (binding.id.clone(), binding.menu_mode.clone()),
                        );
                        tracing::info!(binding_id = %binding.id, "registered ok");
                    }
                }
                Err(e) => {
                    tracing::warn!(binding_id = %binding.id, accelerator = %accelerator, error = %e, "parse failed");
                    failures.push((binding.id.clone(), e.to_string()));
                }
            }
        }
    }
    failures
}

#[allow(dead_code)]
pub fn unregister_all<R: Runtime>(app: &AppHandle<R>, registry: &RegistryMap) {
    let _ = app.global_shortcut().unregister_all();
    registry.lock().clear();
}

/// Stable map key for a Shortcut — combine modifiers + key code.
/// `Shortcut` is a re-export of `global_hotkey::HotKey`, which has public
/// `mods: Modifiers` and `key: Code` fields with `Debug` derived.
pub fn shortcut_key(sc: &Shortcut) -> String {
    format!("{:?}+{:?}", sc.mods, sc.key)
}
