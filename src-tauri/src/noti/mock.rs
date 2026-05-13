use super::{NotiEvent, NotificationSource, Publisher};
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct MockSource {
    publish: Arc<Mutex<Option<Arc<Publisher>>>>,
}

impl MockSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inject(&self, event: NotiEvent) {
        if let Some(p) = self.publish.lock().as_ref() {
            (p)(event);
        }
    }
}

impl NotificationSource for MockSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        *self.publish.lock() = Some(Arc::new(publish));
        Ok(())
    }

    fn stop(&self) {
        *self.publish.lock() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::Mutex;
    use std::sync::Arc;

    #[test]
    fn inject_routes_to_publisher() {
        let collected: Arc<Mutex<Vec<NotiEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = collected.clone();
        let m = MockSource::new();
        m.start(Box::new(move |e| sink.lock().push(e))).unwrap();
        m.inject(NotiEvent::now("a", "A", "t", "b"));
        assert_eq!(collected.lock().len(), 1);
        assert_eq!(collected.lock()[0].app_id, "a");
    }
}
