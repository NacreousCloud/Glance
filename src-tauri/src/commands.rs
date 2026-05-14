use crate::action::ActionRunner;
use crate::settings::{HotkeyBinding, MenuItem, Settings, SettingsStore};
use std::sync::Arc;

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
pub fn set_settings(
    settings: Settings,
    store: tauri::State<Arc<SettingsStore>>,
) -> Result<(), String> {
    store.save(&settings).map_err(|e| e.to_string())
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
    shared: tauri::State<crate::hotkey::SharedBindings>,
    rebind_tx: tauri::State<tokio::sync::mpsc::UnboundedSender<()>>,
) -> Result<(), String> {
    let mut settings = store.load();
    if let Some(existing) = settings
        .hotkey_bindings
        .iter_mut()
        .find(|b| b.id == binding.id)
    {
        *existing = binding;
    } else {
        settings.hotkey_bindings.push(binding);
    }
    store.save(&settings).map_err(|e| e.to_string())?;
    // Mirror to shared state so mouse listener sees it immediately,
    // then signal the rebind drainer to re-register keyboard hotkeys.
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

#[tauri::command]
pub fn extract_app_icon(path: String) -> Result<String, String> {
    crate::app_icon::load_app_icon_base64(&path).map_err(|e| e.to_string())
}
