use crate::settings::{Settings, SettingsStore};
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
