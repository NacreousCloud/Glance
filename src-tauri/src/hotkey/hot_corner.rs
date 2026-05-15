//! Hot-corner trigger.
//!
//! Polls the cursor position on a background thread. When the cursor
//! enters the configured corner zone — and only on the *transition*
//! from "outside" to "inside" — fires the matching binding. Cursor
//! must leave the zone before another trigger can fire, otherwise
//! parking the mouse in the corner would re-fire forever.

use crate::hotkey::{SharedBindings, TriggerEvent};
use crate::overlay::cursor;
use crate::settings::{Corner, HotkeyTrigger};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::mpsc::UnboundedSender;

const POLL_INTERVAL: Duration = Duration::from_millis(80);

pub struct HotCornerMonitor;

impl HotCornerMonitor {
    pub fn start<R: Runtime>(
        app: AppHandle<R>,
        bindings: SharedBindings,
        trigger_tx: UnboundedSender<TriggerEvent>,
    ) -> Self {
        thread::Builder::new()
            .name("glance-hot-corner".into())
            .spawn(move || {
                // Per-binding "currently inside zone" flag. Set on entry,
                // cleared once the cursor leaves so a parked cursor cannot
                // re-fire.
                let mut inside: HashMap<String, bool> = HashMap::new();
                loop {
                    thread::sleep(POLL_INTERVAL);
                    let Some(pos) = cursor::current_position() else {
                        continue;
                    };
                    let Some(rect) = primary_monitor_rect(&app) else {
                        continue;
                    };

                    let snap = bindings.lock().clone();
                    for b in &snap {
                        if let HotkeyTrigger::HotCorner { corner, radius_px } = &b.trigger {
                            let hit = cursor_in_corner(pos, &rect, *corner, *radius_px);
                            let prev = inside.get(&b.id).copied().unwrap_or(false);
                            if hit && !prev {
                                tracing::debug!(binding = %b.id, ?corner, "hot corner entered");
                                let _ = trigger_tx.send(TriggerEvent {
                                    binding_id: b.id.clone(),
                                    menu_mode: b.menu_mode.clone(),
                                });
                            }
                            inside.insert(b.id.clone(), hit);
                        }
                    }
                    // Drop stale entries for deleted bindings so the map
                    // does not grow unboundedly.
                    inside.retain(|id, _| snap.iter().any(|b| &b.id == id));
                }
            })
            .expect("spawn hot-corner thread");
        Self
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn primary_monitor_rect<R: Runtime>(app: &AppHandle<R>) -> Option<Rect> {
    let win = app.get_webview_window("overlay")?;
    let m = win.primary_monitor().ok()??;
    let p = m.position();
    let s = m.size();
    Some(Rect {
        x: p.x as f64,
        y: p.y as f64,
        w: s.width as f64,
        h: s.height as f64,
    })
}

fn cursor_in_corner(pos: (f64, f64), rect: &Rect, corner: Corner, radius: u32) -> bool {
    let r = radius as f64;
    let (cx, cy) = pos;
    match corner {
        Corner::TopLeft => cx <= rect.x + r && cy <= rect.y + r,
        Corner::TopRight => cx >= rect.x + rect.w - r && cy <= rect.y + r,
        Corner::BottomLeft => cx <= rect.x + r && cy >= rect.y + rect.h - r,
        Corner::BottomRight => cx >= rect.x + rect.w - r && cy >= rect.y + rect.h - r,
    }
}
