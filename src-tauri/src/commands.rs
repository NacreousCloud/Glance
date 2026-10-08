use crate::action::ActionRunner;
use crate::settings::{HotkeyBinding, MenuItem, PreferencesPatch, Settings, SettingsStore};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

/// Frontend-side log relay. Used sparingly for diagnostics in webview
/// code paths where DevTools isn't available; default builds rarely
/// invoke this.
#[tauri::command]
pub fn radial_log(msg: String) {
    tracing::debug!("[radial:js] {}", msg);
}

#[tauri::command]
pub fn hide_radial(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("radial") {
        if let Err(e) = win.hide() {
            tracing::warn!(error = %e, "hide_radial: win.hide failed");
        }
    }
}

#[derive(serde::Serialize)]
pub struct PermissionStatus {
    pub accessibility_ok: bool,
    pub notification_listener_ok: bool,
    pub platform: &'static str,
}

#[tauri::command]
pub fn get_settings(store: tauri::State<Arc<SettingsStore>>) -> Settings {
    store.load()
}

#[tauri::command]
pub fn patch_preferences(
    patch: PreferencesPatch,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    store
        .update(|settings| patch.apply(settings))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn permission_status() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    {
        let ok = crate::noti::macos::MacosNotiSource::is_trusted(false);
        PermissionStatus {
            accessibility_ok: ok,
            notification_listener_ok: true,
            platform: "macos",
        }
    }
    #[cfg(target_os = "windows")]
    {
        let ok = match crate::noti::windows::WindowsNotiSource::access_status() {
            Ok(s) => matches!(
                s,
                windows::UI::Notifications::Management::UserNotificationListenerAccessStatus::Allowed
            ),
            Err(_) => false,
        };
        PermissionStatus {
            accessibility_ok: true,
            notification_listener_ok: ok,
            platform: "windows",
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    PermissionStatus {
        accessibility_ok: false,
        notification_listener_ok: false,
        platform: "other",
    }
}

#[tauri::command]
pub async fn request_permission() -> Result<(), String> {
    // Runs on tauri's async runtime; offload blocking dialog to spawn_blocking.
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(target_os = "macos")]
        {
            // Side effect: shows system Accessibility prompt if not granted.
            let _ = crate::noti::macos::MacosNotiSource::is_trusted(true);
            Ok::<(), String>(())
        }
        #[cfg(target_os = "windows")]
        {
            crate::noti::windows::WindowsNotiSource::access_status()
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Err::<(), String>("unsupported".into())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(feature = "mock-os")]
#[tauri::command]
pub fn inject_mock_event(
    bus: tauri::State<crate::event_bus::EventBus>,
    app_id: String,
    app_name: String,
    title: String,
    body: String,
) {
    bus.publish(crate::noti::NotiEvent::now(app_id, app_name, title, body));
}

#[tauri::command]
pub fn get_recent_events(
    bus: tauri::State<crate::event_bus::EventBus>,
) -> Vec<crate::noti::NotiEvent> {
    bus.recent_within(std::time::Duration::from_secs(3600)) // Last hour
}

/// Available in release builds; no OS notification permission is needed.
#[tauri::command]
pub fn test_notification(
    app: tauri::AppHandle,
    bus: tauri::State<crate::event_bus::EventBus>,
) -> Result<(), String> {
    if app.get_webview_window("overlay").is_none() {
        return Err("Notification overlay is unavailable".into());
    }
    if crate::overlay::cursor::current_position().is_none() {
        return Err("Cursor position is unavailable".into());
    }
    bus.publish(crate::noti::NotiEvent::now(
        "dev.preview",
        "Glance",
        "Test notification",
        "Your cursor indicator is ready.",
    ));
    Ok(())
}

#[tauri::command]
pub fn list_menu_items(store: tauri::State<Arc<SettingsStore>>) -> Vec<MenuItem> {
    store.load().menu_items
}

#[tauri::command]
pub fn upsert_menu_item(
    item: MenuItem,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    store
        .update(|settings| {
            if let Some(existing) = settings.menu_items.iter_mut().find(|i| i.id == item.id) {
                *existing = item;
            } else {
                settings.menu_items.push(item);
            }
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_menu_item(
    item_id: String,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    store
        .update(|settings| settings.menu_items.retain(|i| i.id != item_id))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reorder_menu_items(
    ids: Vec<String>,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    store
        .update(|settings| {
            let mut remaining = std::mem::take(&mut settings.menu_items);
            let mut reordered = Vec::with_capacity(remaining.len());
            for id in ids {
                if let Some(index) = remaining.iter().position(|item| item.id == id) {
                    reordered.push(remaining.remove(index));
                }
            }
            reordered.extend(remaining);
            settings.menu_items = reordered;
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_hotkey_bindings(store: tauri::State<Arc<SettingsStore>>) -> Vec<HotkeyBinding> {
    store.load().hotkey_bindings
}

#[tauri::command]
pub fn upsert_hotkey_binding(
    binding: HotkeyBinding,
    store: tauri::State<Arc<SettingsStore>>,
    shared: tauri::State<crate::hotkey::SharedBindings>,
    rebind_tx: tauri::State<tokio::sync::mpsc::UnboundedSender<()>>,
) -> Result<(), String> {
    // Hold the live bindings lock across persistence and publication so
    // concurrent commands cannot publish an older snapshot last.
    let mut live = shared.lock();
    let settings = store
        .update(|settings| {
            if let Some(existing) = settings
                .hotkey_bindings
                .iter_mut()
                .find(|b| b.id == binding.id)
            {
                *existing = binding;
            } else {
                settings.hotkey_bindings.push(binding);
            }
        })
        .map_err(|e| e.to_string())?;
    // Mirror to shared state so mouse listener sees it immediately,
    // then signal the rebind drainer to re-register keyboard hotkeys.
    *live = settings.hotkey_bindings;
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
    let mut live = shared.lock();
    let settings = store
        .update(|settings| settings.hotkey_bindings.retain(|b| b.id != binding_id))
        .map_err(|e| e.to_string())?;
    *live = settings.hotkey_bindings;
    let _ = rebind_tx.send(());
    Ok(())
}

#[tauri::command]
pub async fn exec_menu_item(
    app: tauri::AppHandle,
    item_id: String,
    store: tauri::State<'_, Arc<SettingsStore>>,
    bus: tauri::State<'_, crate::event_bus::EventBus>,
    error_log: tauri::State<'_, crate::error_log::ErrorLog>,
) -> Result<(), String> {
    let settings = store.load();
    let item = settings
        .menu_items
        .iter()
        .find(|i| i.id == item_id)
        .ok_or_else(|| format!("menu item not found: {item_id}"))?
        .clone();
    let action = item.action.clone();
    let label = item.label.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        ActionRunner::execute_with_confirmation(&action, |message| {
            app.dialog()
                .message(message)
                .title(format!("Glance — {label}"))
                .buttons(MessageDialogButtons::OkCancelCustom(
                    "Run".into(),
                    "Cancel".into(),
                ))
                .blocking_show()
        })
    })
    .await;
    let result = result
        .map_err(anyhow::Error::from)
        .and_then(|result| result);
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let msg = format!("{e:#}");
            // One-line summary for the on-screen badge.
            let short: String = msg
                .lines()
                .next()
                .unwrap_or(msg.as_str())
                .chars()
                .take(80)
                .collect();
            tracing::warn!(item_id = %item.id, error = %msg, "menu item exec failed");

            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            error_log.push(crate::error_log::ErrorEntry {
                id: format!("err-{}-{}", item.id, now_ms),
                timestamp_ms: now_ms,
                item_id: item.id.clone(),
                item_label: item.label.clone(),
                message: msg.clone(),
            });

            // Special error event the overlay renders as a red Persistent
            // Badge. app_name carries the short one-liner the user sees.
            bus.publish(crate::noti::NotiEvent::now(
                "dev.error",
                short,
                item.label.clone(),
                msg.clone(),
            ));
            Err(msg)
        }
    }
}

#[tauri::command]
pub fn get_recent_errors(
    log: tauri::State<crate::error_log::ErrorLog>,
) -> Vec<crate::error_log::ErrorEntry> {
    log.snapshot()
}

#[tauri::command]
pub fn clear_errors(log: tauri::State<crate::error_log::ErrorLog>) {
    log.clear();
}

#[tauri::command]
pub fn extract_app_icon(path: String) -> Result<String, String> {
    crate::app_icon::load_app_icon_base64(&path).map_err(|e| e.to_string())
}
