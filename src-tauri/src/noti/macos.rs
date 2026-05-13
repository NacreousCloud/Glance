#![cfg(target_os = "macos")]

use super::{NotiEvent, NotificationSource, Publisher};
use accessibility_sys::{
    kAXChildrenAttribute, kAXErrorSuccess, kAXTitleAttribute, AXIsProcessTrustedWithOptions,
    AXUIElementCopyAttributeValue, AXUIElementCreateApplication, AXUIElementRef,
};
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{CFRelease, CFType, CFTypeRef, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::{CFString, CFStringRef};
use parking_lot::Mutex;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

const MAX_DEPTH: usize = 30;
const SEEN_CAPACITY: usize = 512;
const POLL_INTERVAL: Duration = Duration::from_millis(500);
const CHROME_TITLES: &[&str] = &[
    "Notifications",
    "Notification Center",
    "Do Not Disturb",
    "Show notifications",
    "Clear all",
    "Today",
    "Earlier",
    "Yesterday",
    "Show",
    "Hide",
    "Close",
];

const NOTIFICATION_CENTER_BUNDLE_ID: &str = "com.apple.notificationcenterui";

pub struct MacosNotiSource {
    running: Arc<Mutex<bool>>,
}

impl Default for MacosNotiSource {
    fn default() -> Self {
        Self::new()
    }
}

impl MacosNotiSource {
    pub fn new() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_trusted(prompt: bool) -> bool {
        unsafe {
            let key = CFString::new("AXTrustedCheckOptionPrompt");
            let value = core_foundation::boolean::CFBoolean::from(prompt);
            let opts = CFDictionary::from_CFType_pairs(&[(key, value)]);
            AXIsProcessTrustedWithOptions(opts.as_concrete_TypeRef() as _)
        }
    }

    fn find_nc_pid() -> Option<i32> {
        use objc2_app_kit::NSWorkspace;
        let workspace = unsafe { NSWorkspace::sharedWorkspace() };
        let apps = unsafe { workspace.runningApplications() };
        let count = apps.count();
        for i in 0..count {
            let app = unsafe { apps.objectAtIndex(i) };
            let bundle_id = unsafe { app.bundleIdentifier() };
            if let Some(id) = bundle_id {
                if id.to_string() == NOTIFICATION_CENTER_BUNDLE_ID {
                    let pid: i32 = unsafe { app.processIdentifier() };
                    return Some(pid);
                }
            }
        }
        None
    }
}

impl NotificationSource for MacosNotiSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        if !Self::is_trusted(true) {
            anyhow::bail!("Accessibility permission not granted");
        }

        let pid = Self::find_nc_pid().ok_or_else(|| {
            tracing::error!(
                "Could not find Notification Center process (com.apple.notificationcenterui)"
            );
            anyhow::anyhow!("Notification Center not found")
        })?;
        tracing::info!(pid, "Found Notification Center process");

        *self.running.lock() = true;
        let running = self.running.clone();
        let publish = Arc::new(publish);

        std::thread::spawn(move || {
            tracing::info!(interval_ms = %POLL_INTERVAL.as_millis(), "macOS poll loop started");
            let mut seen_set: HashSet<String> = HashSet::new();
            let mut seen_order: VecDeque<String> = VecDeque::new();
            let mut tick: u64 = 0;

            while *running.lock() {
                std::thread::sleep(POLL_INTERVAL);
                tick += 1;

                let titles = unsafe { read_nc_titles(pid) }.unwrap_or_default();

                if tick % 20 == 0 {
                    tracing::debug!(
                        tick,
                        title_count = titles.len(),
                        seen = seen_set.len(),
                        "poll heartbeat"
                    );
                }

                for t in titles {
                    if CHROME_TITLES.iter().any(|c| *c == t) {
                        continue;
                    }
                    if seen_set.contains(&t) {
                        continue;
                    }
                    tracing::info!(title = %t, "new macOS notification title detected");
                    if seen_order.len() >= SEEN_CAPACITY {
                        if let Some(old) = seen_order.pop_front() {
                            seen_set.remove(&old);
                        }
                    }
                    seen_set.insert(t.clone());
                    seen_order.push_back(t.clone());
                    (publish)(NotiEvent::now(
                        "com.apple.notificationcenterui",
                        "Notification Center",
                        t,
                        String::new(),
                    ));
                }
            }
            tracing::info!("macOS poll loop stopped");
        });

        Ok(())
    }

    fn stop(&self) {
        *self.running.lock() = false;
    }
}

unsafe fn read_nc_titles(pid: i32) -> Option<Vec<String>> {
    let app_ref: AXUIElementRef = AXUIElementCreateApplication(pid);
    if app_ref.is_null() {
        return None;
    }
    let mut out = Vec::new();
    descend(app_ref, &mut out, 0);
    CFRelease(app_ref as _);
    Some(out)
}

unsafe fn descend(elem: AXUIElementRef, out: &mut Vec<String>, depth: usize) {
    if elem.is_null() || depth > MAX_DEPTH {
        return;
    }

    let title_attr = CFString::new(kAXTitleAttribute);
    let mut title_value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(
        elem,
        title_attr.as_concrete_TypeRef() as CFStringRef,
        &mut title_value,
    );
    if err == kAXErrorSuccess && !title_value.is_null() {
        let s = CFString::wrap_under_create_rule(title_value as CFStringRef);
        let text = s.to_string();
        if !text.is_empty() {
            out.push(text);
        }
    }

    let children_attr = CFString::new(kAXChildrenAttribute);
    let mut children_value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(
        elem,
        children_attr.as_concrete_TypeRef() as CFStringRef,
        &mut children_value,
    );
    if err == kAXErrorSuccess && !children_value.is_null() {
        let arr: CFArray<CFType> = CFArray::wrap_under_create_rule(children_value as CFArrayRef);
        for item in arr.iter() {
            let child = item.as_CFTypeRef() as AXUIElementRef;
            descend(child, out, depth + 1);
        }
    }
}
