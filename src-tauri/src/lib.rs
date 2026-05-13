pub mod commands;
pub mod event_bus;
pub mod noti;
pub mod overlay;
pub mod settings;

mod tray;

use std::sync::Arc;

use commands::{get_settings, permission_status, request_permission, set_settings};
use event_bus::EventBus;
use noti::{NotiEvent, NotificationSource};
use settings::{default_config_path, SettingsStore};

fn start_os_source(bus: &EventBus) {
    let bus = bus.clone();
    let publish: Box<dyn Fn(NotiEvent) + Send + Sync> =
        Box::new(move |e| {
            bus.publish(e);
        });
    #[cfg(target_os = "macos")]
    {
        let src = noti::macos::MacosNotiSource::new();
        if let Err(e) = src.start(publish) {
            tracing::warn!(error = %e, "macOS source failed to start");
        }
        std::mem::forget(src); // hold for app lifetime
    }
    #[cfg(target_os = "windows")]
    {
        let src = noti::windows::WindowsNotiSource::new();
        if let Err(e) = src.start(publish) {
            tracing::warn!(error = %e, "Windows source failed to start");
        }
        std::mem::forget(src);
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = publish; // silence unused warning on other platforms
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mouse_noti=info".into()),
        )
        .try_init();

    let bus = EventBus::new();
    let store = Arc::new(SettingsStore::new(default_config_path()));

    start_os_source(&bus);

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .manage(bus.clone())
        .manage(store.clone())
        .invoke_handler({
            #[cfg(feature = "mock-os")]
            {
                tauri::generate_handler![
                    get_settings,
                    set_settings,
                    permission_status,
                    request_permission,
                    commands::inject_mock_event
                ]
            }
            #[cfg(not(feature = "mock-os"))]
            {
                tauri::generate_handler![
                    get_settings,
                    set_settings,
                    permission_status,
                    request_permission
                ]
            }
        })
        .setup(move |app| {
            tray::install(&app.handle())?;
            let store_for_style = store.clone();
            let bus_clone = bus.clone();
            overlay::spawn(app.handle().clone(), bus_clone, move || {
                store_for_style.load().indicator_style
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
