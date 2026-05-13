#![cfg(target_os = "macos")]

use super::{NotiEvent, NotificationSource, Publisher};
use accessibility_sys::{
    kAXChildrenAttribute, kAXCreatedNotification, kAXErrorSuccess, kAXTitleAttribute,
    AXIsProcessTrustedWithOptions, AXObserverAddNotification, AXObserverCreate,
    AXObserverGetRunLoopSource, AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
    AXUIElementRef,
};
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{CFRelease, CFType, CFTypeRef, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoopAddSource, CFRunLoopGetCurrent};
use core_foundation::string::{CFString, CFStringRef};
use parking_lot::Mutex;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

const MAX_DEPTH: usize = 30;
const SEEN_CAPACITY: usize = 512;
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

struct ObserverContext {
    publish: Arc<Publisher>,
    seen_set: HashSet<String>,
    seen_order: VecDeque<String>,
}

extern "C" fn observer_callback(
    _observer: accessibility_sys::AXObserverRef,
    element: AXUIElementRef,
    notification: CFStringRef,
    refcon: *mut std::ffi::c_void,
) {
    unsafe {
        let ctx = &mut *(refcon as *mut ObserverContext);
        let noti_str = CFString::wrap_under_get_rule(notification).to_string();
        tracing::debug!(notification = %noti_str, "AXObserver callback received");

        if noti_str == "AXCreated" || noti_str == "AXTitleChanged" {
            let mut titles = Vec::new();
            descend(element, &mut titles, 0);

            if titles.is_empty() {
                tracing::trace!("No titles found in elements");
            }

            for t in titles {
                if CHROME_TITLES.iter().any(|c| *c == t) {
                    continue;
                }
                if !ctx.seen_set.contains(&t) {
                    tracing::info!(title = %t, "New notification detected on macOS");
                    if ctx.seen_order.len() >= SEEN_CAPACITY {
                        if let Some(old) = ctx.seen_order.pop_front() {
                            ctx.seen_set.remove(&old);
                        }
                    }
                    ctx.seen_set.insert(t.clone());
                    ctx.seen_order.push_back(t.clone());
                    (ctx.publish)(NotiEvent::now(
                        "com.apple.notificationcenterui",
                        "Notification Center",
                        t,
                        String::new(),
                    ));
                }
            }
        }
    }
}

impl NotificationSource for MacosNotiSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        if !Self::is_trusted(true) {
            anyhow::bail!("Accessibility permission not granted");
        }

        let pid = Self::find_nc_pid().ok_or_else(|| {
            tracing::error!("Could not find Notification Center process (com.apple.notificationcenterui)");
            anyhow::anyhow!("Notification Center not found")
        })?;
        tracing::info!(pid, "Found Notification Center process");

        *self.running.lock() = true;
        let running = self.running.clone();
        let publish = Arc::new(publish);

        std::thread::spawn(move || unsafe {
            let mut observer = std::ptr::null_mut();
            let err = AXObserverCreate(pid, observer_callback, &mut observer);
            if err != kAXErrorSuccess {
                tracing::error!(error = err, "Failed to create AXObserver");
                return;
            }
            tracing::info!("AXObserver created successfully");

            let ctx = Box::into_raw(Box::new(ObserverContext {
                publish,
                seen_set: HashSet::new(),
                seen_order: VecDeque::new(),
            }));

            let app_ref = AXUIElementCreateApplication(pid);
            
            // Observe AXCreated
            let noti_created = CFString::new(kAXCreatedNotification);
            let err = AXObserverAddNotification(observer, app_ref, noti_created.as_concrete_TypeRef() as _, ctx as _);
            if err != kAXErrorSuccess {
                tracing::warn!(error = err, "Failed to add AXCreated notification observer");
            }

            // Observe AXTitleChanged
            let noti_title = CFString::new(accessibility_sys::kAXTitleChangedNotification);
            let err = AXObserverAddNotification(observer, app_ref, noti_title.as_concrete_TypeRef() as _, ctx as _);
            if err != kAXErrorSuccess {
                tracing::warn!(error = err, "Failed to add AXTitleChanged notification observer");
            }

            let source = AXObserverGetRunLoopSource(observer);
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopDefaultMode);
            tracing::info!("AXObserver added to RunLoop");

            while *running.lock() {
                core_foundation::runloop::CFRunLoopRunInMode(
                    kCFRunLoopDefaultMode,
                    Duration::from_millis(100).as_secs_f64(),
                    0,
                );
            }

            tracing::info!("Stopping macOS notification observer");
            CFRelease(app_ref as _);
            CFRelease(observer as _);
            let _ = Box::from_raw(ctx);
        });

        Ok(())
    }

    fn stop(&self) {
        *self.running.lock() = false;
    }
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
