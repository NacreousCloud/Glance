#![cfg(target_os = "macos")]

use super::{NotiEvent, NotificationSource, Publisher};
use accessibility_sys::{
    kAXChildrenAttribute, kAXDescriptionAttribute, kAXErrorSuccess, kAXHelpAttribute,
    kAXRoleAttribute, kAXTitleAttribute, kAXValueAttribute, AXIsProcessTrustedWithOptions,
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

// Bundle IDs / executable names that host notification UI on macOS.
// Different macOS versions split banner / drawer / agent across processes.
const CANDIDATE_BUNDLE_IDS: &[&str] = &[
    "com.apple.notificationcenterui",
    "com.apple.UserNotificationCenter",
    "com.apple.nbagent",
    "com.apple.usernoted",
];
const CANDIDATE_EXEC_NAMES: &[&str] = &[
    "nbagent",
    "NotificationCenter",
    "UserNotificationCenter",
];

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

    fn find_candidate_pids() -> Vec<(i32, String)> {
        use objc2_app_kit::NSWorkspace;
        let workspace = unsafe { NSWorkspace::sharedWorkspace() };
        let apps = unsafe { workspace.runningApplications() };
        let count = apps.count();
        let mut out = Vec::new();
        for i in 0..count {
            let app = unsafe { apps.objectAtIndex(i) };
            let bundle_id = unsafe { app.bundleIdentifier() }.map(|s| s.to_string());
            let exec_url = unsafe { app.executableURL() };
            let exec_name = exec_url.and_then(|u| {
                let path = unsafe { u.path() }?.to_string();
                path.rsplit('/').next().map(|s| s.to_string())
            });

            let bundle_match = bundle_id
                .as_deref()
                .map(|id| CANDIDATE_BUNDLE_IDS.contains(&id))
                .unwrap_or(false);
            let exec_match = exec_name
                .as_deref()
                .map(|n| CANDIDATE_EXEC_NAMES.contains(&n))
                .unwrap_or(false);

            if bundle_match || exec_match {
                let pid: i32 = unsafe { app.processIdentifier() };
                let label = bundle_id.or(exec_name).unwrap_or_else(|| format!("pid:{pid}"));
                out.push((pid, label));
            }
        }
        out
    }
}

impl NotificationSource for MacosNotiSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        if !Self::is_trusted(true) {
            anyhow::bail!("Accessibility permission not granted");
        }

        let candidates = Self::find_candidate_pids();
        if candidates.is_empty() {
            tracing::error!("No notification UI host process found");
            anyhow::bail!("No notification host process found");
        }
        for (pid, label) in &candidates {
            tracing::info!(pid, %label, "Tracking notification host process");
        }

        *self.running.lock() = true;
        let running = self.running.clone();
        let publish = Arc::new(publish);

        std::thread::spawn(move || {
            tracing::info!(interval_ms = %POLL_INTERVAL.as_millis(), candidate_count = candidates.len(), "macOS poll loop started");
            let mut seen_set: HashSet<String> = HashSet::new();
            let mut seen_order: VecDeque<String> = VecDeque::new();
            let mut tick: u64 = 0;

            while *running.lock() {
                std::thread::sleep(POLL_INTERVAL);
                tick += 1;

                let mut titles: Vec<String> = Vec::new();
                for (pid, _label) in &candidates {
                    if let Some(t) = unsafe { read_nc_titles(*pid) } {
                        titles.extend(t);
                    }
                }

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

// Read attributes that may carry notification text. Banners typically use
// AXDescription or AXValue, NOT AXTitle.
const TEXT_ATTRS: &[&str] = &[
    kAXTitleAttribute,
    kAXDescriptionAttribute,
    kAXValueAttribute,
    kAXHelpAttribute,
];

unsafe fn read_string_attr(elem: AXUIElementRef, attr: &str) -> Option<String> {
    let cf_attr = CFString::new(attr);
    let mut value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(
        elem,
        cf_attr.as_concrete_TypeRef() as CFStringRef,
        &mut value,
    );
    if err != kAXErrorSuccess || value.is_null() {
        return None;
    }
    // Try as CFString; if not a string, drop the ref.
    let cf_type = core_foundation::base::CFType::wrap_under_create_rule(value);
    cf_type
        .downcast::<CFString>()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

unsafe fn descend(elem: AXUIElementRef, out: &mut Vec<String>, depth: usize) {
    if elem.is_null() || depth > MAX_DEPTH {
        return;
    }

    // Diagnostic at root: log role + texts to help locate banner element.
    let log_diag = depth <= 2 && tracing::enabled!(tracing::Level::TRACE);
    if log_diag {
        let role = read_string_attr(elem, kAXRoleAttribute).unwrap_or_default();
        let texts: Vec<String> = TEXT_ATTRS
            .iter()
            .filter_map(|a| read_string_attr(elem, a).map(|s| format!("{}={}", a, s)))
            .collect();
        if !texts.is_empty() {
            tracing::trace!(depth, role = %role, attrs = ?texts, "AX element");
        }
    }

    for attr in TEXT_ATTRS {
        if let Some(text) = read_string_attr(elem, attr) {
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
