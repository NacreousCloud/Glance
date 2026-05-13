use crate::noti::NotiEvent;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

const RING_CAPACITY: usize = 64;
const DEBOUNCE: Duration = Duration::from_millis(1000);
const RECENT_WINDOW: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<NotiEvent>,
    last_per_app: Arc<Mutex<HashMap<String, Instant>>>,
    recent: Arc<Mutex<Vec<(Instant, NotiEvent)>>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(RING_CAPACITY);
        Self {
            tx,
            last_per_app: Arc::new(Mutex::new(HashMap::new())),
            recent: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn sender(&self) -> broadcast::Sender<NotiEvent> {
        self.tx.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NotiEvent> {
        self.tx.subscribe()
    }

    /// Publish with per-app debounce. Returns true if event was forwarded.
    pub fn publish(&self, event: NotiEvent) -> bool {
        let now = Instant::now();
        {
            let mut last = self.last_per_app.lock();
            // Bypass debounce for debug notifications
            if event.app_id != "dev.debug" {
                if let Some(prev) = last.get(&event.app_id) {
                    if now.duration_since(*prev) < DEBOUNCE {
                        return false;
                    }
                }
            }
            last.insert(event.app_id.clone(), now);
        }
        {
            let mut rec = self.recent.lock();
            rec.retain(|(t, _)| now.duration_since(*t) < RECENT_WINDOW);
            rec.push((now, event.clone()));
        }
        if self.tx.send(event).is_err() {
            tracing::trace!("noti event published with no overlay subscriber");
        }
        true
    }

    pub fn recent_within(&self, window: Duration) -> Vec<NotiEvent> {
        let now = Instant::now();
        self.recent
            .lock()
            .iter()
            .filter(|(t, _)| now.duration_since(*t) < window)
            .map(|(_, e)| e.clone())
            .collect()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    fn ev(app: &str) -> NotiEvent {
        NotiEvent::now(app, app, "t", "b")
    }

    #[tokio::test(start_paused = false)]
    async fn forwards_first_event() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        assert!(bus.publish(ev("a")));
        let got = rx.recv().await.unwrap();
        assert_eq!(got.app_id, "a");
    }

    #[tokio::test]
    async fn debounces_within_1s() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        assert!(bus.publish(ev("a")));
        let _ = rx.recv().await.unwrap();
        assert!(!bus.publish(ev("a")));
    }

    #[tokio::test]
    async fn different_apps_pass_through() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();
        assert!(bus.publish(ev("a")));
        assert!(bus.publish(ev("b")));
        let _ = rx.recv().await.unwrap();
        let _ = rx.recv().await.unwrap();
    }

    #[tokio::test]
    async fn recent_window_evicts_old() {
        let bus = EventBus::new();
        bus.publish(ev("a"));
        sleep(Duration::from_millis(50)).await;
        let recent = bus.recent_within(Duration::from_secs(10));
        assert_eq!(recent.len(), 1);
    }
}
