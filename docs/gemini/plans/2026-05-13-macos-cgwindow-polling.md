# macOS Notification Capture via CGWindowList Polling

> **For agentic workers:** Implement task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Replace the SQLite usernoted polling source with a `CGWindowListCopyWindowInfo`-based polling source so notification indicators appear within ~150ms of banner display instead of the 5-6 second delay caused by `usernoted` writing to the database late.

**Architecture:** A polling thread queries the system window list every 150ms. It tracks visible window numbers belonging to candidate processes (`nbagent`, `NotificationCenter`, `UserNotificationCenter`). When a new window appears under a candidate owner PID, publish a `NotiEvent` with the owner's bundle id / name. Title and body are intentionally left empty for MVP (Ring Pulse style does not render text; Icon Badge can use owner name's initial; Persistent Badge will show owner name). When a candidate window disappears from the list, remove its number from the seen set so a re-displayed banner counts as new.

**Tech Stack:** Rust FFI to CoreGraphics framework (`CGWindowListCopyWindowInfo`). No new crates — uses `core-foundation` already in the macOS target dependencies. `rusqlite` and `plist` deps are removed.

**Related context:**
- Earlier diagnosis: `usernoted` daemon writes `record` rows to the SQLite db ~5 seconds after banner display. Confirmed empirically by `watch sqlite3 ...` showing rec_id increment late.
- Reverting to AX walker is not an option: AX tree of `nbagent`/`NotificationCenter`/`UserNotificationCenter` does not contain banner content for all notifications (Teams subsequent messages were not captured).
- CGWindowList does NOT require Screen Recording permission for owner pid / window number / owner name access. Only `kCGWindowName` (window title) requires it — we don't read that field.

---

## File Structure

```
mouse-noti/
└── src-tauri/
    ├── Cargo.toml                          # remove rusqlite, plist
    └── src/
        └── noti/
            └── macos.rs                    # full rewrite around CGWindowList
```

The Cargo.toml change is small. The macos.rs rewrite is contained to a single file. No public API of `noti` or `overlay` modules changes — `MacosNotiSource` keeps the same `NotificationSource` impl signature and the same `is_trusted()` static helper for the permission UI.

---

## Task 1: Remove SQLite + plist dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1.1: Remove the two lines from the macOS dependencies block**

In `src-tauri/Cargo.toml` find the `[target.'cfg(target_os = "macos")'.dependencies]` block and delete:
```toml
rusqlite = { version = "0.32", features = ["bundled"] }
plist = "1"
```

The block becomes:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = { version = "0.2", features = ["NSString", "NSArray", "NSURL"] }
objc2-app-kit = { version = "0.2", features = ["NSWorkspace", "NSRunningApplication", "libc"] }
core-foundation = "0.10"
core-graphics = "0.24"
accessibility-sys = "0.1"
```

- [ ] **Step 1.2: Skip cargo check until Task 2 lands**

`cargo check` will fail until `macos.rs` is rewritten in Task 2 because the current `macos.rs` imports `rusqlite::*` and `plist::*`. Leave the build broken — Task 2 fixes it in the same commit boundary.

(No commit yet — Task 1 and Task 2 are bundled into a single commit to keep the tree compilable on every commit boundary.)

---

## Task 2: Rewrite `src-tauri/src/noti/macos.rs` to poll CGWindowList

**Files:**
- Modify: `src-tauri/src/noti/macos.rs` (full rewrite)

- [ ] **Step 2.1: Replace the entire file**

Replace `src-tauri/src/noti/macos.rs` with:

```rust
#![cfg(target_os = "macos")]

use super::{NotiEvent, NotificationSource, Publisher};
use accessibility_sys::AXIsProcessTrustedWithOptions;
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

// CoreGraphics window-list FFI
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFArrayRef;
}
const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: u32 = 1 << 0;
const K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;
const K_CG_NULL_WINDOW_ID: u32 = 0;

// Bundle IDs / executable names of processes known to host notification banner UI.
// nbagent (Noticeboard.framework) is the primary host on macOS 14-15. NC and
// UNC fallbacks remain for older versions / sheet-style alerts.
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

    /// Kept for the permission UI. CGWindowList capture does not actually
    /// require Accessibility. This still reports the trust state so existing
    /// callers (`commands::permission_status`) compile unchanged.
    pub fn is_trusted(prompt: bool) -> bool {
        unsafe {
            let key = CFString::new("AXTrustedCheckOptionPrompt");
            let value = core_foundation::boolean::CFBoolean::from(prompt);
            let opts = CFDictionary::from_CFType_pairs(&[(key, value)]);
            AXIsProcessTrustedWithOptions(opts.as_concrete_TypeRef() as _)
        }
    }

    /// Discover the set of candidate notification-host PIDs at startup.
    /// Returns `pid -> human-readable label` (bundle id preferred, exec name
    /// as fallback).
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

/// Copy the on-screen window list (excluding desktop). Returns owner-pid and
/// window-number for every visible window. Does NOT require Screen Recording
/// permission (we never request `kCGWindowName`).
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
        let owner_name =
            read_string_attr(dict_ref, "kCGWindowOwnerName").unwrap_or_default();
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
    let mut value: CFTypeRef = std::ptr::null();
    let present = CFDictionaryGetValueIfPresent(
        dict,
        key_cf.as_concrete_TypeRef() as *const c_void,
        &mut value as *mut CFTypeRef as *mut *const c_void,
    );
    if present != 0 && !value.is_null() {
        Some(value)
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
    cf_type
        .downcast::<CFNumber>()
        .and_then(|n| n.to_i32())
}

unsafe fn read_u32_attr(dict: CFDictionaryRef, key: &str) -> Option<u32> {
    read_i32_attr(dict, key).map(|i| i as u32)
}

impl NotificationSource for MacosNotiSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        *self.running.lock() = true;
        let running = self.running.clone();
        let publish = Arc::new(publish);

        std::thread::spawn(move || {
            // Refresh candidate PIDs once on startup. PIDs change across reboots
            // but the daemons are long-lived during a single session.
            let candidates = MacosNotiSource::find_candidate_pids();
            if candidates.is_empty() {
                tracing::warn!("no notification host candidates found at start; thread exiting");
                return;
            }
            for (pid, label) in &candidates {
                tracing::info!(pid, %label, "tracking notification host process");
            }
            tracing::info!(interval_ms = %POLL_INTERVAL.as_millis(), "CGWindowList polling started");

            // Seed seen_numbers with currently visible candidate windows so we
            // don't fire a flurry of "new" events at startup.
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
                    let label = candidates
                        .get(&w.pid)
                        .cloned()
                        .unwrap_or_else(|| w.owner_name.clone());
                    tracing::info!(
                        window_num = w.number,
                        pid = w.pid,
                        owner = %label,
                        "new banner window detected"
                    );
                    (publish)(NotiEvent::now(
                        label.clone(),
                        label,
                        String::new(),
                        String::new(),
                    ));
                }
                // GC: drop seen entries whose windows have closed, so a re-shown
                // banner with a new window number fires again.
                seen_numbers.retain(|n| current.contains(n));

                if tick % 40 == 0 {
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
```

Notes for the implementer:
- Every `unsafe` block boundary contains exactly one logical FFI hop. No long unsafe regions.
- The `dict_get_raw` helper uses `CFDictionaryGetValueIfPresent` because the keys are CFString constants we look up by string content. `core-foundation` 0.10 exposes that function under `core_foundation::dictionary`. If the path differs, fall back to `core_foundation_sys::dictionary::CFDictionaryGetValueIfPresent` (raw sys crate) and add `core-foundation-sys` to the deps.
- `wrap_under_create_rule` consumes the +1 retain from `CGWindowListCopyWindowInfo` (Apple's "Create" naming rule). `wrap_under_get_rule` does NOT consume — used on values fetched via `CFDictionaryGetValueIfPresent` (Apple's "Get" rule).
- Polling is on a `std::thread`, not `tauri::async_runtime`. Same as the previous SQLite implementation.

- [ ] **Step 2.2: Build**

Run from `/Users/kanghyeon/DEV/mouse-noti`:
```bash
cargo check --manifest-path src-tauri/Cargo.toml
```
Expected: clean compile, zero warnings.

If `CFDictionaryGetValueIfPresent` is not visible from `core_foundation::dictionary`, edit `Cargo.toml` to add `core-foundation-sys = "0.8"` and change the import:
```rust
use core_foundation_sys::dictionary::CFDictionaryGetValueIfPresent;
```

- [ ] **Step 2.3: Run existing tests**

Run:
```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
```
Expected: 10 passed (no new tests, but verify no regressions).

- [ ] **Step 2.4: Commit Task 1 + Task 2 together**

```bash
git add src-tauri/Cargo.toml src-tauri/src/noti/macos.rs
git commit -m "fix(noti/macos): switch to CGWindowList polling for real-time banner detection"
```

The commit message body should describe the user-visible behavior change: indicator latency drops from ~5s (usernoted db write lag) to ~150ms (CGWindowList poll interval). Title/body are now empty per event because CGWindowList does not require Screen Recording permission and we deliberately avoid `kCGWindowName`.

---

## Task 3: Update README + manual QA checklist

**Files:**
- Modify: `README.md`
- Modify: `docs/gemini/checklists/mvp-manual-qa.md`

- [ ] **Step 3.1: README — adjust permissions section**

In `README.md` find the macOS permissions section. Replace the wording so it reflects that Accessibility is no longer required for indicator firing (only used for AX-trust reporting that future Phase 2 features may need). Keep instructions for how to grant if user wants future readiness.

Add a short note under "알려진 제약":
```markdown
- 알림 인디케이터는 banner UI 표시 시점을 기준으로 함. macOS가 banner를 표시하지 않은 알림(do-not-disturb, focus mode 차단된 알림 등)은 인디케이터도 발생하지 않음.
- 현재 buildhealth 인디케이터는 sender 식별 가능하나 title/body 텍스트는 비어있음. Icon Badge / Persistent Badge 스타일이 표시하는 텍스트는 sender의 owner name 첫 글자 / 전체 이름.
```

- [ ] **Step 3.2: QA checklist — bump expected latency**

In `docs/gemini/checklists/mvp-manual-qa.md` find the "Indicator" section and add an explicit latency check:
```markdown
- [ ] Trigger a real OS notification (Slack message, Teams DM, Calendar alert)
- [ ] Indicator appears at cursor within ~300ms of the banner becoming visible
- [ ] Trigger a burst of 3+ notifications from different apps within 1 second
- [ ] Each indicator fires; same (app, title) within 1s is debounced (this is intentional)
```

- [ ] **Step 3.3: Commit**

```bash
git add README.md docs/gemini/checklists/mvp-manual-qa.md
git commit -m "docs: update permissions notes and QA latency expectations after CGWindowList switch"
```

---

## Task 4: Manual verification (operator-driven)

This task is for the human operator, not a subagent.

- [ ] **Step 4.1: Run app**

```bash
RUST_LOG=mouse_noti=debug pnpm tauri dev
```

Expected startup log lines:
```
INFO ... tracking notification host process pid=<pid> label=<bundle-or-exec-name>
INFO ... CGWindowList polling started interval_ms=150
DEBUG ... seeded seen window numbers initial_seen=<n>
```

- [ ] **Step 4.2: Trigger a real notification**

Best signal: a Teams DM or a Calendar alert (osascript notifications are silently dropped under macOS 15 unless Script Editor has Notification permission).

When the banner becomes visible, the log should show:
```
INFO ... new banner window detected window_num=<num> pid=<pid> owner=<label>
INFO ... overlay received event title= app=<label>
INFO ... emitted noti:show to webview local=(...)
```

The visible-banner-to-indicator delay should subjectively be ~150-300ms (1 poll interval + Tauri emit).

- [ ] **Step 4.3: Check burst behavior**

Trigger several notifications in close succession (multiple Teams messages, or send several emails to yourself). Each should produce a `new banner window detected` log. Indicators should fire one per banner, with the EventBus `(app_id, title)` debounce only suppressing exact-duplicate (app_id, title) combos within 1 second — which for empty titles means each banner *does* get suppressed by debounce since all share `(label, "")`. **If this turns out to be a problem in practice, change the debounce key on the macOS source side by using the window_num as part of the app_id, e.g. `format!("{label}#{window_num}")`. Note this in a follow-up task; do not change here without confirming the issue.**

- [ ] **Step 4.4: Check window close GC**

After a banner is shown and auto-dismissed, trigger another banner from the same app. A new window number should appear, fire a new `new banner window detected`, and the indicator should appear. If the second banner does NOT fire, the GC line `seen_numbers.retain(|n| current.contains(n))` is not removing dismissed numbers in time — debug by lowering POLL_INTERVAL or adding more visibility in the GC path.

---

## Definition of Done

- [ ] Task 1 + 2: `cargo check` + `cargo test --lib` clean; new commit replaces SQLite path with CGWindowList path.
- [ ] Task 3: README and QA checklist updated.
- [ ] Task 4: Operator confirms latency < 300ms for at least one real notification on macOS 15.
- [ ] No new clippy warnings introduced (`cargo clippy --manifest-path src-tauri/Cargo.toml --lib --all-targets`).
- [ ] Frontend (`pnpm test:run`) still 6/6 green.

---

## Risks and Open Questions

| Risk | Mitigation |
|---|---|
| EventBus debounce kills bursts because title is empty | If observed in Task 4.3, switch macOS event `app_id` to include `window_num` to make the dedup key per-banner. |
| Candidate PIDs miss the actual banner owner on some macOS versions | The candidate list covers nbagent + NC + UNC. If a future macOS rev moves the banner to another process, the log "seeded seen window numbers initial_seen=0" with no `new banner window detected` after a real notification reveals it. Diagnose by `ps -axo pid,comm` while a banner is visible, then extend `CANDIDATE_BUNDLE_IDS` / `CANDIDATE_EXEC_NAMES`. |
| nbagent has long-lived non-banner windows (settings, sheets) that we'd misfire on | The seed step at startup adds existing windows to `seen_numbers`, so only windows that appear AFTER startup fire. Long-running pre-existing nbagent windows won't fire. |
| Window appears + disappears within one 150ms poll | Theoretically possible. We'd miss it. Lower POLL_INTERVAL to 100ms if observed. |
| `CFDictionaryGetValueIfPresent` not exposed by `core-foundation` 0.10 | Add `core-foundation-sys` dep and use raw FFI. |
| Apple removes / restricts CGWindowList | Out of scope for MVP. If it happens we'd fall back to the SQLite path with the known 5s delay. |
| Screen Recording prompt despite our claim of "not needed" | macOS 12.3+ may still prompt if any window has a non-default sharing state. We never call `CGWindowListCreateImage`. If a prompt appears, document in README. |

---

## Why we abandoned SQLite usernoted polling

For the implementer's reference and to avoid re-litigating in future work:

1. `~/Library/Group Containers/group.com.apple.usernoted/db2/db` is written to by `usernoted` daemon.
2. `record.delivered_date` matches the banner display moment (confirmed via Cocoa-epoch conversion).
3. `record` row INSERTs land in the SQLite db **several seconds after** `delivered_date`.
4. The lag is not in our reader — confirmed by reopening the connection every poll and by an external `watch sqlite3 'SELECT MAX(rec_id) ...'` showing the same lag.
5. Conclusion: the SQLite db is a post-hoc journal, not a real-time event stream. Unsuitable for indicator triggering.

The CGWindowList approach trades title/body content for real-time signal. That trade matches the MVP goal (visual indicator, no required text) and gives the user-visible behavior the spec calls for.
