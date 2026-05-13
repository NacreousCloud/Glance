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
    use core_graphics::event::CGEvent;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    pub fn current_position() -> Option<(f64, f64)> {
        let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
        let evt = CGEvent::new(src).ok()?;
        let loc = evt.location();
        Some((loc.x, loc.y))
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
