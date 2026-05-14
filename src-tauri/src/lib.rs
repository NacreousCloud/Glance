pub mod action;
pub mod app_icon;
pub mod commands;
pub mod event_bus;
pub mod hotkey;
pub mod noti;
pub mod overlay;
pub mod radial;
pub mod settings;

mod tray;

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use commands::{
    debug_hide_radial, debug_show_radial, delete_hotkey_binding, delete_menu_item, exec_menu_item,
    extract_app_icon, get_recent_events, get_settings, list_hotkey_bindings, list_menu_items,
    permission_status, reorder_menu_items, request_permission, set_settings,
    upsert_hotkey_binding, upsert_menu_item,
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

    let hotkey_registry: hotkey::keyboard::RegistryMap =
        Arc::new(Mutex::new(HashMap::new()));
    let (hotkey_tx, mut hotkey_rx) =
        tokio::sync::mpsc::unbounded_channel::<hotkey::TriggerEvent>();

    let registry_for_handler = hotkey_registry.clone();
    let tx_for_handler = hotkey_tx.clone();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    // Preserved: Ctrl+Shift+D fires a debug noti.
                    if shortcut == &debug_shortcut {
                        let bus = app.state::<EventBus>();
                        bus.publish(NotiEvent::now(
                            "dev.debug",
                            "Debug Trigger",
                            "Debug Notification",
                            "Manual trigger for testing",
                        ));
                        return;
                    }
                    // Route to a TriggerEvent if matches a user binding.
                    let key = hotkey::keyboard::shortcut_key(shortcut);
                    if let Some((binding_id, menu_mode)) =
                        registry_for_handler.lock().get(&key).cloned()
                    {
                        let _ = tx_for_handler.send(hotkey::TriggerEvent {
                            binding_id,
                            menu_mode,
                        });
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
                    get_recent_events,
                    list_menu_items,
                    upsert_menu_item,
                    delete_menu_item,
                    reorder_menu_items,
                    list_hotkey_bindings,
                    upsert_hotkey_binding,
                    delete_hotkey_binding,
                    exec_menu_item,
                    extract_app_icon,
                    debug_show_radial,
                    debug_hide_radial
                ]
            }
            #[cfg(not(feature = "mock-os"))]
            {
                tauri::generate_handler![
                    get_settings,
                    set_settings,
                    permission_status,
                    request_permission,
                    get_recent_events,
                    list_menu_items,
                    upsert_menu_item,
                    delete_menu_item,
                    reorder_menu_items,
                    list_hotkey_bindings,
                    upsert_hotkey_binding,
                    delete_hotkey_binding,
                    exec_menu_item,
                    extract_app_icon,
                    debug_show_radial,
                    debug_hide_radial
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

            // Register keyboard hotkeys from settings.
            let bindings = store.load().hotkey_bindings.clone();
            let failures =
                hotkey::keyboard::register_all(app.handle(), &bindings, &hotkey_registry);
            for (id, err) in &failures {
                tracing::warn!(binding_id = %id, error = %err, "failed to register keyboard hotkey");
            }

            // Drain hotkey trigger events; show radial on each.
            let app_handle = app.handle().clone();
            let bus_for_hotkey = bus.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = hotkey_rx.recv().await {
                    tracing::info!(?event, "hotkey fired");
                    crate::radial::show(&app_handle, &bus_for_hotkey, &event.menu_mode).await;
                }
            });

            start_os_source(app.handle(), &bus);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
