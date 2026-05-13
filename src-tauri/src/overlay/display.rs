use tauri::{Monitor, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

pub struct MonitorPlacement {
    pub origin_x: f64,
    pub origin_y: f64,
    pub scale: f64,
}

pub fn position_overlay_at<R: Runtime>(
    win: &WebviewWindow<R>,
    pos: (f64, f64),
) -> Option<MonitorPlacement> {
    let monitors = win.available_monitors().unwrap_or_default();
    let m = pick_monitor(&monitors, pos)?.clone();
    let mp = m.position();
    let ms = m.size();
    let _ = win.set_position(PhysicalPosition::new(mp.x, mp.y));
    let _ = win.set_size(PhysicalSize::new(ms.width, ms.height));
    Some(MonitorPlacement {
        origin_x: mp.x as f64,
        origin_y: mp.y as f64,
        scale: m.scale_factor(),
    })
}

fn pick_monitor<'a>(monitors: &'a [Monitor], pos: (f64, f64)) -> Option<&'a Monitor> {
    monitors.iter().find(|m| {
        let p = m.position();
        let s = m.size();
        let x = p.x as f64;
        let y = p.y as f64;
        pos.0 >= x && pos.0 < x + s.width as f64 && pos.1 >= y && pos.1 < y + s.height as f64
    })
}
