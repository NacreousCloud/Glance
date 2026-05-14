pub mod keyboard;
pub mod mouse;

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub binding_id: String,
    pub menu_mode: String,
}
