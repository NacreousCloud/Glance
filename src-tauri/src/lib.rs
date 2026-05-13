pub mod event_bus;
pub mod noti;
pub mod overlay;
pub mod settings;

mod tray;

use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mouse_noti=info".into()),
        )
        .try_init();

    let bus = event_bus::EventBus::new();
    let store = Arc::new(settings::SettingsStore::new(settings::default_config_path()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(bus.clone())
        .manage(store.clone())
        .invoke_handler(tauri::generate_handler![])
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
