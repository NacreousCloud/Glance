#![cfg(target_os = "macos")]

use super::{NotiEvent, NotificationSource, Publisher};
use accessibility_sys::{
    kAXChildrenAttribute, kAXErrorSuccess, kAXTitleAttribute,
    AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCreateApplication, AXUIElementRef,
};
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{CFRelease, CFType, CFTypeRef, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::{CFString, CFStringRef};
use parking_lot::Mutex;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

/// Maximum AX-tree recursion depth. Notification Center's tree is typically
/// shallow (a handful of levels); 30 is far above realistic worst case and
/// guards against pathological / cyclic trees.
const MAX_DEPTH: usize = 30;

/// Cap on the "seen titles" set. When we exceed this, we drop oldest entries
/// (FIFO) instead of clearing wholesale, so we don't re-fire already-seen
/// notifications.
const SEEN_CAPACITY: usize = 512;

/// Deny-list of well-known Notification Center chrome titles. These are
/// surfaced by the AX tree alongside real notification banners and would
/// otherwise be emitted as fake NotiEvents. This list is necessarily
/// best-effort and may need updates across macOS versions.
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

    fn poll_loop(publish: Arc<Publisher>, running: Arc<Mutex<bool>>) {
        // FIFO + set pair: O(1) contains via HashSet, insertion order via
        // VecDeque so we can evict the oldest entry once we hit capacity.
        let mut seen_set: HashSet<String> = HashSet::new();
        let mut seen_order: VecDeque<String> = VecDeque::new();
        while *running.lock() {
            std::thread::sleep(Duration::from_millis(500));
            let titles = read_nc_titles().unwrap_or_default();
            for t in &titles {
                if CHROME_TITLES.iter().any(|c| *c == t) {
                    continue;
                }
                if !seen_set.contains(t) {
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
                        t.clone(),
                        String::new(),
                    ));
                }
            }
        }
    }
}

fn read_nc_titles() -> Option<Vec<String>> {
    let pid = MacosNotiSource::find_nc_pid()?;
    unsafe {
        let app_ref: AXUIElementRef = AXUIElementCreateApplication(pid);
        if app_ref.is_null() {
            return None;
        }
        let titles = walk_titles(app_ref);
        CFRelease(app_ref as _);
        Some(titles)
    }
}

/// Recursively walk the Accessibility tree rooted at `root`, collecting every
/// non-empty `kAXTitleAttribute` value reachable via `kAXChildrenAttribute`.
///
/// # Safety
/// `root` must be a valid `AXUIElementRef` (or null — handled). Caller retains
/// ownership of `root`; this function does not release it.
unsafe fn walk_titles(root: AXUIElementRef) -> Vec<String> {
    let mut out = Vec::new();
    descend(root, &mut out, 0);
    out
}

unsafe fn descend(elem: AXUIElementRef, out: &mut Vec<String>, depth: usize) {
    if elem.is_null() || depth > MAX_DEPTH {
        return;
    }

    // Read kAXTitleAttribute if present.
    let title_attr = CFString::new(kAXTitleAttribute);
    let mut title_value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(
        elem,
        title_attr.as_concrete_TypeRef() as CFStringRef,
        &mut title_value,
    );
    if err == kAXErrorSuccess && !title_value.is_null() {
        // The +1 retain from CopyAttributeValue is consumed by wrap_under_create_rule.
        let s = CFString::wrap_under_create_rule(title_value as CFStringRef);
        let text = s.to_string();
        if !text.is_empty() {
            out.push(text);
        }
    }

    // Read kAXChildrenAttribute and recurse.
    let children_attr = CFString::new(kAXChildrenAttribute);
    let mut children_value: CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(
        elem,
        children_attr.as_concrete_TypeRef() as CFStringRef,
        &mut children_value,
    );
    if err == kAXErrorSuccess && !children_value.is_null() {
        // Consumes the +1 retain. The CFArray owns the AXUIElementRefs it
        // holds — we recurse with the array alive on the stack frame so the
        // child refs remain valid throughout the for-loop.
        let arr: CFArray<CFType> =
            CFArray::wrap_under_create_rule(children_value as CFArrayRef);
        for item in arr.iter() {
            let child = item.as_CFTypeRef() as AXUIElementRef;
            descend(child, out, depth + 1);
        }
    }
}

impl NotificationSource for MacosNotiSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        if !Self::is_trusted(true) {
            anyhow::bail!("Accessibility permission not granted");
        }
        *self.running.lock() = true;
        let running = self.running.clone();
        let publish = Arc::new(publish);
        std::thread::spawn(move || Self::poll_loop(publish, running));
        Ok(())
    }

    fn stop(&self) {
        *self.running.lock() = false;
    }
}
