# Sender Identification + Hash-based Indicator Color

> **For agentic workers:** Implement task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** When a notification banner is detected, extract the sender's name from the banner's AX tree (best-effort) and color the indicator with a hash-derived hue so the user can identify the sending app at a glance.

**Architecture:**
1. On new banner window detection (CGWindowList), perform a focused AX walk on the host process (`nbagent` / `NotificationCenter`) to extract the first meaningful text element from that specific window.
2. Use the extracted sender text as `app_name` (and as the hash seed for color).
3. Compute a stable `color_hue` (0–359, integer) per sender label and ship it in `IndicatorPayload`.
4. Frontend renders each indicator style with the per-sender accent color (border for RingPulse, background for IconBadge / PersistentBadge).

**Tech Stack:** No new crates. Reuses `accessibility-sys`, `core-foundation` (already in deps). Adds one Rust hash helper. Frontend uses native CSS HSL.

**Related context:**
- Earlier diagnosis confirmed: nbagent is the banner host on macOS 15.
- CGWindowList does not expose window text without Screen Recording permission, hence the AX walk path.
- Accessibility permission is already granted in the user's setup; we already call `is_trusted` in the permission UI.

---

## File Structure

```
mouse-noti/
└── src-tauri/
    └── src/
        ├── noti/
        │   └── macos.rs            # add ax_extract_sender(pid, window_num)
        └── overlay/
            └── mod.rs              # compute hash → hue, ship in payload
mouse-noti/
└── src/
    └── overlay/
        ├── Overlay.tsx             # extend Payload type with color_hue
        ├── RingPulse.tsx           # accept hue prop, use HSL border
        ├── IconBadge.tsx           # accept hue prop, use HSL background
        └── PersistentBadge.tsx     # accept hue prop, use HSL background
```

---

## Task 1: AX-extract sender text in `noti/macos.rs`

**Files:**
- Modify: `src-tauri/src/noti/macos.rs`

- [ ] **Step 1.1: Add `ax_extract_sender(pid: i32) -> Option<String>`**

Place this above the `impl NotificationSource for MacosNotiSource` block:

```rust
use accessibility_sys::{
    kAXChildrenAttribute, kAXDescriptionAttribute, kAXErrorSuccess, kAXTitleAttribute,
    AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
};

/// Walk the AX tree of `pid` (a notification host process) and return the
/// first non-empty AXTitle / AXDescription text we find. Banner content
/// on macOS 15's nbagent typically exposes:
///   - top-level AXWindow
///     - AXStaticText (sender app name)   ← what we want
///     - AXStaticText (body)
///     - AXImage (sender icon)
///     - buttons
/// We grab the first non-trivial string. Depth-limited; called only at
/// banner-detect time so cost is one quick walk per banner.
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
            // Filter out single chars and chrome strings.
            if s.chars().count() >= 2 && !is_chrome(&s) {
                *out = Some(s);
                return;
            }
        }
    }
    // Recurse into children.
    let children_attr = core_foundation::string::CFString::new(kAXChildrenAttribute);
    let mut children_value: core_foundation::base::CFTypeRef = std::ptr::null();
    let err = AXUIElementCopyAttributeValue(
        elem,
        core_foundation::base::TCFType::as_concrete_TypeRef(&children_attr)
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
        core_foundation::base::TCFType::as_concrete_TypeRef(&cf_attr)
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
```

- [ ] **Step 1.2: Call `ax_extract_sender` at banner detection**

Inside the polling loop, replace the publish block:

```rust
// existing:
let label = candidates
    .get(&w.pid)
    .cloned()
    .unwrap_or_else(|| w.owner_name.clone());
(publish)(NotiEvent::now(label.clone(), label, String::new(), String::new()));
```

with:

```rust
let host_label = candidates
    .get(&w.pid)
    .cloned()
    .unwrap_or_else(|| w.owner_name.clone());
// Best-effort: read banner text from the AX tree of this host process.
// May return None for chrome-only windows or if AX permission is missing.
let sender = ax_extract_sender(w.pid).unwrap_or_else(|| host_label.clone());
tracing::info!(
    window_num = w.number,
    pid = w.pid,
    host = %host_label,
    sender = %sender,
    "new banner window detected"
);
(publish)(NotiEvent::now(
    host_label,           // app_id stays host-based for dedup
    sender,               // app_name carries human-readable sender
    String::new(),
    String::new(),
));
```

Rationale:
- `app_id` = host_label keeps EventBus dedup stable (`(host_label, "")` → second banner from same host within 1s still gets debounced; if that turns out to be too aggressive we can swap to `(sender, "")` in a follow-up).
- `app_name` = sender carries the human-readable text into the indicator. Frontend hashes this for color.

- [ ] **Step 1.3: Build + test**

```bash
cd /Users/kanghyeon/DEV/mouse-noti
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Expected: clean, 10/10 tests pass.

- [ ] **Step 1.4: Commit**

```bash
git add src-tauri/src/noti/macos.rs
git commit -m "feat(noti/macos): extract sender text from banner AX tree"
```

---

## Task 2: Add `color_hue` to `IndicatorPayload`

**Files:**
- Modify: `src-tauri/src/overlay/mod.rs`

- [ ] **Step 2.1: Add hash → hue helper**

At the top of `overlay/mod.rs`, after imports:

```rust
use std::hash::{Hash, Hasher};

fn hue_from_label(label: &str) -> u16 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut hasher);
    (hasher.finish() % 360) as u16
}
```

- [ ] **Step 2.2: Add `color_hue: u16` field to `IndicatorPayload`**

```rust
#[derive(Serialize, Clone)]
pub struct IndicatorPayload {
    pub id: String,
    pub timestamp_ms: u64,
    pub style: IndicatorStyle,
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub viewport_w: f64,
    pub viewport_h: f64,
    pub app_name: String,
    pub title: String,
    pub color_hue: u16,
}
```

- [ ] **Step 2.3: Populate `color_hue` in the spawn loop**

In the `let payload = IndicatorPayload { ... }` block, add:

```rust
color_hue: hue_from_label(&event.app_name),
```

(Fall back to using `app_id` if `app_name` is empty — unlikely after Task 1.)

- [ ] **Step 2.4: Build**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

Expected: clean, 10/10.

- [ ] **Step 2.5: Commit**

```bash
git add src-tauri/src/overlay/mod.rs
git commit -m "feat(overlay): derive color_hue per sender from hash"
```

---

## Task 3: Frontend — apply color per sender to all three styles

**Files:**
- Modify: `src/overlay/Overlay.tsx`
- Modify: `src/overlay/RingPulse.tsx`
- Modify: `src/overlay/IconBadge.tsx`
- Modify: `src/overlay/PersistentBadge.tsx`
- Modify: `src/overlay/Overlay.test.tsx` (extend tests for the hue prop)

- [ ] **Step 3.1: Extend `Payload` type and thread `color_hue`**

In `src/overlay/Overlay.tsx`, add `color_hue: number;` to `Payload`. Then thread it into each indicator component:

```tsx
case 'ring_pulse':
  return <RingPulse key={active.id} x={x} y={y} hue={active.color_hue} />;
case 'icon_badge':
  return (
    <IconBadge
      key={active.id}
      x={x}
      y={y}
      appName={active.app_name}
      hue={active.color_hue}
    />
  );
case 'persistent_badge':
  return (
    <PersistentBadge
      key={active.id}
      x={x}
      y={y}
      appName={active.app_name}
      hue={active.color_hue}
    />
  );
```

- [ ] **Step 3.2: RingPulse — use HSL border**

```tsx
type Props = { x: number; y: number; hue: number };
export default function RingPulse({ x, y, hue }: Props) {
  const size = 48;
  const color = `hsl(${hue} 70% 55%)`;
  return (
    <div
      data-testid="ring-pulse"
      style={{
        position: 'absolute',
        left: x - size / 2,
        top: y - size / 2,
        width: size,
        height: size,
        borderRadius: '50%',
        border: `3px solid ${color}`,
        animation: 'mn-pulse 900ms ease-out forwards',
      }}
    />
  );
}
```

- [ ] **Step 3.3: IconBadge — HSL background, contrast-aware text**

```tsx
type Props = { x: number; y: number; appName: string; hue: number };
export default function IconBadge({ x, y, appName, hue }: Props) {
  const initial = appName.trim().slice(0, 1).toUpperCase() || '?';
  const bg = `hsl(${hue} 70% 45%)`;
  return (
    <div
      data-testid="icon-badge"
      style={{
        position: 'absolute',
        left: x + 12,
        top: y + 12,
        width: 28,
        height: 28,
        borderRadius: 8,
        background: bg,
        color: 'white',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontWeight: 700,
        boxShadow: '0 2px 6px rgba(0,0,0,0.3)',
      }}
    >
      {initial}
    </div>
  );
}
```

- [ ] **Step 3.4: PersistentBadge — HSL pill background**

```tsx
type Props = { x: number; y: number; appName: string; hue: number };
export default function PersistentBadge({ x, y, appName, hue }: Props) {
  const bg = `hsl(${hue} 70% 35% / 0.9)`;
  return (
    <div
      data-testid="persistent-badge"
      style={{
        position: 'absolute',
        left: x,
        top: y,
        padding: '4px 8px',
        borderRadius: 12,
        background: bg,
        color: 'white',
        fontSize: 12,
        whiteSpace: 'nowrap',
      }}
    >
      {appName}
    </div>
  );
}
```

- [ ] **Step 3.5: Update Vitest cases**

`src/overlay/Overlay.test.tsx` — pass `hue={0}` (or some constant) in the existing render() calls. Add one extra assertion that verifies the style includes an `hsl(` substring so a future regression that wires hue incorrectly fails loudly.

Existing test calls become:
```tsx
render(<RingPulse x={100} y={150} hue={120} />);
// ...
render(<IconBadge x={0} y={0} appName="Slack" hue={210} />);
// ...
render(<PersistentBadge x={10} y={20} appName="Mail" hue={45} />);
```

Add:
```tsx
it('RingPulse uses hue in border color', () => {
  render(<RingPulse x={0} y={0} hue={210} />);
  const ring = screen.getByTestId('ring-pulse');
  expect(ring.getAttribute('style') ?? '').toMatch(/hsl\(210/);
});
```

- [ ] **Step 3.6: Run tests**

```bash
pnpm test:run
```

Expected: 7 passed (was 6, +1 new).

- [ ] **Step 3.7: Commit**

```bash
git add src/overlay
git commit -m "feat(overlay): color each indicator by sender hue"
```

---

## Task 4: Bump version + tag (after manual sign-off)

This task is for the operator. Don't run it in a subagent.

- [ ] **Step 4.1: Manually verify**

```bash
RUST_LOG=mouse_noti=debug pnpm tauri dev
```

Trigger notifications from at least three different senders (e.g. Teams, KakaoTalk, Calendar, Slack). Confirm:
- Log: `new banner window detected sender="<some text>"` shows different sender text per app.
- Indicators visually have different colors per sender.
- Same sender → same color across multiple notifications.
- AX-extraction failure on some apps still renders the indicator with a host-label fallback color (not a hard error).

If color collisions are annoying (e.g., two apps happen to land near identical hues), note it for a future "preset brand colors" follow-up — out of scope here.

- [ ] **Step 4.2: Version bump**

Bump `0.2.0` → `0.3.0` in:
- `src-tauri/Cargo.toml`
- `package.json`
- `src-tauri/tauri.conf.json`

- [ ] **Step 4.3: Commit + tag**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock package.json src-tauri/tauri.conf.json
git commit -m "chore: bump version to 0.3.0"
git tag -a v0.3.0 -m "v0.3.0 — sender-derived indicator color"
```

---

## Risks and Open Questions

| Risk | Mitigation |
|---|---|
| AX walk on nbagent returns chrome strings ("알림 센터") instead of sender | `is_chrome` filter rejects known chrome titles. Add new strings as they're observed. |
| AX walk fails for some senders (custom banner UI like KakaoTalk's own window) | Fall back to host_label so we never lose the indicator. Color will collapse for that sender but indicator still fires. |
| AX walk is slow and adds latency | Depth-limited to 6 and runs only on banner detection (not per poll). Expected <5ms. If we observe regression in the 150ms latency goal, lower max_depth or cache. |
| Hash collisions between distinct senders | Acceptable for MVP; users rely on indicator existence more than precise color. Follow-up: optional manual override in settings. |
| `app_id` set to `host_label` instead of `sender` means cross-app bursts get debounced | Empirically banners from different senders arrive >>1s apart from each other. Re-evaluate after dogfood. If it's a problem, switch to `(sender, "")` debounce key on macOS only. |
| `DefaultHasher` is unstable across Rust versions | Same Rust toolchain across runs in practice, so colors are stable per build. If we ever serialize and compare across processes, switch to `siphash` or `fxhash`. |

---

## Definition of Done

- [ ] `cargo check --manifest-path src-tauri/Cargo.toml`: clean.
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --all-targets`: zero new warnings.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --lib`: 10/10 pass.
- [ ] `pnpm test:run`: 7/7 pass (existing 6 + 1 new hue assertion).
- [ ] Operator confirms three distinct senders produce three visually distinct indicator colors and the same sender reproduces the same color across multiple notifications.
- [ ] Tagged `v0.3.0`.
