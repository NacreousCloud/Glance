use tauri::{LogicalPosition, LogicalSize, Monitor, Runtime, WebviewWindow};

pub fn position_overlay_at<R: Runtime>(win: &WebviewWindow<R>, pos: (f64, f64)) {
    let monitors = win.available_monitors().unwrap_or_default();
    let scale = win.scale_factor().unwrap_or(1.0);
    let logical = (pos.0 / scale, pos.1 / scale);
    let monitor = pick_monitor(&monitors, logical).cloned();
    if let Some(m) = monitor {
        let mp = m.position();
        let ms = m.size();
        let lp = LogicalPosition::new(mp.x as f64 / scale, mp.y as f64 / scale);
        let ls = LogicalSize::new(ms.width as f64 / scale, ms.height as f64 / scale);
        let _ = win.set_position(lp);
        let _ = win.set_size(ls);
    }
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
