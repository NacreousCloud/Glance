# mouse-noti MVP Manual QA

## Setup
- [ ] `pnpm install`
- [ ] `pnpm tauri dev --features mock-os` (or release build)
- [ ] Tray icon visible
- [ ] Settings window opens from tray

## Permissions
### macOS
- [ ] First launch shows Accessibility prompt
- [ ] Denying permission: PermissionPanel shows "Required" with Grant button
- [ ] Granting + restart: PermissionPanel shows "Granted"
### Windows
- [ ] First launch shows toast listener consent
- [ ] Denying: PermissionPanel shows "Required" with Grant button

## Indicator (per style)
For each of `ring_pulse`, `icon_badge`, `persistent_badge`:
- [ ] Select style in Settings
- [ ] Trigger a real OS notification (Slack message, calendar reminder, etc.)
- [ ] Indicator appears at cursor position
- [ ] Indicator does NOT block clicks beneath it
- [ ] Animation/fade behaves correctly

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
