pub mod geometry;

use crate::event_bus::EventBus;
use crate::overlay::cursor;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime};

#[derive(serde::Serialize, Clone)]
pub struct ShowPayload {
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub menu_mode: String,
    pub recent_app_name: Option<String>,
}

pub async fn show<R: Runtime>(app: &AppHandle<R>, bus: &EventBus, menu_mode: &str) {
    let Some(pos) = cursor::current_position() else {
        tracing::warn!("cursor position unavailable; skipping radial show");
        return;
    };
    let Some(win) = app.get_webview_window("radial") else {
        tracing::warn!("radial window not found");
        return;
    };

    let scale = win.scale_factor().unwrap_or(1.0);
    let half_w = 200.0 * scale;
    let half_h = 200.0 * scale;
    let target_pos = tauri::PhysicalPosition::new(
        (pos.0 - half_w) as i32,
        (pos.1 - half_h) as i32,
    );
    let target_size =
        tauri::PhysicalSize::new((400.0 * scale) as u32, (400.0 * scale) as u32);

    if let Err(e) = win.set_position(target_pos) {
        tracing::warn!(error = %e, "set_position failed");
    }
    if let Err(e) = win.set_size(target_size) {
        tracing::warn!(error = %e, "set_size failed");
    }
    let _ = win.set_visible_on_all_workspaces(true);
    let _ = win.set_always_on_top(true);
    if let Err(e) = win.show() {
        tracing::warn!(error = %e, "win.show failed");
    }
    if let Err(e) = win.set_focus() {
        tracing::warn!(error = %e, "win.set_focus failed");
    }

    tokio::time::sleep(Duration::from_millis(30)).await;

    let recent = bus.recent_within(Duration::from_secs(5));
    let recent_app_name = recent.last().map(|e| e.app_name.clone());

    let payload = ShowPayload {
        cursor_x: pos.0,
        cursor_y: pos.1,
        menu_mode: menu_mode.to_string(),
        recent_app_name,
    };
    if let Err(e) = app.emit("radial:show", payload) {
        tracing::warn!(error = %e, "emit radial:show failed");
    }
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("radial") {
        let _ = win.hide();
    }
}
