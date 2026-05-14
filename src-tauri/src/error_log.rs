use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

const CAPACITY: usize = 50;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorEntry {
    pub id: String,
    pub timestamp_ms: u64,
    pub item_id: String,
    pub item_label: String,
    pub message: String,
}

/// Bounded ring buffer of recent exec failures so the user can review them
/// in Settings after the on-screen badge fades.
#[derive(Clone)]
pub struct ErrorLog {
    inner: Arc<Mutex<VecDeque<ErrorEntry>>>,
}

impl ErrorLog {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(CAPACITY))),
        }
    }

    pub fn push(&self, entry: ErrorEntry) {
        let mut q = self.inner.lock();
        if q.len() >= CAPACITY {
            q.pop_front();
        }
        q.push_back(entry);
    }

    /// Newest first.
    pub fn snapshot(&self) -> Vec<ErrorEntry> {
        self.inner.lock().iter().rev().cloned().collect()
    }

    pub fn clear(&self) {
        self.inner.lock().clear();
    }
}

impl Default for ErrorLog {
    fn default() -> Self {
        Self::new()
    }
}
