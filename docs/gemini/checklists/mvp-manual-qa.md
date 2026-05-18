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

## Radial menu (v0.3.0)

### Menu items
- [ ] Settings → Menu items → `+ Add item` → save 3 items: Slack (launch_app, `/Applications/Slack.app`, emoji 💬), GitHub (open_url, `https://github.com`, emoji 🐙), echo (run_shell, `/bin/echo`, args `hello`, confirm=true)
- [ ] App icon extraction: edit Slack item → `Use app icon` button → icon swaps from emoji to real Slack app icon
- [ ] Reorder via ↑/↓ → order persists across restart
- [ ] Delete an item → removed from list

### Hotkey bindings
- [ ] Settings → Hotkey bindings → `+ Add binding`
- [ ] Keyboard capture: `Capture` button → press `Cmd+Shift+M` → accelerator string updates
- [ ] Mouse capture: switch to `Mouse` tab → `Capture` → hold `Cmd` and click mouse button 4 (back) → `button=4 mods=1`
- [ ] Save binding → fires WITHOUT app restart (live re-registration)

### Radial trigger
- [ ] Press the keyboard binding → radial menu appears at cursor position
- [ ] Press the mouse binding → radial appears
- [ ] Hover a sector → highlights
- [ ] Click Slack sector → Slack opens, menu closes
- [ ] Re-trigger → click GitHub sector → browser opens
- [ ] Re-trigger → click echo sector → confirm dialog appears (TODO if not yet implemented) → confirm → command runs
- [ ] ESC during menu → menu closes without executing
- [ ] Click outside the donut ring → menu closes without executing

### Context-aware mode
- [ ] Add a binding with `menu_mode = notification`
- [ ] Tag one menu item with `notification` only (uncheck `launcher`)
- [ ] Trigger an OS notification (Teams/KakaoTalk)
- [ ] Within 5s, press the notification-mode binding → menu shows ONLY notification-tagged items + sender name in center
- [ ] Press the same binding 10s later (after recent_within window expires) → sender label disappears

### Variable sector count
- [ ] Add a 4th menu item → re-trigger → menu now has 4 sectors
- [ ] Add up to 12 items → all visible as sectors
- [ ] Delete down to 1 item → single sector renders as full donut

### Persistence
- [ ] Menu items + hotkey bindings survive app restart
- [ ] Legacy v0.2.x config (no menu_items / hotkey_bindings fields) loads without error
