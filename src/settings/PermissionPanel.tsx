import { useEffect, useState } from 'react';
import { permissionStatus, requestPermission, type PermissionStatus } from './api';

export default function PermissionPanel() {
  const [status, setStatus] = useState<PermissionStatus | null>(null);

  const refresh = () => permissionStatus().then(setStatus);
  useEffect(() => {
    refresh();
    const id = window.setInterval(refresh, 2000);
    return () => window.clearInterval(id);
  }, []);

  if (!status) return null;
  const required =
    status.platform === 'macos' ? status.accessibility_ok : status.notification_listener_ok;
  const label = status.platform === 'macos' ? 'Accessibility' : 'Notification listener';

  return (
    <section className="space-y-2">
      <h2 className="text-sm font-semibold">Permissions</h2>
      <div className="flex items-center justify-between border rounded p-3">
        <div>
          <div className="font-medium">{label}</div>
          <div className={required ? 'text-green-600 text-sm' : 'text-red-600 text-sm'}>
            {required ? 'Granted' : 'Required'}
          </div>
        </div>
        {!required && (
          <button
            className="px-3 py-1 bg-blue-600 text-white rounded text-sm"
            onClick={() => requestPermission().then(refresh)}
          >
            Grant
          </button>
        )}
      </div>
    </section>
  );
}
