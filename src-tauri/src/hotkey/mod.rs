pub mod keyboard;
#[cfg(target_os = "macos")]
pub mod mouse;

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
