pub mod action;
pub mod app_icon;
pub mod commands;
pub mod error_log;
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
    clear_errors, delete_hotkey_binding, delete_menu_item, exec_menu_item, extract_app_icon,
    get_recent_errors, get_recent_events, get_settings, hide_radial, list_hotkey_bindings,
    list_menu_items, permission_status, radial_log, reorder_menu_items, request_permission,
    set_settings, upsert_hotkey_binding, upsert_menu_item,
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
                .unwrap_or_else(|_| "glance=info".into()),
        )
        .try_init();

    // Surface any thread panic to stderr + tracing so silent exits become visible.
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let tname = thread.name().unwrap_or("<unnamed>");
        eprintln!("[PANIC] thread={tname} info={info}");
        tracing::error!(thread = %tname, %info, "panic in thread");
    }));

    let bus = EventBus::new();
    let store = Arc::new(SettingsStore::new(default_config_path()));
    let active_source = ActiveSource(Arc::new(Mutex::new(None)));
    let error_log = error_log::ErrorLog::new();

    let debug_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);

    let hotkey_registry: hotkey::keyboard::RegistryMap =
        Arc::new(Mutex::new(HashMap::new()));
    let (hotkey_tx, mut hotkey_rx) =
        tokio::sync::mpsc::unbounded_channel::<hotkey::TriggerEvent>();

    // Shared, mutable hotkey bindings — both keyboard registry rebinder and
    // mouse listener read from the same Arc<Mutex<...>>.
    let shared_bindings: hotkey::SharedBindings =
        Arc::new(Mutex::new(store.load().hotkey_bindings.clone()));
    // Rebind notifier: CRUD commands signal here; drainer in setup re-registers.
    let (rebind_tx, mut rebind_rx) =
        tokio::sync::mpsc::unbounded_channel::<()>();

    let registry_for_handler = hotkey_registry.clone();
    let tx_for_handler = hotkey_tx.clone();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
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
                    let key = hotkey::keyboard::shortcut_key(shortcut);
                    let lookup = registry_for_handler.lock().get(&key).cloned();
                    if let Some((binding_id, menu_mode)) = lookup {
                        let _ = tx_for_handler.send(hotkey::TriggerEvent {
                            binding_id,
                            menu_mode,
                        });
                    } else {
                        tracing::debug!(key = %key, "shortcut pressed but not in registry");
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(bus.clone())
        .manage(store.clone())
        .manage(active_source)
        .manage(error_log.clone())
        .manage(shared_bindings.clone())
        .manage(rebind_tx.clone())
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
                    radial_log,
                    hide_radial,
                    get_recent_errors,
                    clear_errors
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
                    radial_log,
                    hide_radial,
                    get_recent_errors,
                    clear_errors
                ]
            }
        })
        .setup(move |app| {
            let ctrl_shift_d = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);
            app.global_shortcut().register(ctrl_shift_d)?;

            tray::install(app.handle())?;

            // Intercept Settings window close → hide instead of destroy so
            // the tray "Settings…" entry can re-show it on subsequent clicks.
            if let Some(settings_win) = app.get_webview_window("settings") {
                let win_clone = settings_win.clone();
                settings_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            // Subscribe overlay BEFORE starting the OS source so no early publishes are lost.
            let store_for_style = store.clone();
            let store_for_enabled = store.clone();
            let bus_clone = bus.clone();
            overlay::spawn(
                app.handle().clone(),
                bus_clone,
                move || store_for_style.load().indicator_style,
                move || store_for_enabled.load().indicator_enabled,
            );

            // Register keyboard hotkeys from settings (initial snapshot).
            {
                let initial = shared_bindings.lock().clone();
                let failures =
                    hotkey::keyboard::register_all(app.handle(), &initial, &hotkey_registry);
                for (id, err) in &failures {
                    tracing::warn!(binding_id = %id, error = %err, "failed to register keyboard hotkey");
                }
            }

            // Drain hotkey trigger events; show radial on each.
            let app_handle = app.handle().clone();
            let bus_for_hotkey = bus.clone();
            tauri::async_runtime::spawn(async move {
                while let Some(event) = hotkey_rx.recv().await {
                    tracing::debug!(?event, "hotkey fired");
                    crate::radial::show(&app_handle, &bus_for_hotkey, &event.menu_mode).await;
                }
            });

            // Drain rebind signals; unregister + re-register keyboard hotkeys
            // from the latest shared_bindings snapshot.
            let app_handle_for_rebind = app.handle().clone();
            let registry_for_rebind = hotkey_registry.clone();
            let bindings_for_rebind = shared_bindings.clone();
            tauri::async_runtime::spawn(async move {
                while rebind_rx.recv().await.is_some() {
                    hotkey::keyboard::unregister_all(
                        &app_handle_for_rebind,
                        &registry_for_rebind,
                    );
                    let current = bindings_for_rebind.lock().clone();
                    let failures = hotkey::keyboard::register_all(
                        &app_handle_for_rebind,
                        &current,
                        &registry_for_rebind,
                    );
                    for (id, err) in &failures {
                        tracing::warn!(binding_id = %id, error = %err, "rebind failed");
                    }
                }
            });

            start_os_source(app.handle(), &bus);

            // Mouse hotkey listener. macOS: NSEvent global monitor (needs
            // Accessibility). Windows: WH_MOUSE_LL low-level hook on a
            // dedicated thread. Monitor is intentionally leaked — it must
            // live for the process lifetime and the OS cleans up on exit.
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                let monitor = hotkey::mouse::MouseMonitor::start(
                    shared_bindings.clone(),
                    hotkey_tx.clone(),
                );
                std::mem::forget(monitor);
            }

            // Force Touch (trackpad) — macOS only. Pressure event global
            // monitor, same lifetime semantics as the mouse monitor.
            #[cfg(target_os = "macos")]
            {
                let monitor = hotkey::gestures::GestureMonitor::start(
                    shared_bindings.clone(),
                    hotkey_tx.clone(),
                );
                std::mem::forget(monitor);
            }

            // Hot-corner trigger — cross-platform cursor poll thread.
            {
                let monitor = hotkey::hot_corner::HotCornerMonitor::start(
                    app.handle().clone(),
                    shared_bindings.clone(),
                    hotkey_tx.clone(),
                );
                std::mem::forget(monitor);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
