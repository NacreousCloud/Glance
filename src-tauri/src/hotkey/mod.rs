pub mod keyboard;
#[cfg(target_os = "macos")]
pub mod mouse;
#[cfg(target_os = "macos")]
#[path = "gestures_macos.rs"]
pub mod gestures;
#[cfg(target_os = "macos")]
#[path = "trackpad_macos.rs"]
pub mod trackpad;

#[cfg(target_os = "windows")]
#[path = "mouse_windows.rs"]
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
