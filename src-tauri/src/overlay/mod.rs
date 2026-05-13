pub mod cursor;
pub mod display;

use crate::event_bus::EventBus;
use crate::settings::IndicatorStyle;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

#[derive(Serialize, Clone)]
pub struct IndicatorPayload {
    pub id: String,
    pub timestamp_ms: u64,
    pub style: IndicatorStyle,
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub app_name: String,
    pub title: String,
}

pub fn spawn<R: Runtime>(
    app: AppHandle<R>,
    bus: EventBus,
    style_provider: impl Fn() -> IndicatorStyle + Send + 'static,
) {
    let mut rx = bus.subscribe();

    // Ensure initial overlay window is click-through
    if let Some(win) = ensure_window(&app) {
        let _ = win.set_ignore_cursor_events(true);
    }

    tauri::async_runtime::spawn(async move {
        tracing::info!("overlay subscriber attached, waiting for events");
        loop {
            let event = match rx.recv().await {
                Ok(e) => e,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "overlay subscriber lagged behind, skipping events");
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    tracing::warn!("overlay subscriber channel closed");
                    break;
                }
            };
            tracing::info!(
                title = %event.title,
                app = %event.app_name,
                "overlay received event"
            );
            let style = style_provider();
            let Some(pos) = cursor::current_position() else {
                tracing::warn!("cursor position unavailable; skipping indicator");
                continue;
            };
            let Some(win) = ensure_window(&app) else {
                tracing::warn!("overlay window not found; skipping indicator");
                continue;
            };
            let Some(placement) = display::position_overlay_at(&win, pos) else {
                tracing::warn!(cursor = ?pos, "no monitor contains cursor; skipping indicator");
                continue;
            };
            tracing::debug!(
                cursor_physical = ?pos,
                origin = ?(placement.origin_x, placement.origin_y),
                scale = placement.scale,
                "overlay positioned"
            );

            // Double check click-through is enabled whenever showing
            let _ = win.set_ignore_cursor_events(true);
            if let Err(e) = win.show() {
                tracing::warn!(error = %e, "win.show failed");
            }

            let local_logical = (
                (pos.0 - placement.origin_x) / placement.scale,
                (pos.1 - placement.origin_y) / placement.scale,
            );
            let payload = IndicatorPayload {
                id: event.id.clone(),
                timestamp_ms: event.timestamp_ms,
                style,
                cursor_x: local_logical.0,
                cursor_y: local_logical.1,
                app_name: event.app_name.clone(),
                title: event.title.clone(),
            };
            match app.emit("noti:show", payload) {
                Ok(()) => tracing::info!(
                    local = ?local_logical,
                    "emitted noti:show to webview"
                ),
                Err(e) => tracing::warn!(error = %e, "emit noti:show failed"),
            }
        }
    });
}

fn ensure_window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window("overlay")
}
