use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotiEvent {
    pub app_id: String,
    pub app_name: String,
    pub title: String,
    pub body: String,
    pub timestamp_ms: u128,
}

impl NotiEvent {
    pub fn now(app_id: impl Into<String>, app_name: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        Self {
            app_id: app_id.into(),
            app_name: app_name.into(),
            title: title.into(),
            body: body.into(),
            timestamp_ms,
        }
    }
}

pub type Publisher = Box<dyn Fn(NotiEvent) + Send + Sync + 'static>;

pub trait NotificationSource: Send + Sync {
    fn start(&self, publish: Publisher) -> anyhow::Result<()>;
    fn stop(&self);
}

pub mod mock;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noti_event_roundtrip_json() {
        let e = NotiEvent::now("com.app", "App", "hi", "body");
        let json = serde_json::to_string(&e).unwrap();
        let parsed: NotiEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(e, parsed);
    }
}
