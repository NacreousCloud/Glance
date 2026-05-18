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
  const label = isMac ? 'Accessibility' : 'Notifications';
  const settingsUri = isMac ? MAC_ACCESSIBILITY_URI : WIN_NOTIFICATIONS_URI;

  const unlocks = isMac
    ? [
        'Notification detection',
        'Mouse & Trackpad gestures',
      ]
    : ['System notification capture'];

  if (granted) {
    return (
      <div className="ios-card">
        <div className="ios-item">
          <div className="flex flex-col">
            <span className="ios-title">{label}</span>
            <span className="ios-subtitle text-ios-system-green">Permission Granted</span>
          </div>
          <div className="w-6 h-6 rounded-full bg-ios-system-green flex items-center justify-center text-white">
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
            </svg>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-1.5">
      <h2 className="px-4 text-[13px] uppercase tracking-wider text-ios-system-red">
        Action Required
      </h2>
      <div className="ios-card border-ios-system-red/30 bg-ios-system-red/[0.03]">
        <div className="p-4 space-y-4">
          <div className="flex items-start gap-3">
            <div className="w-10 h-10 rounded-full bg-ios-system-red/10 flex items-center justify-center text-ios-system-red shrink-0">
              <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <div>
              <h3 className="ios-title text-ios-system-red">{label} Access Required</h3>
              <p className="text-[13px] text-gray-600 dark:text-gray-400 mt-1 leading-snug">
                Glance needs this to detect notifications and mouse gestures.
                Without it, features like {unlocks.join(', ')} will be disabled.
              </p>
            </div>
          </div>

          <div className="grid grid-cols-1 gap-2">
            <button
              type="button"
              className="w-full py-2.5 bg-ios-system-blue text-white rounded-ios font-semibold text-[15px] active:opacity-80 transition-opacity"
              onClick={() => openUrl(settingsUri).catch(() => {})}
            >
              Open System Settings
            </button>
            <div className="flex gap-2">
              <button
                type="button"
                className="flex-1 py-2 bg-gray-200 dark:bg-white/10 text-ios-label-primary dark:text-white rounded-ios text-[14px] font-medium"
                onClick={() => requestPermission().then(refresh).catch(() => {})}
              >
                Prompt
              </button>
              <button
                type="button"
                className="flex-1 py-2 bg-gray-200 dark:bg-white/10 text-ios-label-primary dark:text-white rounded-ios text-[14px] font-medium"
                onClick={refresh}
              >
                Re-check
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
