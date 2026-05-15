#![cfg(target_os = "macos")]

//! macOS trackpad gesture listeners (currently: Force Touch).
//!
//! Uses `NSEvent.addGlobalMonitorForEventsMatchingMask` so the same
//! threading constraints as `mouse.rs` apply. Force Click is detected
//! by watching pressure events for the transition into stage 2
//! (`stage == 2` is the moment the OS announces a "force click").

use crate::hotkey::{SharedBindings, TriggerEvent};
use crate::settings::HotkeyTrigger;
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSEvent, NSEventMask};
use std::cell::Cell;
use std::ptr::NonNull;
use tokio::sync::mpsc::UnboundedSender;

pub struct GestureMonitor {
    handle: Option<Retained<AnyObject>>,
}

impl GestureMonitor {
    pub fn start(bindings: SharedBindings, trigger_tx: UnboundedSender<TriggerEvent>) -> Self {
        // Pressure events report `stage` 0 → 1 → 2. We only fire on the
        // upward transition into 2 so a single Force Click does not
        // generate a flood of triggers as the user keeps pressing.
        let last_stage = Cell::new(-1isize);

        let block = RcBlock::new(move |event: NonNull<NSEvent>| {
            let ev = unsafe { event.as_ref() };
            let stage = unsafe { ev.stage() };
            let prev = last_stage.get();
            last_stage.set(stage);
            if !(prev < 2 && stage >= 2) {
                return;
            }
            tracing::debug!(stage, "force-touch transition observed");

            let snap = bindings.lock();
            for b in snap.iter() {
                if matches!(b.trigger, HotkeyTrigger::ForceTouch) {
                    let _ = trigger_tx.send(TriggerEvent {
                        binding_id: b.id.clone(),
                        menu_mode: b.menu_mode.clone(),
                    });
                    break;
                }
            }
        });

        let handle = unsafe {
            NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
                NSEventMask::Pressure,
                &block,
            )
        };
        if handle.is_none() {
            tracing::warn!("NSEvent pressure global monitor returned nil; Force Touch hotkey inactive");
        }
        Self { handle }
    }
}

impl Drop for GestureMonitor {
    fn drop(&mut self) {
        if let Some(h) = self.handle.take() {
            unsafe { NSEvent::removeMonitor(&h) };
        }
    }
}
