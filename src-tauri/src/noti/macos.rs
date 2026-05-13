#![cfg(target_os = "macos")]

use super::{NotiEvent, NotificationSource, Publisher};
use accessibility_sys::{
    kAXChildrenAttribute, kAXErrorSuccess,
    AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCreateApplication, AXUIElementRef,
};
use core_foundation::base::{CFRelease, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::{CFString, CFStringRef};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

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
        let mut seen: std::collections::HashSet<String> = Default::default();
        while *running.lock() {
            std::thread::sleep(Duration::from_millis(500));
            let titles = read_nc_titles().unwrap_or_default();
            for t in &titles {
                if !seen.contains(t) {
                    seen.insert(t.clone());
                    (publish)(NotiEvent::now(
                        "com.apple.notificationcenterui",
                        "Notification Center",
                        t.clone(),
                        String::new(),
                    ));
                }
            }
            if seen.len() > 100 {
                seen.clear();
            }
        }
    }
}

fn read_nc_titles() -> Option<Vec<String>> {
    let pid = MacosNotiSource::find_nc_pid()?;
    unsafe {
        let app_ref: AXUIElementRef = AXUIElementCreateApplication(pid);
        let mut value: core_foundation::base::CFTypeRef = std::ptr::null();
        let attr = CFString::new(kAXChildrenAttribute);
        let err = AXUIElementCopyAttributeValue(
            app_ref,
            attr.as_concrete_TypeRef() as CFStringRef,
            &mut value,
        );
        CFRelease(app_ref as _);
        if err != kAXErrorSuccess || value.is_null() {
            return None;
        }
        let titles = walk_titles(value as AXUIElementRef);
        CFRelease(value);
        Some(titles)
    }
}

unsafe fn walk_titles(_root: AXUIElementRef) -> Vec<String> {
    // MVP: simplified walker returns empty. Production would recurse and read kAXTitleAttribute.
    // Polling still gates on permission; this is intentionally minimal.
    Vec::new()
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
