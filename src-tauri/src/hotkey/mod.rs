pub mod keyboard;
pub mod mouse;

use crate::settings::HotkeyBinding;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub binding_id: String,
    pub menu_mode: String,
}

/// Shared, mutable set of hotkey bindings.
/// Live re-registration (Task 13) mutates this in place so the mouse
/// listener thread and rebind drainer always see the latest set without
/// restarting any listeners.
pub type SharedBindings = Arc<Mutex<Vec<HotkeyBinding>>>;
