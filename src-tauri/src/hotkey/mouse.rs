use crate::hotkey::TriggerEvent;
use crate::settings::HotkeyTrigger;
use parking_lot::Mutex;
use rdev::{listen, Button, Event, EventType};
use std::sync::Arc;

#[derive(Default, Clone, Copy)]
struct ModState {
    cmd: bool,
    ctrl: bool,
    shift: bool,
    alt: bool,
}

impl ModState {
    fn bits(&self) -> u8 {
        (self.cmd as u8)
            | ((self.ctrl as u8) << 1)
            | ((self.shift as u8) << 2)
            | ((self.alt as u8) << 3)
    }
}

fn button_to_u8(b: Button) -> u8 {
    match b {
        Button::Left => 1,
        Button::Right => 2,
        Button::Middle => 3,
        Button::Unknown(n) => 3 + n,
    }
}

fn update_mod(m: &mut ModState, k: rdev::Key, pressed: bool) {
    use rdev::Key;
    match k {
        Key::MetaLeft | Key::MetaRight => m.cmd = pressed,
        Key::ControlLeft | Key::ControlRight => m.ctrl = pressed,
        Key::ShiftLeft | Key::ShiftRight => m.shift = pressed,
        Key::Alt | Key::AltGr => m.alt = pressed,
        _ => {}
    }
}

/// Spawn a dedicated OS thread that listens for global mouse + key events.
/// On any ButtonPress, walk `bindings` looking for a matching (button, modifiers)
/// and send a TriggerEvent.
///
/// `bindings` is shared so Task 13 (live rebind) can mutate it without
/// restarting the listener thread.
pub fn spawn_listener(
    bindings: super::SharedBindings,
    tx: tokio::sync::mpsc::UnboundedSender<TriggerEvent>,
) {
    std::thread::spawn(move || {
        let mods: Arc<Mutex<ModState>> = Arc::new(Mutex::new(ModState::default()));
        let mods_for_cb = mods.clone();
        if let Err(e) = listen(move |event: Event| match event.event_type {
            EventType::KeyPress(k) => {
                let mut m = mods_for_cb.lock();
                update_mod(&mut m, k, true);
            }
            EventType::KeyRelease(k) => {
                let mut m = mods_for_cb.lock();
                update_mod(&mut m, k, false);
            }
            EventType::ButtonPress(b) => {
                let pressed_button = button_to_u8(b);
                let current_mods = mods_for_cb.lock().bits();
                let snapshot = bindings.lock().clone();
                for binding in &snapshot {
                    if let HotkeyTrigger::Mouse { button, modifiers } = &binding.trigger {
                        if *button == pressed_button && *modifiers == current_mods {
                            let _ = tx.send(TriggerEvent {
                                binding_id: binding.id.clone(),
                                menu_mode: binding.menu_mode.clone(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }) {
            tracing::error!(error = ?e, "rdev::listen failed");
        }
    });
}
