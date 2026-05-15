#![cfg(target_os = "macos")]

//! Trackpad multi-finger tap detection via the private
//! `MultitouchSupport.framework`.
//!
//! Loaded with `dlopen` so we don't take a hard link-time dependency
//! on a private symbol set. The framework exposes a contact-frame
//! callback that fires repeatedly while fingers are on the trackpad
//! and once more when all fingers lift. A "tap" is detected by
//! tracking the peak simultaneous finger count over the lifetime of
//! a contact group; when the group ends (`n_fingers == 0`) and the
//! contact lasted under the configured duration, we emit a trigger.
//!
//! Risks:
//! - Private API. Symbols may disappear or change semantics on
//!   future macOS versions.
//! - App Store distribution is impossible while this code is linked.
//!   This is acceptable for self-distributed builds only.

use crate::hotkey::{SharedBindings, TriggerEvent};
use crate::settings::HotkeyTrigger;
use libloading::{Library, Symbol};
use parking_lot::Mutex;
use std::os::raw::{c_double, c_int, c_void};
use std::sync::OnceLock;
use std::thread;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

type MTDeviceRef = *mut c_void;

// Callback signature exposed by MultitouchSupport.
type ContactCallback = unsafe extern "C" fn(
    device: MTDeviceRef,
    fingers: *mut c_void,
    n_fingers: c_int,
    timestamp: c_double,
    frame: c_int,
) -> c_int;

type MTDeviceCreateDefault = unsafe extern "C" fn() -> MTDeviceRef;
type MTRegisterContactFrameCallback =
    unsafe extern "C" fn(device: MTDeviceRef, cb: ContactCallback);
type MTDeviceStart = unsafe extern "C" fn(device: MTDeviceRef, run_mode: c_int);

struct ContactState {
    peak_fingers: u8,
    started_at: Option<Instant>,
}

static STATE: OnceLock<Mutex<ContactState>> = OnceLock::new();
static TX: OnceLock<Mutex<UnboundedSender<TriggerEvent>>> = OnceLock::new();
static BINDINGS: OnceLock<SharedBindings> = OnceLock::new();
// Keep the dlopen'd library alive for the process lifetime so the
// callback function pointer stays valid.
static LIB: OnceLock<Library> = OnceLock::new();

unsafe extern "C" fn contact_callback(
    _device: MTDeviceRef,
    _fingers: *mut c_void,
    n_fingers: c_int,
    _timestamp: c_double,
    _frame: c_int,
) -> c_int {
    let count = n_fingers.max(0) as u8;
    let mut state = STATE.get().expect("STATE init").lock();
    if count == 0 {
        // All fingers lifted. Decide whether this was a tap.
        if let Some(started) = state.started_at.take() {
            let dur = started.elapsed();
            let peak = state.peak_fingers;
            state.peak_fingers = 0;
            drop(state);
            try_dispatch(peak, dur.as_millis() as u32);
        } else {
            state.peak_fingers = 0;
        }
    } else {
        if state.started_at.is_none() {
            state.started_at = Some(Instant::now());
        }
        if count > state.peak_fingers {
            state.peak_fingers = count;
        }
    }
    0
}

fn try_dispatch(peak: u8, duration_ms: u32) {
    let Some(bindings) = BINDINGS.get() else { return };
    let Some(tx) = TX.get() else { return };
    let snap = bindings.lock();
    for b in snap.iter() {
        if let HotkeyTrigger::TrackpadTap {
            fingers,
            max_duration_ms,
        } = &b.trigger
        {
            if peak == *fingers && duration_ms <= *max_duration_ms {
                tracing::debug!(peak, duration_ms, "trackpad tap matched");
                let _ = tx.lock().send(TriggerEvent {
                    binding_id: b.id.clone(),
                    menu_mode: b.menu_mode.clone(),
                });
                break;
            }
        }
    }
}

pub struct TrackpadMonitor;

impl TrackpadMonitor {
    pub fn start(bindings: SharedBindings, trigger_tx: UnboundedSender<TriggerEvent>) -> Self {
        let _ = STATE.set(Mutex::new(ContactState {
            peak_fingers: 0,
            started_at: None,
        }));
        if BINDINGS.set(bindings).is_err() {
            tracing::warn!("TrackpadMonitor::start called twice");
            return Self;
        }
        let _ = TX.set(Mutex::new(trigger_tx));

        thread::Builder::new()
            .name("glance-trackpad".into())
            .spawn(|| unsafe {
                // dlopen the framework. Path is stable across recent macOS
                // versions; the framework itself sits in /System/Library.
                let path = "/System/Library/PrivateFrameworks/MultitouchSupport.framework/MultitouchSupport";
                let lib = match Library::new(path) {
                    Ok(l) => l,
                    Err(e) => {
                        tracing::warn!(error = %e, "failed to dlopen MultitouchSupport; trackpad-tap hotkeys disabled");
                        return;
                    }
                };

                let create: Symbol<MTDeviceCreateDefault> =
                    match lib.get(b"MTDeviceCreateDefault") {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::warn!(error = %e, "MTDeviceCreateDefault missing");
                            return;
                        }
                    };
                let register: Symbol<MTRegisterContactFrameCallback> =
                    match lib.get(b"MTRegisterContactFrameCallback") {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::warn!(error = %e, "MTRegisterContactFrameCallback missing");
                            return;
                        }
                    };
                let start: Symbol<MTDeviceStart> = match lib.get(b"MTDeviceStart") {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!(error = %e, "MTDeviceStart missing");
                        return;
                    }
                };

                let device = create();
                if device.is_null() {
                    tracing::warn!("MTDeviceCreateDefault returned null; no trackpad?");
                    return;
                }
                register(device, contact_callback);
                start(device, 0);
                tracing::info!("trackpad multitouch monitor installed");

                // Stash lib so it's not dropped + dylib unmapped while
                // the registered callback function pointer is still in
                // use by the system.
                let _ = LIB.set(lib);

                // The callback fires on whatever runloop the framework
                // hooks. Park the thread so the OnceLock keeps the
                // library alive indefinitely.
                loop {
                    thread::park();
                }
            })
            .expect("spawn trackpad thread");
        Self
    }
}
