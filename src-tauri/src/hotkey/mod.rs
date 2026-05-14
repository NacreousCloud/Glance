pub mod keyboard;

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub binding_id: String,
    pub menu_mode: String,
}
