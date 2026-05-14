pub mod keyboard;
// Mouse hotkey listener (rdev-based) is disabled on macOS due to a
// TSMGetInputSourceProperty main-thread assertion failure inside rdev's
// keyboard conversion path. Tracked as a follow-up to re-implement via
// NSEvent.addGlobalMonitorForEventsMatchingMask.

use crate::settings::HotkeyBinding;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub binding_id: String,
    pub menu_mode: String,
}

/// Shared, mutable set of hotkey bindings. Mutated by CRUD commands; read
/// by the keyboard rebind drainer on each rebind signal.
pub type SharedBindings = Arc<Mutex<Vec<HotkeyBinding>>>;
