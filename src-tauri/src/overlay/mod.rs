pub mod cursor;
pub mod display;

use crate::event_bus::EventBus;
use crate::settings::IndicatorStyle;
use serde::Serialize;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

fn hue_from_label(label: &str) -> u16 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut hasher);
    (hasher.finish() % 360) as u16
}

#[derive(Serialize, Clone)]
pub struct IndicatorPayload {
    pub id: String,
    pub timestamp_ms: u64,
    pub style: IndicatorStyle,
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub viewport_w: f64,
    pub viewport_h: f64,
    pub app_name: String,
    pub title: String,
    pub color_hue: u16,
}

pub fn spawn<R: Runtime>(
    app: AppHandle<R>,
    bus: EventBus,
    style_provider: impl Fn() -> IndicatorStyle + Send + 'static,
    enabled_provider: impl Fn() -> bool + Send + 'static,
) {
    let mut rx = bus.subscribe();

    // Ensure initial overlay window is click-through and visible everywhere.
    if let Some(win) = ensure_window(&app) {
        let _ = win.set_ignore_cursor_events(true);
        // Visible across all macOS desktop spaces + fullscreen apps.
        let _ = win.set_visible_on_all_workspaces(true);
        // Re-assert always-on-top in case settings drift.
        let _ = win.set_always_on_top(true);
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
            // Master switch. Error events (dev.error) bypass the toggle so
            // failures always reach the user even with the indicator off.
            if !enabled_provider() && event.app_id != "dev.error" {
                tracing::debug!("indicator disabled; dropping event");
                continue;
            }
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
                size_physical = ?(placement.width, placement.height),
                scale = placement.scale,
                "overlay positioned"
            );

            // Re-assert window properties on every show. macOS occasionally
            // strips these when the window moves between displays / spaces.
            let _ = win.set_ignore_cursor_events(true);
            let _ = win.set_visible_on_all_workspaces(true);
            let _ = win.set_always_on_top(true);
            if let Err(e) = win.show() {
                tracing::warn!(error = %e, "win.show failed");
            }

            // macOS applies set_position/set_size asynchronously. Webview can
            // render with the previous (stale) size if we emit immediately,
            // which makes the indicator land outside the new viewport. Give
            // the window manager a frame to settle.
            tokio::time::sleep(Duration::from_millis(40)).await;

            // Read inner_size AFTER the sleep so the diagnostic reflects the
            // size webview is actually using to render.
            let inner = win.inner_size().ok();
            if let Some(sz) = inner {
                tracing::debug!(
                    physical = ?(sz.width, sz.height),
                    "overlay window inner_size at emit time"
                );
            }

            let local_logical = (
                (pos.0 - placement.origin_x) / placement.scale,
                (pos.1 - placement.origin_y) / placement.scale,
            );
            let viewport_logical = inner
                .map(|sz| (sz.width as f64 / placement.scale, sz.height as f64 / placement.scale))
                .unwrap_or((placement.width / placement.scale, placement.height / placement.scale));
            // Error events (app_id="dev.error") get a fixed red Persistent
            // Badge regardless of the user's chosen indicator style so
            // failures are visually distinct and the message stays on
            // screen long enough to read.
            let is_error = event.app_id == "dev.error";
            let style_used = if is_error {
                IndicatorStyle::PersistentBadge
            } else {
                style
            };
            let color_hue = if is_error {
                0
            } else {
                hue_from_label(if event.app_name.is_empty() {
                    &event.app_id
                } else {
                    &event.app_name
                })
            };
            let payload = IndicatorPayload {
                id: event.id.clone(),
                timestamp_ms: event.timestamp_ms,
                style: style_used,
                cursor_x: local_logical.0,
                cursor_y: local_logical.1,
                viewport_w: viewport_logical.0,
                viewport_h: viewport_logical.1,
                app_name: event.app_name.clone(),
                title: event.title.clone(),
                color_hue,
            };
            match app.emit("noti:show", payload) {
                Ok(()) => tracing::info!(
                    local = ?local_logical,
                    viewport = ?viewport_logical,
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
