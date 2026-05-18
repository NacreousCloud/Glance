#![cfg(target_os = "macos")]

use super::{NotiEvent, NotificationSource, Publisher};
use accessibility_sys::{
    kAXChildrenAttribute, kAXDescriptionAttribute, kAXErrorSuccess, kAXTitleAttribute,
    AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
};
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::{CFType, CFTypeRef, TCFType};
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::Arc;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_millis(150);

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFArrayRef;
}
const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;
const K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;
const K_CG_NULL_WINDOW_ID: u32 = 0;

const CANDIDATE_BUNDLE_IDS: &[&str] = &[
    "com.apple.notificationcenterui",
    "com.apple.UserNotificationCenter",
    "com.apple.nbagent",
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

    /// Kept for the permission UI. CGWindowList capture does not require
    /// Accessibility; this still reports the trust state so the permission
    /// command compiles unchanged.
    pub fn is_trusted(prompt: bool) -> bool {
        unsafe {
            let key = CFString::new("AXTrustedCheckOptionPrompt");
            let value = core_foundation::boolean::CFBoolean::from(prompt);
            let opts = CFDictionary::from_CFType_pairs(&[(key, value)]);
            AXIsProcessTrustedWithOptions(opts.as_concrete_TypeRef() as _)
        }
    }

    fn find_candidate_pids() -> HashMap<i32, String> {
        use objc2_app_kit::NSWorkspace;
        let workspace = unsafe { NSWorkspace::sharedWorkspace() };
        let apps = unsafe { workspace.runningApplications() };
        let mut out = HashMap::new();
        for i in 0..apps.count() {
            let app = unsafe { apps.objectAtIndex(i) };
            let bundle_id = unsafe { app.bundleIdentifier() }.map(|s| s.to_string());
            let exec_name = unsafe { app.executableURL() }.and_then(|u| {
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
                let label = bundle_id
                    .or(exec_name)
                    .unwrap_or_else(|| format!("pid:{pid}"));
                out.insert(pid, label);
            }
        }
        out
    }
}

#[derive(Debug)]
struct WindowInfo {
    number: u32,
    pid: i32,
    owner_name: String,
}

unsafe fn copy_window_list() -> Vec<WindowInfo> {
    let raw = CGWindowListCopyWindowInfo(
        K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY | K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS,
        K_CG_NULL_WINDOW_ID,
    );
    if raw.is_null() {
        return Vec::new();
    }
    let arr: CFArray<CFType> = CFArray::wrap_under_create_rule(raw);
    let mut out = Vec::with_capacity(arr.len() as usize);
    for item in arr.iter() {
        let dict_ref = item.as_CFTypeRef() as CFDictionaryRef;
        let number = read_u32_attr(dict_ref, "kCGWindowNumber").unwrap_or(0);
        let pid = read_i32_attr(dict_ref, "kCGWindowOwnerPID").unwrap_or(0);
        let owner_name = read_string_attr(dict_ref, "kCGWindowOwnerName").unwrap_or_default();
        out.push(WindowInfo {
            number,
            pid,
            owner_name,
        });
    }
    out
}

unsafe fn dict_get_raw(dict: CFDictionaryRef, key: &str) -> Option<CFTypeRef> {
    use core_foundation::dictionary::CFDictionaryGetValueIfPresent;
    let key_cf = CFString::new(key);
    let mut value: *const c_void = std::ptr::null();
    let present = CFDictionaryGetValueIfPresent(
        dict,
        key_cf.as_concrete_TypeRef() as *const c_void,
        &mut value as *mut *const c_void,
    );
    if present != 0 && !value.is_null() {
        Some(value as CFTypeRef)
    } else {
        None
    }
}

unsafe fn read_string_attr(dict: CFDictionaryRef, key: &str) -> Option<String> {
    let v = dict_get_raw(dict, key)?;
    let cf_type = CFType::wrap_under_get_rule(v);
    cf_type.downcast::<CFString>().map(|s| s.to_string())
}

unsafe fn read_i32_attr(dict: CFDictionaryRef, key: &str) -> Option<i32> {
    let v = dict_get_raw(dict, key)?;
    let cf_type = CFType::wrap_under_get_rule(v);
    cf_type.downcast::<CFNumber>().and_then(|n| n.to_i32())
}

unsafe fn read_u32_attr(dict: CFDictionaryRef, key: &str) -> Option<u32> {
    read_i32_attr(dict, key).map(|i| i as u32)
}

/// Walk the AX tree of `pid` (a notification host process) and return the
/// first non-empty AXTitle / AXDescription text we find. Depth-limited;
/// called once per banner detection.
fn ax_extract_sender(pid: i32) -> Option<String> {
    const MAX_DEPTH: usize = 6;
    unsafe {
        let app_ref = AXUIElementCreateApplication(pid);
        if app_ref.is_null() {
            return None;
        }
        let mut result: Option<String> = None;
        descend_first_text(app_ref, &mut result, 0, MAX_DEPTH);
        core_foundation::base::CFRelease(app_ref as _);
        result
    }
}

unsafe fn descend_first_text(
    elem: accessibility_sys::AXUIElementRef,
    out: &mut Option<String>,
    depth: usize,
    max_depth: usize,
) {
    if out.is_some() || depth > max_depth || elem.is_null() {
        return;
    }
    for attr in [kAXTitleAttribute, kAXDescriptionAttribute] {
        if let Some(s) = read_ax_string(elem, attr) {
            if s.chars().count() >= 2 && !is_chrome(&s) {
                *out = Some(s);
                return;
            }
        }
    }
    let children_attr = core_foundation::string::CFString::new(kAXChildrenAttribute);
    let mut children_value: core_foundation::base::CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(
        elem,
        <core_foundation::string::CFString as core_foundation::base::TCFType>::as_concrete_TypeRef(&children_attr)
            as core_foundation::string::CFStringRef,
        &mut children_value,
    );
    if err == kAXErrorSuccess && !children_value.is_null() {
        let arr: core_foundation::array::CFArray<core_foundation::base::CFType> =
            core_foundation::array::CFArray::wrap_under_create_rule(
                children_value as core_foundation::array::CFArrayRef,
            );
        for item in arr.iter() {
            if out.is_some() {
                break;
            }
            let child = item.as_CFTypeRef() as accessibility_sys::AXUIElementRef;
            descend_first_text(child, out, depth + 1, max_depth);
        }
    }
}

unsafe fn read_ax_string(elem: accessibility_sys::AXUIElementRef, attr: &str) -> Option<String> {
    let cf_attr = core_foundation::string::CFString::new(attr);
    let mut value: core_foundation::base::CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(
        elem,
        <core_foundation::string::CFString as core_foundation::base::TCFType>::as_concrete_TypeRef(&cf_attr)
            as core_foundation::string::CFStringRef,
        &mut value,
    );
    if err != kAXErrorSuccess || value.is_null() {
        return None;
    }
    let cf_type = core_foundation::base::CFType::wrap_under_create_rule(value);
    cf_type
        .downcast::<core_foundation::string::CFString>()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

fn is_chrome(s: &str) -> bool {
    matches!(
        s,
        "Notifications"
            | "Notification Center"
            | "Do Not Disturb"
            | "Clear all"
            | "Today"
            | "Earlier"
            | "Yesterday"
            | "Show"
            | "Hide"
            | "Close"
            | "알림 센터"
            | "알림 지우기…"
            | "알림 지우기"
    )
}

impl NotificationSource for MacosNotiSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        *self.running.lock() = true;
        let running = self.running.clone();
        let publish = Arc::new(publish);

        std::thread::spawn(move || {
            let candidates = MacosNotiSource::find_candidate_pids();
            if candidates.is_empty() {
                tracing::warn!("no notification host candidates found at start; thread exiting");
                return;
            }
            for (pid, label) in &candidates {
                tracing::info!(pid, %label, "tracking notification host process");
            }
            tracing::info!(interval_ms = %POLL_INTERVAL.as_millis(), "CGWindowList polling started");

            let mut seen_numbers: HashSet<u32> = HashSet::new();
            unsafe {
                for w in copy_window_list() {
                    if candidates.contains_key(&w.pid) {
                        seen_numbers.insert(w.number);
                    }
                }
            }
            tracing::debug!(initial_seen = seen_numbers.len(), "seeded seen window numbers");

            let mut tick: u64 = 0;
            while *running.lock() {
                std::thread::sleep(POLL_INTERVAL);
                tick += 1;
                let windows = unsafe { copy_window_list() };
                let mut current: HashSet<u32> = HashSet::new();
                for w in &windows {
                    current.insert(w.number);
                    if !candidates.contains_key(&w.pid) {
                        continue;
                    }
                    if seen_numbers.contains(&w.number) {
                        continue;
                    }
                    seen_numbers.insert(w.number);
                    let host_label = candidates
                        .get(&w.pid)
                        .cloned()
                        .unwrap_or_else(|| w.owner_name.clone());
                    let sender = ax_extract_sender(w.pid).unwrap_or_else(|| host_label.clone());
                    tracing::info!(
                        window_num = w.number,
                        pid = w.pid,
                        host = %host_label,
                        sender = %sender,
                        "new banner window detected"
                    );
                    (publish)(NotiEvent::now(
                        host_label,
                        sender,
                        String::new(),
                        String::new(),
                    ));
                }
                seen_numbers.retain(|n| current.contains(n));

                if tick.is_multiple_of(40) {
                    tracing::debug!(
                        tick,
                        visible = windows.len(),
                        tracked_candidates = candidates.len(),
                        "cgwindow poll heartbeat"
                    );
                }
            }
            tracing::info!("CGWindowList polling stopped");
        });

        Ok(())
    }

    fn stop(&self) {
        *self.running.lock() = false;
    }
}
