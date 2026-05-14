pub mod action;
pub mod commands;
pub mod event_bus;
pub mod noti;
pub mod overlay;
pub mod settings;

mod tray;

use std::sync::Arc;
use parking_lot::Mutex;

use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use commands::{
    get_recent_events, get_settings, permission_status, request_permission, set_settings,
};

#[cfg(feature = "mock-os")]
use commands::inject_mock_event;

use event_bus::EventBus;
use noti::{NotiEvent, NotificationSource};
use settings::{default_config_path, SettingsStore};

pub struct ActiveSource(pub Arc<Mutex<Option<Box<dyn NotificationSource>>>>);

fn start_os_source(app: &tauri::AppHandle, bus: &EventBus) {
    let bus = bus.clone();
    let publish: Box<dyn Fn(NotiEvent) + Send + Sync> = Box::new(move |e| {
        bus.publish(e);
    });

    let source: Option<Box<dyn NotificationSource>> = {
        #[cfg(target_os = "macos")]
        {
            let src = noti::macos::MacosNotiSource::new();
            if let Err(e) = src.start(publish) {
                tracing::warn!(error = %e, "macOS source failed to start");
                None
            } else {
                Some(Box::new(src))
            }
        }
        #[cfg(target_os = "windows")]
        {
            let src = noti::windows::WindowsNotiSource::new();
            if let Err(e) = src.start(publish) {
                tracing::warn!(error = %e, "Windows source failed to start");
                None
            } else {
                Some(Box::new(src))
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = publish;
            None
        }
    };

    if let Some(s) = source {
        let active_source = app.state::<ActiveSource>();
        *active_source.0.lock() = Some(s);
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
    let active_source = ActiveSource(Arc::new(Mutex::new(None)));

    let debug_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed && shortcut == &debug_shortcut {
                        let bus = app.state::<EventBus>();
                        bus.publish(NotiEvent::now(
                            "dev.debug",
                            "Debug Trigger",
                            "Debug Notification",
                            "Manual trigger for testing",
                        ));
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .manage(bus.clone())
        .manage(store.clone())
        .manage(active_source)
        .invoke_handler({
            #[cfg(feature = "mock-os")]
            {
                tauri::generate_handler![
                    get_settings,
                    set_settings,
                    permission_status,
                    request_permission,
                    inject_mock_event,
                    get_recent_events
                ]
            }
            #[cfg(not(feature = "mock-os"))]
            {
                tauri::generate_handler![
                    get_settings,
                    set_settings,
                    permission_status,
                    request_permission,
                    get_recent_events
                ]
            }
        })
        .setup(move |app| {
            let ctrl_shift_d = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);
            app.global_shortcut().register(ctrl_shift_d)?;

            tray::install(app.handle())?;

            // Subscribe overlay BEFORE starting the OS source so no early publishes are lost.
            let store_for_style = store.clone();
            let bus_clone = bus.clone();
            overlay::spawn(app.handle().clone(), bus_clone, move || {
                store_for_style.load().indicator_style
            });

            start_os_source(app.handle(), &bus);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
