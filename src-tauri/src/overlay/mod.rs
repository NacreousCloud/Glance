pub mod cursor;
pub mod display;

use crate::event_bus::EventBus;
use crate::settings::IndicatorStyle;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

#[derive(Serialize, Clone)]
pub struct IndicatorPayload {
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
        while let Ok(event) = rx.recv().await {
            let style = style_provider();
            let Some(pos) = cursor::current_position() else {
                continue;
            };
            let Some(win) = ensure_window(&app) else {
                continue;
            };
            let Some(placement) = display::position_overlay_at(&win, pos) else {
                continue;
            };

            // Double check click-through is enabled whenever showing
            let _ = win.set_ignore_cursor_events(true);
            let _ = win.show();

            let local_logical = (
                (pos.0 - placement.origin_x) / placement.scale,
                (pos.1 - placement.origin_y) / placement.scale,
            );
            let payload = IndicatorPayload {
                style,
                cursor_x: local_logical.0,
                cursor_y: local_logical.1,
                app_name: event.app_name.clone(),
                title: event.title.clone(),
            };
            let _ = app.emit("noti:show", payload);
        }
    });
}

fn ensure_window<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window("overlay")
}
