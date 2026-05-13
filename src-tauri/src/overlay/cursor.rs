pub fn current_position() -> Option<(f64, f64)> {
    platform::current_position()
}

#[cfg(target_os = "windows")]
mod platform {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    pub fn current_position() -> Option<(f64, f64)> {
        let mut pt = POINT::default();
        unsafe {
            if GetCursorPos(&mut pt).is_ok() {
                Some((pt.x as f64, pt.y as f64))
            } else {
                None
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use core_graphics::display::CGDisplay;
    use core_graphics::event::CGEvent;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    pub fn current_position() -> Option<(f64, f64)> {
        let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
        let evt = CGEvent::new(src).ok()?;
        let loc = evt.location();
        // CGEvent.location is in points; multiply by main display scale to get pixels.
        // NOTE: This uses the MAIN display's scale, not the cursor's containing display.
        // For typical setups (all-Retina or all-non-Retina) this is correct. Mixed-DPI
        // setups (e.g. Retina laptop + non-Retina external) will over-scale the cursor
        // on the external monitor. Fixing requires looking up the display containing the
        // cursor and using its pixel scale. Accepted for MVP.
        let scale = main_display_scale();
        Some((loc.x * scale, loc.y * scale))
    }

    fn main_display_scale() -> f64 {
        // CGDisplay-based scale lookup. Avoids AppKit's NSScreen, which requires
        // a MainThreadMarker — our cursor loop runs on a worker thread.
        let display = CGDisplay::main();
        let mode = match display.display_mode() {
            Some(m) => m,
            None => return 1.0,
        };
        let logical_w = mode.width() as f64;
        let pixel_w = mode.pixel_width() as f64;
        if logical_w > 0.0 {
            pixel_w / logical_w
        } else {
            1.0
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    pub fn current_position() -> Option<(f64, f64)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::current_position;

    #[test]
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn returns_some_on_host() {
        let pos = current_position();
        assert!(pos.is_some(), "expected cursor position on host OS");
    }
}
