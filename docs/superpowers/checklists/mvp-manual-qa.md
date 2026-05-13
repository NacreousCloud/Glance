# mouse-noti MVP Manual QA

## Setup
- [ ] `pnpm install`
- [ ] `pnpm tauri dev --features mock-os` (or release build)
- [ ] Tray icon visible
- [ ] Settings window opens from tray

## Permissions
### macOS
- [ ] Indicator fires even WITHOUT Accessibility (CGWindowList does not require it)
- [ ] Accessibility prompt may appear if Grant button pressed in PermissionPanel; this is for forward-compat with Phase 2 (radial menu)
- [ ] Denying Accessibility: PermissionPanel shows "Required" but indicator still works
- [ ] Granting + restart: PermissionPanel shows "Granted"
### Windows
- [ ] First launch shows toast listener consent
- [ ] Denying: PermissionPanel shows "Required" with Grant button

## Indicator (per style)
For each of `ring_pulse`, `icon_badge`, `persistent_badge`:
- [ ] Select style in Settings
- [ ] Trigger a real OS notification (Slack message, Teams DM, Calendar reminder)
- [ ] Indicator appears at cursor within ~300ms of the banner becoming visible
- [ ] Indicator does NOT block clicks beneath it
- [ ] Animation/fade behaves correctly

## Latency (macOS CGWindowList)
- [ ] Trigger a Teams or Slack notification
- [ ] Subjectively confirm indicator appears within ~150-300ms (no perceptible lag)
- [ ] Trigger 3+ notifications from different apps within 1 second
- [ ] Each banner produces a `new banner window detected` log line
- [ ] Indicators fire per banner (note: same (app, "") within 1s may be debounced — follow-up if user-reported)

## Mock injection (dev only)
- [ ] In DevTools console: `await window.__TAURI__.core.invoke('inject_mock_event', { appId: 'test', appName: 'Test', title: 't', body: 'b' })`
- [ ] Indicator appears within 100ms

## Debounce
- [ ] Trigger 5 notifications from the same app within 1 second
- [ ] Only first indicator appears (others suppressed)

## Multi-monitor
- [ ] Move cursor to secondary monitor → trigger notification
- [ ] Indicator appears on secondary monitor

## Autostart
- [ ] Enable "Launch at login"
- [ ] Logout/login → mouse-noti is running

## Persistence
- [ ] Change style → quit app → relaunch → style preserved
- [ ] Corrupt config.toml manually → relaunch → defaults loaded, `.bak` recovered
