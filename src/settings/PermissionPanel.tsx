import { useEffect, useState } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { permissionStatus, requestPermission, type PermissionStatus } from './api';

const MAC_ACCESSIBILITY_URI =
  'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility';
const WIN_NOTIFICATIONS_URI = 'ms-settings:notifications';

export default function PermissionPanel() {
  const [status, setStatus] = useState<PermissionStatus | null>(null);

  const refresh = () => permissionStatus().then(setStatus);
  useEffect(() => {
    refresh();
    const id = window.setInterval(refresh, 2000);
    return () => window.clearInterval(id);
  }, []);

  if (!status) return null;

  const isMac = status.platform === 'macos';
  const granted = isMac ? status.accessibility_ok : status.notification_listener_ok;
  const label = isMac ? 'Accessibility' : 'Notification listener';
  const settingsUri = isMac ? MAC_ACCESSIBILITY_URI : WIN_NOTIFICATIONS_URI;

  const unlocks = isMac
    ? [
        'Notification detection (CGWindowList polling)',
        'Mouse-button hotkeys',
        'Force Touch + trackpad-tap hotkeys',
      ]
    : ['System notification capture (UserNotificationListener)'];

  const stepHints = isMac
    ? [
        'Click "Open System Settings" below.',
        'In "Privacy & Security → Accessibility", toggle Glance ON.',
        'Quit Glance from the tray icon, then relaunch it (macOS caches the trust state per binary hash).',
      ]
    : [
        'Click "Open System Settings" below.',
        'In "System → Notifications", make sure Glance is allowed.',
        'If the listener is still blocked, sign out and back in once (Windows caches the consent).',
      ];

  return (
    <section className="space-y-2">
      <h2 className="text-sm font-semibold">Permissions</h2>
      {granted ? (
        <div className="flex items-center justify-between rounded border p-3">
          <div>
            <div className="font-medium">{label}</div>
            <div className="text-green-600 text-sm">Granted</div>
          </div>
        </div>
      ) : (
        <div className="rounded border border-red-300 bg-red-50 p-3 space-y-2">
          <div className="font-medium text-red-700">
            ⚠️ {label} permission required
          </div>
          <div className="text-xs text-gray-700">
            Without it the following stops working:
            <ul className="list-disc list-inside mt-1">
              {unlocks.map((u) => (
                <li key={u}>{u}</li>
              ))}
            </ul>
          </div>
          <ol className="text-xs text-gray-700 list-decimal list-inside space-y-0.5">
            {stepHints.map((s, i) => (
              <li key={i}>{s}</li>
            ))}
          </ol>
          <div className="flex gap-2 pt-1">
            <button
              type="button"
              className="px-3 py-1 bg-blue-600 text-white rounded text-sm"
              onClick={() => requestPermission().then(refresh).catch(() => {})}
            >
              Trigger system prompt
            </button>
            <button
              type="button"
              className="px-3 py-1 bg-gray-200 text-gray-800 rounded text-sm"
              onClick={() => openUrl(settingsUri).catch(() => {})}
            >
              Open System Settings
            </button>
            <button
              type="button"
              className="px-3 py-1 text-sm text-gray-600 hover:text-gray-900"
              onClick={refresh}
            >
              Re-check
            </button>
          </div>
        </div>
      )}
    </section>
  );
}
