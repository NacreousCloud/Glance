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
    tracing::info!(menu_mode, "radial::show entered");
    let Some(pos) = cursor::current_position() else {
        tracing::warn!("cursor position unavailable; skipping radial show");
        return;
    };
    let Some(win) = app.get_webview_window("radial") else {
        tracing::warn!("radial window not found");
        return;
    };
    tracing::info!(?pos, "radial::show: window found, cursor ok");
    match win.url() {
        Ok(url) => tracing::info!(%url, "radial::show: current webview url"),
        Err(e) => tracing::warn!(error = %e, "radial::show: win.url() failed"),
    }

    let scale = win.scale_factor().unwrap_or(1.0);
    let half_w = 200.0 * scale;
    let half_h = 200.0 * scale;
    let target_pos = tauri::PhysicalPosition::new(
        (pos.0 - half_w) as i32,
        (pos.1 - half_h) as i32,
    );
    let target_size =
        tauri::PhysicalSize::new((400.0 * scale) as u32, (400.0 * scale) as u32);
    tracing::info!(?target_pos, ?target_size, scale, "radial::show: positioning");
    if let Err(e) = win.set_position(target_pos) {
        tracing::warn!(error = %e, "set_position failed");
    }
    if let Err(e) = win.set_size(target_size) {
        tracing::warn!(error = %e, "set_size failed");
    }
    if let Err(e) = win.set_visible_on_all_workspaces(true) {
        tracing::warn!(error = %e, "set_visible_on_all_workspaces failed");
    }
    if let Err(e) = win.set_always_on_top(true) {
        tracing::warn!(error = %e, "set_always_on_top failed");
    }
    match win.show() {
        Ok(()) => tracing::info!("radial::show: win.show OK"),
        Err(e) => tracing::warn!(error = %e, "win.show failed"),
    }
    match win.set_focus() {
        Ok(()) => tracing::info!("radial::show: win.set_focus OK"),
        Err(e) => tracing::warn!(error = %e, "win.set_focus failed"),
    }

    if let Ok(visible) = win.is_visible() {
        tracing::info!(visible, "radial::show: post-show visibility");
    }
    if let Ok(inner) = win.inner_position() {
        tracing::info!(?inner, "radial::show: inner_position after set");
    }
    if let Ok(inner) = win.inner_size() {
        tracing::info!(?inner, "radial::show: inner_size after set");
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
    match app.emit("radial:show", payload) {
        Ok(()) => tracing::info!("radial::show: emitted radial:show event"),
        Err(e) => tracing::warn!(error = %e, "emit radial:show failed"),
    }

    // Diagnostic: read back the webview title (set by frontend on invoke
    // failure) so we can see invoke error text in the terminal.
    tokio::time::sleep(Duration::from_millis(200)).await;
    if let Ok(title) = win.title() {
        tracing::info!(%title, "radial::show: webview title after emit");
    }
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(win) = app.get_webview_window("radial") {
        let _ = win.hide();
    }
}
