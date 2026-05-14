#![cfg(target_os = "macos")]

//! macOS mouse hotkey listener via `NSEvent.addGlobalMonitorForEventsMatchingMask`.
//!
//! Why not rdev? rdev's CGEventTap path converts every event (including
//! keyboard) via `TSMGetInputSourceProperty`, which asserts main-thread on
//! macOS 15+, crashing the app. The NSEvent global monitor runs on the
//! main runloop, observes mouse events without touching TSM, and only fires
//! when a *different* application has key focus — fine for a tray app.
//!
//! Permission: requires Accessibility (the system surface the prompt the
//! first time the monitor is attached).
//!
//! Scope: only listens to `OtherMouseDown` (mouse buttons 3+), so binding
//! left/right mouse is intentionally impossible — those are too dangerous
//! as global hotkeys.

use crate::hotkey::{SharedBindings, TriggerEvent};
use crate::settings::HotkeyTrigger;
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags};
use std::ptr::NonNull;
use tokio::sync::mpsc::UnboundedSender;

const MOD_CMD: u8 = 1;
const MOD_CTRL: u8 = 2;
const MOD_SHIFT: u8 = 4;
const MOD_ALT: u8 = 8;

/// NSEvent.buttonNumber → settings.rs button convention.
/// Cocoa:    0=left, 1=right, 2=middle, 3=back, 4=forward, …
/// Settings: 1=left, 2=right, 3=middle, 4=back, 5=forward, …
fn cocoa_button_to_setting(n: isize) -> u8 {
    if n < 0 {
        return 0;
    }
    (n + 1).clamp(0, u8::MAX as isize) as u8
}

fn flags_to_modifiers(f: NSEventModifierFlags) -> u8 {
    let mut out = 0u8;
    if f.contains(NSEventModifierFlags::NSEventModifierFlagCommand) {
        out |= MOD_CMD;
    }
    if f.contains(NSEventModifierFlags::NSEventModifierFlagControl) {
        out |= MOD_CTRL;
    }
    if f.contains(NSEventModifierFlags::NSEventModifierFlagShift) {
        out |= MOD_SHIFT;
    }
    if f.contains(NSEventModifierFlags::NSEventModifierFlagOption) {
        out |= MOD_ALT;
    }
    out
}

/// Holds the NSEvent monitor handle. Dropping the struct removes the
/// monitor via `NSEvent.removeMonitor:`.
pub struct MouseMonitor {
    handle: Option<Retained<AnyObject>>,
}

impl MouseMonitor {
    pub fn start(bindings: SharedBindings, trigger_tx: UnboundedSender<TriggerEvent>) -> Self {
        let mask = NSEventMask::OtherMouseDown;

        let block = RcBlock::new(move |event: NonNull<NSEvent>| {
            let ev = unsafe { event.as_ref() };
            let button_raw = unsafe { ev.buttonNumber() };
            let mods_raw = unsafe { ev.modifierFlags() };
            let button = cocoa_button_to_setting(button_raw);
            let modifiers = flags_to_modifiers(mods_raw);
            tracing::debug!(button, modifiers, "mouse event observed");

            let snap = bindings.lock();
            for b in snap.iter() {
                if let HotkeyTrigger::Mouse {
                    button: bb,
                    modifiers: mm,
                } = &b.trigger
                {
                    if *bb == button && *mm == modifiers {
                        let _ = trigger_tx.send(TriggerEvent {
                            binding_id: b.id.clone(),
                            menu_mode: b.menu_mode.clone(),
                        });
                        break;
                    }
                }
            }
        });

        let handle = unsafe {
            NSEvent::addGlobalMonitorForEventsMatchingMask_handler(mask, &block)
        };
        if handle.is_none() {
            tracing::warn!("NSEvent::addGlobalMonitorForEventsMatchingMask returned nil; mouse hotkeys inactive (likely missing Accessibility permission)");
        }
        Self { handle }
    }
}

impl Drop for MouseMonitor {
    fn drop(&mut self) {
        if let Some(h) = self.handle.take() {
            unsafe { NSEvent::removeMonitor(&h) };
        }
    }
}
